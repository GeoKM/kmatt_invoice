use std::collections::HashMap;
use std::io::{self, BufWriter};
use chrono::{DateTime, Local, NaiveDate, TimeZone};
use printpdf::*;
use std::fs::{File, OpenOptions};
use serde::{Serialize, Deserialize};
use std::io::Write;
use prettytable::{Table, Row, Cell, format};

#[derive(Clone, Serialize, Deserialize)]
struct Company {
    name: String,
    abn: String,
    address: String,
    phone: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct Customer {
    name: String,
    address: String,
    phone: String,
    contact_person: String,
    contact_phone: String,
    email: String,
    code: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct InvoiceItem {
    description: String,
    quantity: u32,
    rate: f64,
    amount: f64,
}

#[derive(Clone, Serialize, Deserialize)]
struct Invoice {
    invoice_number: String,
    date: DateTime<Local>,
    due_date: DateTime<Local>,
    customer: Customer,
    items: Vec<InvoiceItem>,
    subtotal: f64,
    total: f64,
    notes: String,
    paid: bool,
}

#[derive(Serialize, Deserialize)]
struct Database {
    company: Company,
    customers: HashMap<String, Customer>,
    invoices: HashMap<String, Invoice>,
    last_invoice_nums: HashMap<String, u32>,
}

impl Database {
    fn new() -> Self {
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

    fn load() -> Self {
        match File::open("database.json") {
            Ok(file) => serde_json::from_reader(file).unwrap_or_else(|_| Database::new()),
            Err(_) => Database::new(),
        }
    }

    fn save(&self) {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("database.json")
            .unwrap();
        serde_json::to_writer_pretty(file, self).unwrap();
    }

    fn add_customer(&mut self) {
        println!("Enter customer name (non-empty):");
        let name = read_non_empty_string("Name cannot be empty!");
        if self.customers.contains_key(&name) {
            println!("Customer already exists!");
            return;
        }
        println!("Enter address (non-empty):");
        let address = read_non_empty_string("Address cannot be empty!");
        println!("Enter phone (non-empty):");
        let phone = read_non_empty_string("Phone cannot be empty!");
        println!("Enter contact person (non-empty):");
        let contact_person = read_non_empty_string("Contact person cannot be empty!");
        println!("Enter contact phone (non-empty):");
        let contact_phone = read_non_empty_string("Contact phone cannot be empty!");
        println!("Enter email (non-empty):");
        let email = read_non_empty_string("Email cannot be empty!");
        println!("Enter customer code (2-3 letters):");
        let code = read_customer_code();

        let customer = Customer { name: name.clone(), address, phone, contact_person, contact_phone, email, code };
        self.customers.insert(name, customer.clone());
        self.last_invoice_nums.insert(customer.code, 75);
        self.save();
        println!("Customer added successfully!");
    }

    fn edit_customer(&mut self) {
        println!("Enter customer name to edit:");
        let name = read_non_empty_string("Name cannot be empty!");
        if let Some(customer) = self.customers.get_mut(&name) {
            println!("Editing {}. Leave blank to keep current value.", name);
            println!("Current address: {}", customer.address);
            let address = read_optional_non_empty(&customer.address, "Address cannot be empty!");
            println!("Current phone: {}", customer.phone);
            let phone = read_optional_non_empty(&customer.phone, "Phone cannot be empty!");
            println!("Current contact person: {}", customer.contact_person);
            let contact_person = read_optional_non_empty(&customer.contact_person, "Contact person cannot be empty!");
            println!("Current contact phone: {}", customer.contact_phone);
            let contact_phone = read_optional_non_empty(&customer.contact_phone, "Contact phone cannot be empty!");
            println!("Current email: {}", customer.email);
            let email = read_optional_non_empty(&customer.email, "Email cannot be empty!");
            println!("Current code: {}", customer.code);
            let code = read_optional_customer_code(&customer.code);

            *customer = Customer { name: name.clone(), address, phone, contact_person, contact_phone, email, code };
            self.save();
            println!("Customer updated successfully!");
        } else {
            println!("Customer not found!");
        }
    }

    fn remove_customer(&mut self) {
        println!("Enter customer name to remove:");
        let name = read_non_empty_string("Name cannot be empty!");
        if let Some(customer) = self.customers.remove(&name) {
            self.last_invoice_nums.remove(&customer.code);
            self.save();
            println!("Customer removed successfully!");
        } else {
            println!("Customer not found!");
        }
    }

    fn list_customers(&self) {
        println!("Customers:");
        if self.customers.is_empty() {
            println!("No customers found.");
        } else {
            for (i, (name, customer)) in self.customers.iter().enumerate() {
                println!("{}. {} (Code: {})", i + 1, name, customer.code);
            }
        }
    }

    fn generate_invoice_number(&mut self, customer_code: &str) -> String {
        let num = self.last_invoice_nums.entry(customer_code.to_string())
            .and_modify(|n| *n += 1)
            .or_insert(75);
        format!("{}{:03}", customer_code, num)
    }

    fn create_invoice(&mut self) {
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
        let due_date = loop {
            match read_line().parse::<NaiveDate>() {
                Ok(date) => {
                    if date > now.date_naive() {
                        break Local.from_local_datetime(&date.and_hms_opt(0, 0, 0).unwrap()).unwrap();
                    } else {
                        println!("Due date must be after the current date. Please try again.");
                    }
                }
                Err(_) => println!("Invalid date format. Use YYYY-MM-DD and try again."),
            }
        };

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

    fn list_invoices(&self) {
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

    fn view_invoice(&self) {
        println!("Enter invoice number to view:");
        let num = read_non_empty_string("Invoice number cannot be empty!");
        if let Some(invoice) = self.invoices.get(&num) {
            println!("{}", self.format_invoice_text(invoice));
        } else {
            println!("Invoice not found!");
        }
    }

    fn format_invoice_text(&self, invoice: &Invoice) -> String {
        let mut output = String::new();
        output.push_str(&format!("{}\n", self.company.name));
        output.push_str(&format!("A.B.N. {}\n", self.company.abn));
        output.push_str(&format!("{}\nPh. {}\n", self.company.address, self.company.phone));
        output.push_str(&format!("Invoice # {}\n", invoice.invoice_number));
        output.push_str(&format!("Date: {}\n\n", invoice.date.format("%b %-d, %Y")));
        output.push_str("\x1B[1mBill To:\x1B[0m\n"); // ANSI bold for console
        output.push_str(&format!("{}\n", invoice.customer.name));
        output.push_str(&format!("{}\n", invoice.customer.address));
        output.push_str(&format!("Phone | {}\n", invoice.customer.phone));
        output.push_str(&format!("Attn - {} ({}), {}\n", 
            invoice.customer.contact_person,
            invoice.customer.contact_phone,
            invoice.customer.email
        ));
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
        output.push_str(&format!("{:>30} AU ${:.2}\n\n", "Total:", invoice.total));
        output.push_str(&format!("Notes:\n{}\n\n", invoice.notes));
        output.push_str("Please Pay to by bank transfer to our bank account Commonwealth Bank Tuggeranong.\n");
        output.push_str("Account Name - James Matthews\n");
        output.push_str("BSB - 062692\n");
        output.push_str("Acct Number - 33455315\n\n");
        output.push_str("Terms:\nStrictly 30 Days Net Full Payment Please\n");
        output
    }

    fn mark_paid(&mut self) {
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

    fn delete_invoice(&mut self) {
        println!("Enter invoice number to delete:");
        let num = read_non_empty_string("Invoice number cannot be empty!");
        if self.invoices.remove(&num).is_some() {
            self.save();
            println!("Invoice {} deleted successfully!", num);
        } else {
            println!("Invoice not found!");
        }
    }

    fn generate_pdf(&self) {
        println!("Enter invoice number to generate PDF:");
        let num = read_non_empty_string("Invoice number cannot be empty!");
        if let Some(invoice) = self.invoices.get(&num) {
            // Initialize PDF document (A4 size: 210mm x 297mm)
            let (doc, page1, layer1) = PdfDocument::new(
                format!("Invoice #{}", invoice.invoice_number),
                Mm(210.0),
                Mm(297.0),
                "Layer 1",
            );
            let layer = doc.get_page(page1).get_layer(layer1);
            let helvetica_font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap(); // For general text
            let courier_font = doc.add_builtin_font(BuiltinFont::Courier).unwrap(); // For table (monospaced)
            let font_size = 10.0;
            let line_height = 4.23; // ~12pt for 10pt font (1pt = 0.3527mm)
            let mut y_pos = 280.0; // Start near top of page

            // Helper to add text at specific positions with specified font
            let add_text = |layer: &PdfLayerReference, text: &str, x: Mm, y: f32, font: &IndirectFontRef| {
                layer.use_text(text, font_size, x, Mm(y), font);
            };

            // Company Header (Helvetica)
            add_text(&layer, "JMATTS CLEANING Canberra", Mm(15.0), y_pos, &helvetica_font);
            y_pos -= line_height;
            add_text(&layer, "A.B.N. 78734213681", Mm(15.0), y_pos, &helvetica_font);
            y_pos -= line_height;
            add_text(&layer, "40 Wyndham Avenue, Denman Prospect, ACT, 2611", Mm(15.0), y_pos, &helvetica_font);
            y_pos -= line_height;
            add_text(&layer, "Ph: 0403-491446", Mm(15.0), y_pos, &helvetica_font);
            y_pos -= line_height;
            add_text(&layer, &format!("Invoice #{}", invoice.invoice_number), Mm(15.0), y_pos, &helvetica_font);
            y_pos -= line_height;
            add_text(&layer, &format!("Date: {}", invoice.date.format("%b %d, %Y")), Mm(15.0), y_pos, &helvetica_font);
            y_pos -= 2.0 * line_height; // Extra spacing

            // Bill To and Payment Terms on same lines, left-justified and right-justified respectively
            let bill_to_y = y_pos; // Store y-position for Bill To
            // Bill To (Helvetica, left-justified at 15mm)
            add_text(&layer, "Bill To:", Mm(15.0), bill_to_y, &helvetica_font);
            y_pos -= line_height;
            add_text(&layer, &invoice.customer.name, Mm(15.0), y_pos, &helvetica_font);
            y_pos -= line_height;
            add_text(&layer, &invoice.customer.address, Mm(15.0), y_pos, &helvetica_font);
            y_pos -= line_height;
            add_text(&layer, &format!("Phone: {}", invoice.customer.phone), Mm(15.0), y_pos, &helvetica_font);

            // Reset y_pos to align Payment Terms with Bill To
            y_pos = bill_to_y;
            // Payment Terms (Helvetica, right-justified at 150mm)
            add_text(&layer, "Payment Terms: Net 30 Days", Mm(150.0), y_pos, &helvetica_font);
            y_pos -= line_height;
            add_text(&layer, &format!("Due Date: {}", invoice.due_date.format("%b %d, %Y")), Mm(150.0), y_pos, &helvetica_font);
            y_pos -= line_height;
            add_text(&layer, &format!("Balance Due: AU ${:.2}", invoice.total), Mm(150.0), y_pos, &helvetica_font);

            // Display Current Date above Payment Terms
            y_pos -= 2.0 * line_height; // Extra spacing before current date
            add_text(&layer, &format!("(Current Date: {})", invoice.date.format("%b %d, %Y")), Mm(15.0), y_pos, &helvetica_font);
            y_pos -= 2.0 * line_height; // Extra spacing after current date

            // Create the table using prettytable-rs
            let mut table = Table::new();
            table.set_format(*format::consts::FORMAT_CLEAN); // Clean format without borders for simplicity

            // Set column titles
            table.set_titles(Row::new(vec![
                Cell::new("#"),
                Cell::new("Item"),
                Cell::new("Qty"),
                Cell::new("Rate"),
                Cell::new("Amount"),
            ]));

            // Add table rows
            for (idx, item) in invoice.items.iter().enumerate() {
                let line_num = idx + 1;
                let description_lines = wrap_text(&item.description, 50); // Wrap at 50 characters
                for (i, line) in description_lines.iter().enumerate() {
                    if i == 0 {
                        // First line: include all columns
                        table.add_row(Row::new(vec![
                            Cell::new(&format!("{:>3}", line_num)),
                            Cell::new(line),
                            Cell::new(&format!("{:>6}", item.quantity)),
                            Cell::new(&format!("AU ${:>6.2}", item.rate)),
                            Cell::new(&format!("AU ${:>6.2}", item.amount)),
                        ]));
                    } else {
                        // Subsequent lines: only Item description
                        table.add_row(Row::new(vec![
                            Cell::new(""),
                            Cell::new(line),
                            Cell::new(""),
                            Cell::new(""),
                            Cell::new(""),
                        ]));
                    }
                }
            }

            // Convert table to string
            let table_string = table.to_string();
            let table_lines: Vec<&str> = table_string.lines().collect();

            // Render table lines into PDF
            for line in table_lines {
                add_text(&layer, line, Mm(15.0), y_pos, &courier_font);
                y_pos -= line_height;
            }

            // Total (Helvetica for "Total:", Courier for amount, aligned with "Amount" column)
            y_pos -= 3.0 * line_height; // Increased spacing to ensure a fresh line (3x line_height)
            let total_label_x = Mm(15.0 + (69.0 * 1.0)); // 69 characters to start of "Amount" column
            let total_amount_x = Mm(15.0 + (75.0 * 1.0)); // 75 characters to right edge of "Amount" column
            add_text(&layer, "Total:        ", total_label_x, y_pos, &helvetica_font); // 8 spaces after "Total:"
            add_text(&layer, &format!("${:>6.2}", invoice.total), total_amount_x, y_pos, &courier_font); // Removed "AU ", right-justified at "Amount" column edge

            y_pos -= 2.0 * line_height;

            // Notes (Helvetica)
            add_text(&layer, "Notes:", Mm(15.0), y_pos, &helvetica_font);
            y_pos -= line_height;
            add_text(&layer, &invoice.notes, Mm(15.0), y_pos, &helvetica_font);
            y_pos -= 2.0 * line_height;

            // Payment Instructions (Helvetica)
            add_text(&layer, "Please Pay to by bank transfer to our bank account Commonwealth Bank Tuggeranong.", Mm(15.0), y_pos, &helvetica_font);
            y_pos -= line_height;
            add_text(&layer, "Account Name - James Matthews", Mm(15.0), y_pos, &helvetica_font);
            y_pos -= line_height;
            add_text(&layer, "BSB - 062692", Mm(15.0), y_pos, &helvetica_font);
            y_pos -= line_height;
            add_text(&layer, "Acct Number - 33455315", Mm(15.0), y_pos, &helvetica_font);
            y_pos -= line_height;
            add_text(&layer, "Terms:", Mm(15.0), y_pos, &helvetica_font);
            y_pos -= line_height;
            add_text(&layer, "Strictly 30 Days Net Full Payment Please", Mm(15.0), y_pos, &helvetica_font);

            // Save PDF
            let file = File::create(format!("invoice_{}.pdf", invoice.invoice_number)).unwrap();
            doc.save(&mut BufWriter::new(file)).unwrap();
            println!("PDF generated: invoice_{}.pdf", invoice.invoice_number);
        } else {
            println!("Invoice not found!");
        }
    }
}

// Helper function to wrap text within a column width
fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut lines = Vec::new();
    let mut current_line = String::new();
    for word in words {
        if current_line.len() + word.len() + 1 > max_chars {
            if !current_line.is_empty() {
                lines.push(current_line.clone());
                current_line.clear();
            }
            if word.len() > max_chars {
                // Split long words
                let mut start = 0;
                while start < word.len() {
                    let end = (start + max_chars).min(word.len());
                    let part = &word[start..end];
                    lines.push(part.to_string());
                    start = end;
                }
            } else {
                current_line = word.to_string();
            }
        } else {
            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(word);
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    lines
}

fn read_line() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn read_non_empty_string(error_msg: &str) -> String {
    loop {
        let input = read_line();
        if !input.is_empty() {
            return input;
        }
        println!("{}", error_msg);
    }
}

fn read_optional_non_empty(default: &str, error_msg: &str) -> String {
    let input = read_line();
    if input.is_empty() { 
        default.to_string() 
    } else if !input.trim().is_empty() { 
        input 
    } else {
        println!("{}", error_msg);
        default.to_string()
    }
}

fn read_customer_code() -> String {
    loop {
        let input = read_line().to_uppercase();
        if input.len() >= 2 && input.len() <= 3 && input.chars().all(|c| c.is_ascii_alphabetic()) {
            return input;
        }
        println!("Code must be 2-3 letters!");
    }
}

fn read_optional_customer_code(default: &str) -> String {
    let input = read_line().to_uppercase();
    if input.is_empty() {
        default.to_string()
    } else if input.len() >= 2 && input.len() <= 3 && input.chars().all(|c| c.is_ascii_alphabetic()) {
        input
    } else {
        println!("Code must be 2-3 letters! Keeping default.");
        default.to_string()
    }
}

fn read_yes_no() -> String {
    loop {
        let input = read_line().to_lowercase();
        if input == "y" || input == "n" {
            return input;
        }
        println!("Please enter 'y' or 'n'!");
    }
}

fn read_positive_u32() -> u32 {
    loop {
        match read_line().parse::<u32>() {
            Ok(n) if n > 0 => return n,
            _ => println!("Please enter a positive integer!"),
        }
    }
}

fn read_positive_f64() -> f64 {
    loop {
        match read_line().parse::<f64>() {
            Ok(n) if n > 0.0 => return n,
            _ => println!("Please enter a positive number!"),
        }
    }
}

fn main() {
    let mut db = Database::load();

    loop {
        println!("\nInvoice Management System");
        println!("1. Add Customer");
        println!("2. Edit Customer");
        println!("3. Remove Customer");
        println!("4. List Customers");
        println!("5. Create Invoice");
        println!("6. List Invoices");
        println!("7. View Invoice");
        println!("8. Generate Invoice PDF");
        println!("9. Mark Invoice Paid");
        println!("10. Delete Invoice");
        println!("11. Exit");
        print!("Choose an option: ");
        io::stdout().flush().unwrap();

        match read_line().as_str() {
            "1" => db.add_customer(),
            "2" => db.edit_customer(),
            "3" => db.remove_customer(),
            "4" => db.list_customers(),
            "5" => db.create_invoice(),
            "6" => db.list_invoices(),
            "7" => db.view_invoice(),
            "8" => db.generate_pdf(),
            "9" => db.mark_paid(),
            "10" => db.delete_invoice(),
            "11" => {
                db.save();
                println!("Database saved. Exiting...");
                break;
            },
            _ => println!("Invalid option!"),
        }
    }
}
