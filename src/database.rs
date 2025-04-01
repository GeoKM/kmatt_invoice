use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use serde::{Serialize, Deserialize};
use chrono::{Local, TimeZone};
use crate::models::{Company, Customer, InvoiceItem, Invoice};
use crate::utils::*;
use crate::pdf_generator::generate_pdf;

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

    pub fn load() -> Self {
        match File::open("database.json") {
            Ok(file) => serde_json::from_reader(file).unwrap_or_else(|_| Database::new()),
            Err(_) => Database::new(),
        }
    }

    pub fn save(&self) {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("database.json")
            .unwrap();
        serde_json::to_writer_pretty(file, self).unwrap();
    }

    pub fn add_customer(&mut self) {
        println!("Enter customer name (non-empty, enter each line, press Enter twice to finish):");
        let name = read_multi_line("Name cannot be empty!");
        if self.customers.contains_key(&name) {
            println!("Customer already exists!");
            return;
        }
        println!("Enter address (non-empty):");
        let address = read_non_empty_string("Address cannot be empty!");
        println!("Enter phone (non-empty):");
        let phone = read_non_empty_string("Phone cannot be empty!");
        println!("Enter contact person (non-empty, enter each line, press Enter twice to finish):");
        let contact_person = read_multi_line("Contact person cannot be empty!");
        println!("Enter contact phone (non-empty):");
        let contact_phone = read_non_empty_string("Contact phone cannot be empty!");
        println!("Enter email (non-empty, enter each line, press Enter twice to finish):");
        let email = read_multi_line("Email cannot be empty!");
        println!("Enter customer code (2-3 letters):");
        let code = read_customer_code();

        let customer = Customer { name: name.clone(), address, phone, contact_person, contact_phone, email, code };
        self.customers.insert(name, customer.clone());
        self.last_invoice_nums.insert(customer.code, 75);
        self.save();
        println!("Customer added successfully!");
    }

    pub fn edit_customer(&mut self) {
        println!("Available customers:");
        if self.customers.is_empty() {
            println!("No customers found. Please add a customer first!");
            return;
        }

        let customer_vec: Vec<(String, Customer)> = self.customers.iter()
            .map(|(name, customer)| (name.clone(), customer.clone()))
            .collect();
        for (i, (name, customer)) in customer_vec.iter().enumerate() {
            println!("{}. {} (Code: {})", i + 1, name, customer.code);
        }

        println!("Enter the line number of the customer to edit:");
        let selection: usize = loop {
            match read_line().parse::<usize>() {
                Ok(n) if n > 0 && n <= customer_vec.len() => break n - 1,
                _ => println!("Please enter a valid number between 1 and {}!", customer_vec.len()),
            }
        };

        if let Some((name, customer)) = customer_vec.get(selection) {
            println!("Editing {}. Leave blank to keep current value.", name);
            println!("Current name:\n{}", customer.name);
            let new_name = read_optional_multi_line(&customer.name);
            println!("Current address: {}", customer.address);
            let address = read_optional_non_empty(&customer.address, "Address cannot be empty!");
            println!("Current phone: {}", customer.phone);
            let phone = read_optional_non_empty(&customer.phone, "Phone cannot be empty!");
            println!("Current contact person:\n{}", customer.contact_person);
            let contact_person = read_optional_multi_line(&customer.contact_person);
            println!("Current contact phone: {}", customer.contact_phone);
            let contact_phone = read_optional_non_empty(&customer.contact_phone, "Contact phone cannot be empty!");
            println!("Current email:\n{}", customer.email);
            let email = read_optional_multi_line(&customer.email);
            println!("Current code: {}", customer.code);
            let code = read_optional_customer_code(&customer.code);

            let updated_customer = Customer { name: new_name.clone(), address, phone, contact_person, contact_phone, email, code };
            self.customers.remove(name);
            self.customers.insert(new_name.clone(), updated_customer.clone());
            self.last_invoice_nums.insert(updated_customer.code, *self.last_invoice_nums.get(&customer.code).unwrap_or(&75));
            self.save();
            println!("Customer updated successfully!");
        } else {
            println!("Customer not found!");
        }
    }

    pub fn remove_customer(&mut self) {
        println!("Available customers:");
        if self.customers.is_empty() {
            println!("No customers found. Please add a customer first!");
            return;
        }

        let customer_vec: Vec<(String, Customer)> = self.customers.iter()
            .map(|(name, customer)| (name.clone(), customer.clone()))
            .collect();
        for (i, (name, customer)) in customer_vec.iter().enumerate() {
            println!("{}. {} (Code: {})", i + 1, name, customer.code);
        }

        println!("Enter the line number of the customer to remove:");
        let selection: usize = loop {
            match read_line().parse::<usize>() {
                Ok(n) if n > 0 && n <= customer_vec.len() => break n - 1,
                _ => println!("Please enter a valid number between 1 and {}!", customer_vec.len()),
            }
        };

        if let Some((name, customer)) = customer_vec.get(selection) {
            self.customers.remove(name);
            self.last_invoice_nums.remove(&customer.code);
            self.save();
            println!("Customer removed successfully!");
        } else {
            println!("Customer not found!");
        }
    }

    pub fn list_customers(&self) {
        println!("Customers:");
        if self.customers.is_empty() {
            println!("No customers found.");
        } else {
            for (i, (name, customer)) in self.customers.iter().enumerate() {
                println!("{}. {} (Code: {})", i + 1, name, customer.code);
            }
        }
    }

    pub fn generate_invoice_number(&mut self, customer_code: &str) -> String {
        let num = self.last_invoice_nums.entry(customer_code.to_string())
            .and_modify(|n| *n += 1)
            .or_insert(75);
        format!("{}{:03}", customer_code, num)
    }

    pub fn create_invoice(&mut self) {
        println!("Available customers:");
        if self.customers.is_empty() {
            println!("No customers found. Please add a customer first!");
            return;
        }
        
        let customer_vec: Vec<(String, Customer)> = self.customers.iter()
            .map(|(name, customer)| (name.clone(), customer.clone()))
            .collect();
        for (i, (name, customer)) in customer_vec.iter().enumerate() {
            println!("{}. {} (Code: {})", i + 1, name, customer.code);
        }

        println!("Enter the number of the customer:");
        let selection: usize = loop {
            match read_line().parse::<usize>() {
                Ok(n) if n > 0 && n <= customer_vec.len() => break n - 1,
                _ => println!("Please enter a valid number between 1 and {}!", customer_vec.len()),
            }
        };

        let customer = customer_vec[selection].1.clone();
        let mut items = Vec::new();
        loop {
            println!("Add an item? (y/n)");
            if read_yes_no() != "y" {
                break;
            }
            println!("Enter description (non-empty):");
            let description = read_non_empty_string("Description cannot be empty!");
            println!("Enter quantity (positive integer):");
            let quantity = read_positive_u32();
            println!("Enter rate (positive number):");
            let rate = read_positive_f64();
            let amount = quantity as f64 * rate;
            items.push(InvoiceItem { description, quantity, rate, amount });
        }
        println!("Enter notes (press Enter for none):");
        let notes = read_line();

        let now = Local::now();
        println!("Enter due date (YYYY-MM-DD, must be after {}):", now.format("%Y-%m-%d"));
        let due_date = read_date_after(now.date_naive());
        let due_date = Local.from_local_datetime(&due_date.and_hms_opt(0, 0, 0).unwrap()).unwrap();

        let subtotal = items.iter().map(|item| item.amount).sum();
        let total = subtotal;

        let invoice_number = self.generate_invoice_number(&customer.code);
        let invoice = Invoice {
            invoice_number: invoice_number.clone(),
            date: now,
            due_date,
            customer,
            items,
            subtotal,
            total,
            notes,
            paid: false,
        };

        self.invoices.insert(invoice_number.clone(), invoice);
        self.save();
        println!("Invoice {} created!", invoice_number);
    }

    pub fn list_invoices(&self) {
        println!("Invoices:");
        if self.invoices.is_empty() {
            println!("No invoices found.");
        } else {
            for (num, invoice) in &self.invoices {
                println!("{} - {} - ${:.2} - {}", 
                    num, 
                    invoice.customer.name, 
                    invoice.total, 
                    if invoice.paid { "PAID" } else { "UNPAID" }
                );
            }
        }
    }

    pub fn view_invoice(&self) {
        println!("Enter invoice number to view:");
        let num = read_non_empty_string("Invoice number cannot be empty!");
        if let Some(invoice) = self.invoices.get(&num) {
            println!("{}", self.format_invoice_text(invoice));
        } else {
            println!("Invoice not found!");
        }
    }

    pub fn format_invoice_text(&self, invoice: &Invoice) -> String {
        let mut output = String::new();
        output.push_str(&format!("{}\n", self.company.name));
        output.push_str(&format!("A.B.N. {}\n", self.company.abn));
        output.push_str(&format!("{}\nPh. {}\n", self.company.address, self.company.phone));
        output.push_str(&format!("Invoice # {}\n", invoice.invoice_number));
        output.push_str(&format!("Date: {}\n\n", invoice.date.format("%b %-d, %Y")));
        output.push_str("\x1B[1mBill To:\x1B[0m\n"); // ANSI bold for console
        for line in invoice.customer.name.split('\n') {
            output.push_str(&format!("{}\n", line));
        }
        output.push_str(&format!("{}\n", invoice.customer.address));
        output.push_str(&format!("Phone | {}\n", invoice.customer.phone));
        output.push_str("Attn - ");
        let contact_lines: Vec<&str> = invoice.customer.contact_person.split('\n').collect();
        if !contact_lines.is_empty() {
            output.push_str(&format!("{}", contact_lines[0]));
        }
        output.push_str(&format!(" ({}), ", invoice.customer.contact_phone));
        let email_lines: Vec<&str> = invoice.customer.email.split('\n').collect();
        if !email_lines.is_empty() {
            output.push_str(&format!("{}", email_lines[0]));
        }
        output.push_str("\n");
        for i in 1..contact_lines.len() {
            output.push_str(&format!("       {}\n", contact_lines[i]));
        }
        for i in 1..email_lines.len() {
            output.push_str(&format!("       {}\n", email_lines[i]));
        }
        output.push_str(&format!("Payment Terms: Net 30 Days\n"));
        output.push_str(&format!("Due Date: {}\n", invoice.due_date.format("%b %-d, %Y")));
        output.push_str(&format!("Balance Due: AU ${:.2}\n\n", invoice.total));
        output.push_str("#\tItem\t\t\tQty\tRate\tAmount\n");
        for (idx, item) in invoice.items.iter().enumerate() {
            output.push_str(&format!(
                "{:<3}\t{:<30}\t{:>2}\tAU ${:>6.2}\tAU ${:>6.2}\n",
                idx + 1, item.description, item.quantity, item.rate, item.amount
            ));
        }
        output.push_str(&format!("\n{:>30} AU ${:.2}\n", "Subtotal:", invoice.subtotal));
        output.push_str(&format!("{:>30} AU $0.00\n", "Tax (0%):"));
        output.push_str(&format!("Notes:\n{}\n\n", invoice.notes));
        output.push_str("Please Pay to by bank transfer to our bank account Commonwealth Bank Tuggeranong.\n");
        output.push_str("Account Name - James Matthews\n");
        output.push_str("BSB - 062692\n");
        output.push_str("Acct Number - 33455315\n\n");
        output.push_str("Terms:\nStrictly 30 Days Net Full Payment Please\n");
        output
    }

    pub fn mark_paid(&mut self) {
        println!("Enter invoice number to mark as paid:");
        let num = read_non_empty_string("Invoice number cannot be empty!");
        if let Some(invoice) = self.invoices.get_mut(&num) {
            invoice.paid = true;
            self.save();
            println!("Invoice {} marked as paid!", num);
        } else {
            println!("Invoice not found!");
        }
    }

    pub fn delete_invoice(&mut self) {
        println!("Enter invoice number to delete:");
        let num = read_non_empty_string("Invoice number cannot be empty!");
        if self.invoices.remove(&num).is_some() {
            self.save();
            println!("Invoice {} deleted successfully!", num);
        } else {
            println!("Invoice not found!");
        }
    }

    pub fn generate_pdf(&self) {
        println!("Enter invoice number to generate PDF:");
        let num = read_non_empty_string("Invoice number cannot be empty!");
        if let Some(invoice) = self.invoices.get(&num) {
            generate_pdf(
                invoice,
                &self.company.name,
                &self.company.abn,
                &self.company.address,
                &self.company.phone,
            );
        } else {
            println!("Invoice not found!");
        }
    }
}
