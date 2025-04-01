mod database;
mod models;
mod pdf_generator;
mod utils;

use database::Database;
use std::io::{self, Write};

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

        match utils::read_line().as_str() {
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
