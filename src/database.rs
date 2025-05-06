use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::Path;
use serde::{Serialize, Deserialize};
use chrono::{Local, DateTime, NaiveDate, Utc, TimeZone}; // Added TimeZone import
use crate::models::{Company, Customer, InvoiceItem, Invoice};
// Removed unused utils import: use crate::utils::*;
use crate::pdf_generator::generate_pdf;

const DB_FILENAME: &str = "database.json";
const MAX_BACKUPS: usize = 5;

#[derive(Debug)]
pub enum DatabaseError {
    Io(io::Error),
    Serialization(serde_json::Error),
    CustomerExists(String),
    CustomerNotFound(String),
    InvoiceNotFound(String),
    InvalidInput(String),
    PdfGeneration(String),
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<
'_>) -> std::fmt::Result {
        match self {
            DatabaseError::Io(e) => write!(f, "I/O Error: {}", e),
            DatabaseError::Serialization(e) => write!(f, "Serialization Error: {}", e),
            DatabaseError::CustomerExists(name) => write!(f, "Customer already exists: {}", name),
            DatabaseError::CustomerNotFound(name) => write!(f, "Customer not found: {}", name),
            DatabaseError::InvoiceNotFound(num) => write!(f, "Invoice not found: {}", num),
            DatabaseError::InvalidInput(msg) => write!(f, "Invalid Input: {}", msg),
            DatabaseError::PdfGeneration(msg) => write!(f, "PDF Generation Error: {}", msg),
        }
    }
}

impl std::error::Error for DatabaseError {}

impl From<io::Error> for DatabaseError {
    fn from(err: io::Error) -> DatabaseError {
        DatabaseError::Io(err)
    }
}

impl From<serde_json::Error> for DatabaseError {
    fn from(err: serde_json::Error) -> DatabaseError {
        DatabaseError::Serialization(err)
    }
}

impl From<Box<dyn std::error::Error>> for DatabaseError {
    fn from(err: Box<dyn std::error::Error>) -> DatabaseError {
        DatabaseError::PdfGeneration(err.to_string())
    }
}


#[derive(Serialize, Deserialize)]
pub struct Database {
    pub company: Company,
    pub customers: HashMap<String, Customer>,
    pub invoices: HashMap<String, Invoice>,
    pub last_invoice_nums: HashMap<String, u32>,
}

impl Database {
    pub fn new() -> Self {
        Database {
            company: Company {
                name: "JMATTS CLEANING Canberra".to_string(),
                abn: "78734213681".to_string(),
                address: "40 Wyndham Avenue Denman Prospect, ACT, 2611".to_string(),
                phone: "0403-491446".to_string(),
            },
            customers: HashMap::new(),
            invoices: HashMap::new(),
            last_invoice_nums: HashMap::new(),
        }
    }

