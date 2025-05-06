mod database;
mod models;
mod pdf_generator;
mod utils;
mod gui; // Add the gui module

// Removed unused: use database::Database;
// Removed unused: use std::env;
// Removed unused: use std::io::{self, Write};
// Removed unused: use crate::utils; // No longer needed for CLI

fn main() {
    // Removed unused args: let args: Vec<String> = env::args().collect();

    // Default to GUI unless a specific CLI flag is added later (if needed)
    // For now, always launch GUI
    // if args.contains(&"--gui".to_string()) { 
        // Launch GUI using the new run function
        println!("Launching GUI...");
        if let Err(e) = gui::run() { // Changed run_gui() to run()
            eprintln!("Error running GUI: {}", e);
            std::process::exit(1);
        }
    // } else {
        // Removed CLI logic block
    // }
}

// Removed run_cli function entirely