    // Function to handle backup creation and rotation
    fn backup_database() -> Result<(), io::Error> {
        let db_path = Path::new(DB_FILENAME);
        if !db_path.exists() {
            return Ok(()); // No database file to back up
        }

        // Create backup filename with timestamp
        let timestamp = Utc::now().format("%Y%m%d%H%M%S");
        let backup_filename = format!("{}.{}.bak", DB_FILENAME, timestamp);
        fs::copy(db_path, &backup_filename)?;
        println!("Database backed up to {}", backup_filename);

        // Manage old backups
        let mut backups = Vec::new();
        for entry in fs::read_dir(".")? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if filename.starts_with(DB_FILENAME) && filename.ends_with(".bak") {
                        backups.push(path.clone());
                    }
                }
            }
        }

        // Sort backups by filename (timestamp)
        backups.sort();

        // Remove oldest backups if count exceeds MAX_BACKUPS
        if backups.len() > MAX_BACKUPS {
            let num_to_remove = backups.len() - MAX_BACKUPS;
            for i in 0..num_to_remove {
                if let Some(filename) = backups[i].file_name().and_then(|n| n.to_str()) {
                     match fs::remove_file(&backups[i]) {
                        Ok(_) => println!("Removed old backup: {}", filename),
                        Err(e) => eprintln!("Error removing backup {}: {}", filename, e),
                    }
                } else {
                    eprintln!("Error getting filename for backup: {:?}", backups[i]);
                }
            }
        }

        Ok(())
    }

    pub fn load() -> Result<Self, DatabaseError> {
        // Perform backup before attempting to load
        if let Err(e) = Self::backup_database() {
            eprintln!("Warning: Failed to create database backup: {}", e);
            // Continue loading even if backup fails
        }

        match File::open(DB_FILENAME) {
            Ok(file) => {
                serde_json::from_reader(file).map_err(DatabaseError::from)
            },
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                println!("Database file not found, creating new one.");
                Ok(Database::new())
            }
            Err(e) => Err(DatabaseError::from(e)),
        }
    }

    pub fn save(&self) -> Result<(), DatabaseError> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(DB_FILENAME)?;
        serde_json::to_writer_pretty(file, self)?;
        Ok(())
    }

    // Removed add_customer_cli

    pub fn add_customer_gui(&mut self, customer: Customer) -> Result<(), DatabaseError> {
        if customer.name.trim().is_empty() {
            return Err(DatabaseError::InvalidInput("Customer name cannot be empty.".to_string()));
        }
        if self.customers.contains_key(customer.name.trim()) {
            return Err(DatabaseError::CustomerExists(customer.name.trim().to_string()));
        }
        let code = customer.code.trim().to_uppercase();
        if !(code.len() >= 2 && code.len() <= 3 && code.chars().all(|c| c.is_ascii_alphabetic())) {
             return Err(DatabaseError::InvalidInput("Customer code must be 2-3 alphabetic characters.".to_string()));
        }
        if self.customers.values().any(|c| c.code == code) {
             return Err(DatabaseError::InvalidInput(format!("Customer code \"{}\" is already in use.", code)));
        }

        let mut validated_customer = customer;
        validated_customer.name = validated_customer.name.trim().to_string();
        validated_customer.code = code;
        self.customers.insert(validated_customer.name.clone(), validated_customer.clone());
        self.last_invoice_nums.entry(validated_customer.code.clone()).or_insert(75);

        self.save()?;

        Ok(())
    }

    // Removed edit_customer_cli

    pub fn edit_customer_gui(&mut self, original_name: &str, updated_customer: Customer) -> Result<(), DatabaseError> {
        if updated_customer.name.trim().is_empty() {
            return Err(DatabaseError::InvalidInput("Customer name cannot be empty.".to_string()));
        }
        let new_name = updated_customer.name.trim().to_string();
        let new_code = updated_customer.code.trim().to_uppercase();

        if !(new_code.len() >= 2 && new_code.len() <= 3 && new_code.chars().all(|c| c.is_ascii_alphabetic())) {
             return Err(DatabaseError::InvalidInput("Customer code must be 2-3 alphabetic characters.".to_string()));
        }

        let original_customer = match self.customers.get(original_name) {
            Some(c) => c.clone(),
            None => return Err(DatabaseError::CustomerNotFound(original_name.to_string())),
        };

        if original_name != new_name && self.customers.contains_key(&new_name) {
            return Err(DatabaseError::CustomerExists(new_name));
        }

        if original_customer.code != new_code {
            if self.customers.values().any(|c| c.name != original_name && c.code == new_code) {
                return Err(DatabaseError::InvalidInput(format!("Customer code \"{}\" is already in use by another customer.", new_code)));
            }
        }

        let mut final_customer = updated_customer;
        final_customer.name = new_name;
        final_customer.code = new_code;

        self.customers.remove(original_name);
        self.customers.insert(final_customer.name.clone(), final_customer.clone());

        if original_customer.code != final_customer.code {
            let last_num = self.last_invoice_nums.remove(&original_customer.code).unwrap_or(75);
            self.last_invoice_nums.insert(final_customer.code.clone(), last_num);
        }
        
        self.save()?;

        Ok(())
    }

    // Removed remove_customer_cli

    // Updated to delete by code, not name, for consistency with GUI state
    pub fn delete_customer_gui(&mut self, customer_code: &str) -> Result<(), DatabaseError> {
        let customer_name = match self.customers.values().find(|c| c.code == customer_code) {
            Some(c) => c.name.clone(),
            None => return Err(DatabaseError::CustomerNotFound(customer_code.to_string())),
        };

        self.customers.remove(&customer_name);
        self.last_invoice_nums.remove(customer_code);
        
        // Also remove associated invoices
        let invoices_to_remove: Vec<String> = self.invoices.iter()
            .filter(|(_, inv)| inv.customer.code == customer_code)
            .map(|(num, _)| num.clone())
            .collect();
        for inv_num in invoices_to_remove {
            self.invoices.remove(&inv_num);
        }

        self.save()?;

        Ok(())
    }

    // Removed list_customers_cli

    pub fn get_customers_vec(&self) -> Vec<Customer> {
        let mut customers: Vec<Customer> = self.customers.values().cloned().collect();
        customers.sort_by(|a, b| a.name.cmp(&b.name));
        customers
    }

    fn generate_next_invoice_number(&mut self, customer_code: &str) -> String {
        let next_num = self.last_invoice_nums.entry(customer_code.to_string()).or_insert(75);
        *next_num += 1;
        format!("{}{}", customer_code, next_num)
    }

    // Removed create_invoice_cli

    pub fn create_invoice_gui(&mut self, customer_code: String, items: Vec<InvoiceItem>, notes: String, due_date_naive: NaiveDate) -> Result<Invoice, DatabaseError> {
        let customer = match self.customers.values().find(|c| c.code == customer_code) {
            Some(c) => c.clone(),
            None => return Err(DatabaseError::CustomerNotFound(customer_code)),
        };

        if items.is_empty() {
            return Err(DatabaseError::InvalidInput("Invoice must have at least one item.".to_string()));
        }

        let invoice_number = self.generate_next_invoice_number(&customer_code);
        // Use DateTime<Local> for date
        let date: DateTime<Local> = Local::now();
        // Convert NaiveDate to DateTime<Local> (assuming midnight)
        let due_date: DateTime<Local> = match due_date_naive.and_hms_opt(0, 0, 0) {
            Some(naive_dt) => Local.from_local_datetime(&naive_dt).single()
                                .ok_or_else(|| DatabaseError::InvalidInput("Invalid due date conversion.".to_string()))?,
            None => return Err(DatabaseError::InvalidInput("Invalid due date provided.".to_string())),
        };

        let mut calculated_items = Vec::new();
        let mut subtotal = 0.0;

        for item in items {
            let amount = item.quantity as f64 * item.rate;
            subtotal += amount;
            calculated_items.push(InvoiceItem {
                description: item.description,
                quantity: item.quantity,
                rate: item.rate,
                amount,
            });
        }

        let total = subtotal; // Assuming no tax for now

        let invoice = Invoice {
            invoice_number: invoice_number.clone(),
            customer,
            date, // Use DateTime<Local>
            due_date, // Use DateTime<Local>
            items: calculated_items,
            notes,
            subtotal,
            total,
            paid: false,
        };

        self.invoices.insert(invoice_number.clone(), invoice.clone());
        self.save()?;

        Ok(invoice)
    }

    // Added function to edit an existing invoice
    pub fn edit_invoice_gui(&mut self, invoice_number: &str, items: Vec<InvoiceItem>, notes: String, due_date_naive: NaiveDate, paid: bool) -> Result<(), DatabaseError> {
        let invoice = match self.invoices.get_mut(invoice_number) {
            Some(inv) => inv,
            None => return Err(DatabaseError::InvoiceNotFound(invoice_number.to_string())),
        };

        if items.is_empty() {
            return Err(DatabaseError::InvalidInput("Invoice must have at least one item.".to_string()));
        }

        // Convert NaiveDate to DateTime<Local> (assuming midnight)
        let due_date: DateTime<Local> = match due_date_naive.and_hms_opt(0, 0, 0) {
            Some(naive_dt) => Local.from_local_datetime(&naive_dt).single()
                                .ok_or_else(|| DatabaseError::InvalidInput("Invalid due date conversion.".to_string()))?,
            None => return Err(DatabaseError::InvalidInput("Invalid due date provided.".to_string())),
        };

        let mut calculated_items = Vec::new();
        let mut subtotal = 0.0;

        for item in items {
            let amount = item.quantity as f64 * item.rate;
            subtotal += amount;
            calculated_items.push(InvoiceItem {
                description: item.description,
                quantity: item.quantity,
                rate: item.rate,
                amount,
            });
        }

        let total = subtotal; // Assuming no tax for now

        invoice.items = calculated_items;
        invoice.notes = notes;
        invoice.due_date = due_date; // Use DateTime<Local>
        invoice.paid = paid;
        invoice.subtotal = subtotal;
        invoice.total = total;
        // invoice.date remains the original issue date

        self.save()?;

        Ok(())
    }

    // Added function to delete an invoice
    pub fn delete_invoice_gui(&mut self, invoice_number: &str) -> Result<(), DatabaseError> {
        if self.invoices.remove(invoice_number).is_none() {
            return Err(DatabaseError::InvoiceNotFound(invoice_number.to_string()));
        }
        self.save()?;
        Ok(())
    }

    // Removed view_invoice_cli
    // Removed mark_invoice_paid_cli
    // Removed list_invoices_cli

    pub fn mark_invoice_paid_gui(&mut self, invoice_number: &str) -> Result<(), DatabaseError> {
        match self.invoices.get_mut(invoice_number) {
            Some(invoice) => {
                invoice.paid = true;
                self.save()?;
                Ok(())
            }
            None => Err(DatabaseError::InvoiceNotFound(invoice_number.to_string())),
        }
    }

    pub fn get_invoices_for_customer(&self, customer_code: &str) -> Vec<Invoice> {
        let mut invoices: Vec<Invoice> = self.invoices.values()
            .filter(|inv| inv.customer.code == customer_code)
            .cloned()
            .collect();
        // Sort by date descending (DateTime<Local> comparison works)
        invoices.sort_by(|a, b| b.date.cmp(&a.date)); 
        invoices
    }

    // Removed generate_pdf_cli

    pub fn generate_pdf_gui(&self, invoice_number: &str, filename: &str) -> Result<String, DatabaseError> {
        match self.invoices.get(invoice_number) {
            Some(invoice) => {
                // Pass individual company details
                generate_pdf(
                    invoice,
                    &self.company.name,
                    &self.company.abn,
                    &self.company.address,
                    &self.company.phone,
                    filename
                )?;
                Ok(filename.to_string())
            }
            None => Err(DatabaseError::InvoiceNotFound(invoice_number.to_string())),
        }
    }
}

