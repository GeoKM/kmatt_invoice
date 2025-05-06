use crate::database::Database;
use crate::models::{Customer, Invoice, InvoiceItem};
use egui::{CentralPanel, Context, SidePanel, TopBottomPanel, Window, ViewportCommand, TextEdit, Color32, ScrollArea, Grid, RichText, Id};
use chrono::{Local, NaiveDate};
use rfd::FileDialog;
use std::error::Error; // Import Error trait

// Function to run the GUI
pub fn run() -> Result<(), Box<dyn Error>> { // Return Box<dyn Error> for compatibility
    let options = eframe::NativeOptions::default();
    // Use Ok(...) and ? to handle the error type conversion
    Ok(eframe::run_native(
        "JMATT CLEANING Invoice Manager", // Changed window title
        options,
        Box::new(|cc| Ok(Box::new(KmattInvoiceApp::new(cc)))),
    )?)
}

#[derive(Default, Clone)]
pub struct AddCustomerState {
    name: String,
    address: String,
    phone: String,
    contact_person: String,
    contact_phone: String,
    email: String,
    code: String,
    error_message: Option<String>,
}

#[derive(Default, Clone)]
pub struct EditCustomerState {
    original_name: String,
    name: String,
    address: String,
    phone: String,
    contact_person: String,
    contact_phone: String,
    email: String,
    code: String,
    error_message: Option<String>,
}

#[derive(Default, Clone)]
pub struct InvoiceItemState {
    description: String,
    quantity_str: String,
    rate_str: String,
}

#[derive(Clone)]
pub struct CreateInvoiceState {
    customer_code: String,
    customer_name: String,
    items: Vec<InvoiceItemState>,
    notes: String,
    due_date_str: String,
    error_message: Option<String>,
}

impl Default for CreateInvoiceState {
    fn default() -> Self {
        Self {
            customer_code: String::new(),
            customer_name: String::new(),
            items: vec![InvoiceItemState::default()],
            notes: String::new(),
            due_date_str: Local::now().date_naive().format("%Y-%m-%d").to_string(),
            error_message: None,
        }
    }
}

// Added state for editing invoices
#[derive(Clone)]
pub struct EditInvoiceState {
    original_invoice_number: String,
    // Removed unused field: customer_code: String, 
    customer_name: String,
    items: Vec<InvoiceItemState>,
    notes: String,
    due_date_str: String,
    paid: bool, // Allow editing paid status?
    error_message: Option<String>,
}

impl Default for EditInvoiceState {
    fn default() -> Self {
        Self {
            original_invoice_number: String::new(),
            // Removed unused field: customer_code: String::new(),
            customer_name: String::new(),
            items: vec![InvoiceItemState::default()],
            notes: String::new(),
            due_date_str: Local::now().date_naive().format("%Y-%m-%d").to_string(),
            paid: false,
            error_message: None,
        }
    }
}


pub struct KmattInvoiceApp {
    db: Database,
    customers: Vec<Customer>,
    selected_customer_code: Option<String>,
    invoices_for_selected_customer: Vec<Invoice>,
    selected_invoice_number: Option<String>, // Added to track selected invoice
    show_add_customer_window: bool,
    add_customer_state: AddCustomerState,
    show_edit_customer_window: bool,
    edit_customer_state: EditCustomerState,
    show_create_invoice_window: bool,
    create_invoice_state: CreateInvoiceState,
    show_view_invoice_window: bool,
    invoice_to_view: Option<Invoice>,
    show_delete_customer_confirm_window: bool, 
    customer_to_delete_code: Option<String>, 
    show_edit_invoice_window: bool, // Added for edit invoice
    edit_invoice_state: EditInvoiceState, // Added for edit invoice
    show_delete_invoice_confirm_window: bool, // Added for delete invoice confirm
    invoice_to_delete_number: Option<String>, // Added for delete invoice confirm
    status_message: String,
}

impl KmattInvoiceApp {
    pub fn new(_cc: &eframe::CreationContext<
'_>) -> Self {
        let db = match Database::load() {
            Ok(db) => db,
            Err(e) => {
                eprintln!("Failed to load database: {}, creating new.", e);
                Database::new()
            }
        };
        
        let customers = db.get_customers_vec();

        Self {
            db,
            customers,
            selected_customer_code: None,
            invoices_for_selected_customer: Vec::new(),
            selected_invoice_number: None, // Init selected invoice
            show_add_customer_window: false,
            add_customer_state: AddCustomerState::default(),
            show_edit_customer_window: false,
            edit_customer_state: EditCustomerState::default(),
            show_create_invoice_window: false,
            create_invoice_state: CreateInvoiceState::default(),
            show_view_invoice_window: false,
            invoice_to_view: None,
            show_delete_customer_confirm_window: false, 
            customer_to_delete_code: None, 
            show_edit_invoice_window: false, // Init edit invoice state
            edit_invoice_state: EditInvoiceState::default(), // Init edit invoice state
            show_delete_invoice_confirm_window: false, // Init delete invoice confirm state
            invoice_to_delete_number: None, // Init delete invoice confirm state
            status_message: "GUI Initialized.".to_string(),
        }
    }

    fn add_customer_window(&mut self, ctx: &Context) {
        let mut close_window = false;
        Window::new("Add New Customer")
            .id(Id::new("add_customer_window")) // Unique ID
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
            Grid::new("add_customer_grid")
                .num_columns(2)
                .spacing([10.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Name:");
                    ui.add(TextEdit::singleline(&mut self.add_customer_state.name).hint_text("Required"));
                    ui.end_row();
                    ui.label("Address:");
                    ui.text_edit_singleline(&mut self.add_customer_state.address);
                    ui.end_row();
                    ui.label("Phone:");
                    ui.text_edit_singleline(&mut self.add_customer_state.phone);
                    ui.end_row();
                    ui.label("Contact Person:");
                    ui.text_edit_singleline(&mut self.add_customer_state.contact_person);
                    ui.end_row();
                    ui.label("Contact Phone:");
                    ui.text_edit_singleline(&mut self.add_customer_state.contact_phone);
                    ui.end_row();
                    ui.label("Email:");
                    ui.text_edit_singleline(&mut self.add_customer_state.email);
                    ui.end_row();
                    ui.label("Code (2-3 letters):");
                    ui.add(TextEdit::singleline(&mut self.add_customer_state.code).hint_text("Required, e.g., ABC"));
                    ui.end_row();
                });
            ui.separator();
            if let Some(err) = &self.add_customer_state.error_message {
                ui.colored_label(Color32::RED, err);
            }
            ui.horizontal(|ui| {
                if ui.button("Save Customer").clicked() {
                    let new_customer = Customer {
                        name: self.add_customer_state.name.trim().to_string(),
                        address: self.add_customer_state.address.trim().to_string(),
                        phone: self.add_customer_state.phone.trim().to_string(),
                        contact_person: self.add_customer_state.contact_person.trim().to_string(),
                        contact_phone: self.add_customer_state.contact_phone.trim().to_string(),
                        email: self.add_customer_state.email.trim().to_string(),
                        code: self.add_customer_state.code.trim().to_uppercase(),
                    };
                    match self.db.add_customer_gui(new_customer) {
                        Ok(_) => {
                            self.status_message = format!("Customer \"{}\" added successfully.", self.add_customer_state.name.trim());
                            self.update_customer_list();
                            self.add_customer_state = AddCustomerState::default();
                            close_window = true;
                        },
                        Err(e) => {
                            self.add_customer_state.error_message = Some(e.to_string());
                        }
                    }
                }
                if ui.button("Cancel").clicked() {
                    self.add_customer_state = AddCustomerState::default();
                    close_window = true;
                }
            });
        });
        if close_window {
            self.show_add_customer_window = false;
        }
    }

    fn edit_customer_window(&mut self, ctx: &Context) {
        let mut close_window = false;
        Window::new(format!("Edit Customer: {}", self.edit_customer_state.original_name))
            .id(Id::new("edit_customer_window")) // Unique ID
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
            Grid::new("edit_customer_grid")
                .num_columns(2)
                .spacing([10.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Name:");
                    ui.add(TextEdit::singleline(&mut self.edit_customer_state.name).hint_text("Required"));
                    ui.end_row();
                    ui.label("Address:");
                    ui.text_edit_singleline(&mut self.edit_customer_state.address);
                    ui.end_row();
                    ui.label("Phone:");
                    ui.text_edit_singleline(&mut self.edit_customer_state.phone);
                    ui.end_row();
                    ui.label("Contact Person:");
                    ui.text_edit_singleline(&mut self.edit_customer_state.contact_person);
                    ui.end_row();
                    ui.label("Contact Phone:");
                    ui.text_edit_singleline(&mut self.edit_customer_state.contact_phone);
                    ui.end_row();
                    ui.label("Email:");
                    ui.text_edit_singleline(&mut self.edit_customer_state.email);
                    ui.end_row();
                    ui.label("Code (2-3 letters):");
                    ui.add(TextEdit::singleline(&mut self.edit_customer_state.code).hint_text("Required, e.g., ABC"));
                    ui.end_row();
                });
            ui.separator();
            if let Some(err) = &self.edit_customer_state.error_message {
                ui.colored_label(Color32::RED, err);
            }
            ui.horizontal(|ui| {
                if ui.button("Save Changes").clicked() {
                    let updated_customer = Customer {
                        name: self.edit_customer_state.name.trim().to_string(),
                        address: self.edit_customer_state.address.trim().to_string(),
                        phone: self.edit_customer_state.phone.trim().to_string(),
                        contact_person: self.edit_customer_state.contact_person.trim().to_string(),
                        contact_phone: self.edit_customer_state.contact_phone.trim().to_string(),
                        email: self.edit_customer_state.email.trim().to_string(),
                        code: self.edit_customer_state.code.trim().to_uppercase(),
                    };
                    match self.db.edit_customer_gui(&self.edit_customer_state.original_name, updated_customer) {
                        Ok(_) => {
                            self.status_message = format!("Customer \"{}\" updated successfully.", self.edit_customer_state.name.trim());
                            self.update_customer_list();
                            if Some(self.edit_customer_state.original_name.clone()) == self.get_selected_customer_name() {
                                self.selected_customer_code = Some(self.edit_customer_state.code.trim().to_uppercase());
                                self.update_invoice_list();
                            }
                            self.edit_customer_state = EditCustomerState::default();
                            close_window = true;
                        },
                        Err(e) => {
                            self.edit_customer_state.error_message = Some(e.to_string());
                        }
                    }
                }
                if ui.button("Cancel").clicked() {
                    self.edit_customer_state = EditCustomerState::default();
                    close_window = true;
                }
            });
        });
        if close_window {
            self.show_edit_customer_window = false;
        }
    }

    fn delete_customer_confirm_window(&mut self, ctx: &Context) {
        let mut close_window = false;
        let mut confirmed_delete = false;
        let customer_name = self.customer_to_delete_code.as_ref().and_then(|code| {
            self.customers.iter().find(|c| c.code == *code).map(|c| c.name.clone())
        }).unwrap_or_else(|| "Unknown".to_string());

        Window::new("Confirm Delete Customer")
            .id(Id::new("delete_customer_confirm_window")) // Unique ID
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(format!("Are you sure you want to delete customer \"{}\" ({})?", 
                                customer_name, 
                                self.customer_to_delete_code.as_deref().unwrap_or("")));
                ui.label("This will also delete all associated invoices.");
                ui.colored_label(Color32::RED, "This action cannot be undone.");
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Yes, Delete Customer").clicked() {
                        confirmed_delete = true;
                        close_window = true;
                    }
                    if ui.button("Cancel").clicked() {
                        close_window = true;
                    }
                });
            });

        if confirmed_delete {
            if let Some(code) = self.customer_to_delete_code.take() {
                match self.db.delete_customer_gui(&code) {
                    Ok(_) => {
                        self.status_message = format!("Customer \"{}\" ({}) deleted successfully.", customer_name, code);
                        self.selected_customer_code = None; // Deselect customer
                        self.invoices_for_selected_customer.clear();
                        self.update_customer_list();
                    },
                    Err(e) => {
                        self.status_message = format!("Error deleting customer: {}", e);
                    }
                }
            }
        }

        if close_window {
            self.show_delete_customer_confirm_window = false;
            if !confirmed_delete { // Clear the code if cancelled
                self.customer_to_delete_code = None;
            }
        }
    }

    // Added confirmation window for deleting invoice
    fn delete_invoice_confirm_window(&mut self, ctx: &Context) {
        let mut close_window = false;
        let mut confirmed_delete = false;
        let invoice_number = self.invoice_to_delete_number.clone().unwrap_or_default();

        Window::new("Confirm Delete Invoice")
            .id(Id::new("delete_invoice_confirm_window")) // Unique ID
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(format!("Are you sure you want to delete invoice #{}?", invoice_number));
                ui.colored_label(Color32::RED, "This action cannot be undone.");
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Yes, Delete Invoice").clicked() {
                        confirmed_delete = true;
                        close_window = true;
                    }
                    if ui.button("Cancel").clicked() {
                        close_window = true;
                    }
                });
            });

        if confirmed_delete {
            if let Some(num) = self.invoice_to_delete_number.take() {
                match self.db.delete_invoice_gui(&num) {
                    Ok(_) => {
                        self.status_message = format!("Invoice #{} deleted successfully.", num);
                        self.selected_invoice_number = None; // Deselect invoice
                        self.update_invoice_list(); // Refresh list
                    },
                    Err(e) => {
                        self.status_message = format!("Error deleting invoice: {}", e);
                    }
                }
            }
        }

        if close_window {
            self.show_delete_invoice_confirm_window = false;
            if !confirmed_delete { // Clear the number if cancelled
                self.invoice_to_delete_number = None;
            }
        }
    }

    fn create_invoice_window(&mut self, ctx: &Context) {
        let mut close_window = false;
        // Use customer code in the ID to make it unique per customer
        let window_id = Id::new(format!("create_invoice_window_{}", self.create_invoice_state.customer_code));
        Window::new(format!("Create Invoice for {}", self.create_invoice_state.customer_name))
            .id(window_id) // Unique ID for the window
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
            ui.label(format!("Customer: {} ({})", self.create_invoice_state.customer_name, self.create_invoice_state.customer_code));
            ui.separator();
            ui.label("Invoice Items:");
            // Use customer code in the ScrollArea ID
            let scroll_id = Id::new(format!("create_invoice_items_scroll_{}", self.create_invoice_state.customer_code));
            ScrollArea::vertical().id_source(scroll_id).max_height(200.0).show(ui, |ui| {
                let mut item_to_remove = None;
                let num_items = self.create_invoice_state.items.len(); // Get length before loop
                for (i, item_state) in self.create_invoice_state.items.iter_mut().enumerate() {
                    // Keep using index for item ID as it's unique within this window instance
                    ui.push_id(format!("create_item_{}", i), |ui| {
                        Grid::new(format!("item_grid_{}", i))
                            .num_columns(4)
                            .spacing([10.0, 4.0])
                            .show(ui, |ui| {
                                ui.label("Description:");
                                ui.add(TextEdit::singleline(&mut item_state.description).hint_text("Item/Service"));
                                ui.label("Quantity:");
                                ui.add(TextEdit::singleline(&mut item_state.quantity_str).hint_text("e.g., 1"));
                                ui.end_row();
                                ui.label("Rate:");
                                ui.add(TextEdit::singleline(&mut item_state.rate_str).hint_text("e.g., 50.00"));
                                if num_items > 1 { // Use variable here
                                    if ui.button("Remove").clicked() {
                                        item_to_remove = Some(i);
                                    }
                                } else {
                                    ui.label(""); // Placeholder
                                }
                                ui.end_row();
                            });
                        ui.separator();
                    });
                }
                if let Some(index) = item_to_remove {
                    self.create_invoice_state.items.remove(index);
                }
            });
            if ui.button("Add Item").clicked() {
                self.create_invoice_state.items.push(InvoiceItemState::default());
            }
            ui.separator();
            ui.label("Notes:");
            ui.text_edit_multiline(&mut self.create_invoice_state.notes);
            ui.separator();
            ui.label("Due Date (YYYY-MM-DD):");
            ui.text_edit_singleline(&mut self.create_invoice_state.due_date_str);
            ui.separator();
            if let Some(err) = &self.create_invoice_state.error_message {
                ui.colored_label(Color32::RED, err);
            }
            ui.horizontal(|ui| {
                if ui.button("Create Invoice").clicked() {
                    let mut items = Vec::new();
                    let mut valid = true;
                    for item_state in &self.create_invoice_state.items {
                        let quantity = match item_state.quantity_str.parse::<u32>() {
                            Ok(q) if q > 0 => q,
                            _ => {
                                self.create_invoice_state.error_message = Some("Invalid quantity. Must be a positive integer.".to_string());
                                valid = false;
                                break;
                            }
                        };
                        let rate = match item_state.rate_str.parse::<f64>() {
                            Ok(r) if r >= 0.0 => r,
                            _ => {
                                self.create_invoice_state.error_message = Some("Invalid rate. Must be a non-negative number.".to_string());
                                valid = false;
                                break;
                            }
                        };
                        if item_state.description.trim().is_empty() {
                            self.create_invoice_state.error_message = Some("Item description cannot be empty.".to_string());
                            valid = false;
                            break;
                        }
                        items.push(InvoiceItem {
                            description: item_state.description.trim().to_string(),
                            quantity,
                            rate,
                            amount: 0.0, // Will be calculated in backend
                        });
                    }

                    let due_date = if valid {
                        match NaiveDate::parse_from_str(&self.create_invoice_state.due_date_str, "%Y-%m-%d") {
                            Ok(d) => Some(d),
                            Err(_) => {
                                self.create_invoice_state.error_message = Some("Invalid due date format. Use YYYY-MM-DD.".to_string());
                                valid = false;
                                None
                            }
                        }
                    } else { None };

                    if valid {
                        if let Some(due_date_naive) = due_date {
                            match self.db.create_invoice_gui(
                                self.create_invoice_state.customer_code.clone(),
                                items,
                                self.create_invoice_state.notes.trim().to_string(),
                                due_date_naive,
                            ) {
                                Ok(invoice) => {
                                    self.status_message = format!("Invoice #{} created successfully.", invoice.invoice_number);
                                    self.update_invoice_list();
                                    self.create_invoice_state = CreateInvoiceState::default();
                                    close_window = true;
                                },
                                Err(e) => {
                                    self.create_invoice_state.error_message = Some(e.to_string());
                                }
                            }
                        }
                    }
                }
                if ui.button("Cancel").clicked() {
                    self.create_invoice_state = CreateInvoiceState::default();
                    close_window = true;
                }
            });
        });
        if close_window {
            self.show_create_invoice_window = false;
        }
    }

    // Added window for editing invoices
    fn edit_invoice_window(&mut self, ctx: &Context) {
        let mut close_window = false;
        // Get customer code from the main app state, not the edit state
        let customer_code = self.selected_customer_code.clone().unwrap_or_default(); 
        // Use invoice number in the ID to make it unique per invoice
        let window_id = Id::new(format!("edit_invoice_window_{}", self.edit_invoice_state.original_invoice_number));
        Window::new(format!("Edit Invoice #{} for {}", self.edit_invoice_state.original_invoice_number, self.edit_invoice_state.customer_name))
            .id(window_id) // Unique ID for the window
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
            // Use the retrieved customer_code here
            ui.label(format!("Customer: {} ({})", self.edit_invoice_state.customer_name, customer_code)); 
            ui.separator();
            ui.label("Invoice Items:");
            // Use invoice number in the ScrollArea ID
            let scroll_id = Id::new(format!("edit_invoice_items_scroll_{}", self.edit_invoice_state.original_invoice_number));
            ScrollArea::vertical().id_source(scroll_id).max_height(200.0).show(ui, |ui| {
                let mut item_to_remove = None;
                let num_items = self.edit_invoice_state.items.len(); // Get length before loop
                for (i, item_state) in self.edit_invoice_state.items.iter_mut().enumerate() {
                    ui.push_id(format!("edit_item_{}", i), |ui| { // Unique ID for each item
                        Grid::new(format!("edit_item_grid_{}", i))
                            .num_columns(4)
                            .spacing([10.0, 4.0])
                            .show(ui, |ui| {
                                ui.label("Description:");
                                ui.add(TextEdit::singleline(&mut item_state.description).hint_text("Item/Service"));
                                ui.label("Quantity:");
                                ui.add(TextEdit::singleline(&mut item_state.quantity_str).hint_text("e.g., 1"));
                                ui.end_row();
                                ui.label("Rate:");
                                ui.add(TextEdit::singleline(&mut item_state.rate_str).hint_text("e.g., 50.00"));
                                if num_items > 1 { // Use variable here
                                    if ui.button("Remove").clicked() {
                                        item_to_remove = Some(i);
                                    }
                                } else {
                                    ui.label(""); // Placeholder
                                }
                                ui.end_row();
                            });
                        ui.separator();
                    });
                }
                if let Some(index) = item_to_remove {
                    self.edit_invoice_state.items.remove(index);
                }
            });
            if ui.button("Add Item").clicked() {
                self.edit_invoice_state.items.push(InvoiceItemState::default());
            }
            ui.separator();
            ui.label("Notes:");
            ui.text_edit_multiline(&mut self.edit_invoice_state.notes);
            ui.separator();
            ui.label("Due Date (YYYY-MM-DD):");
            ui.text_edit_singleline(&mut self.edit_invoice_state.due_date_str);
            ui.checkbox(&mut self.edit_invoice_state.paid, "Mark as Paid");
            ui.separator();
            if let Some(err) = &self.edit_invoice_state.error_message {
                ui.colored_label(Color32::RED, err);
            }
            ui.horizontal(|ui| {
                if ui.button("Save Changes").clicked() {
                    let mut items = Vec::new();
                    let mut valid = true;
                    for item_state in &self.edit_invoice_state.items {
                        let quantity = match item_state.quantity_str.parse::<u32>() {
                            Ok(q) if q > 0 => q,
                            _ => {
                                self.edit_invoice_state.error_message = Some("Invalid quantity. Must be a positive integer.".to_string());
                                valid = false;
                                break;
                            }
                        };
                        let rate = match item_state.rate_str.parse::<f64>() {
                            Ok(r) if r >= 0.0 => r,
                            _ => {
                                self.edit_invoice_state.error_message = Some("Invalid rate. Must be a non-negative number.".to_string());
                                valid = false;
                                break;
                            }
                        };
                        if item_state.description.trim().is_empty() {
                            self.edit_invoice_state.error_message = Some("Item description cannot be empty.".to_string());
                            valid = false;
                            break;
                        }
                        items.push(InvoiceItem {
                            description: item_state.description.trim().to_string(),
                            quantity,
                            rate,
                            amount: 0.0, // Will be calculated in backend
                        });
                    }

                    let due_date = if valid {
                        match NaiveDate::parse_from_str(&self.edit_invoice_state.due_date_str, "%Y-%m-%d") {
                            Ok(d) => Some(d),
                            Err(_) => {
                                self.edit_invoice_state.error_message = Some("Invalid due date format. Use YYYY-MM-DD.".to_string());
                                valid = false;
                                None
                            }
                        }
                    } else { None };

                    if valid {
                        if let Some(due_date_naive) = due_date {
                            match self.db.edit_invoice_gui(
                                &self.edit_invoice_state.original_invoice_number,
                                items,
                                self.edit_invoice_state.notes.trim().to_string(),
                                due_date_naive,
                                self.edit_invoice_state.paid,
                            ) {
                                Ok(_) => {
                                    self.status_message = format!("Invoice #{} updated successfully.", self.edit_invoice_state.original_invoice_number);
                                    self.update_invoice_list();
                                    self.edit_invoice_state = EditInvoiceState::default();
                                    close_window = true;
                                },
                                Err(e) => {
                                    self.edit_invoice_state.error_message = Some(e.to_string());
                                }
                            }
                        }
                    }
                }
                if ui.button("Cancel").clicked() {
                    self.edit_invoice_state = EditInvoiceState::default();
                    close_window = true;
                }
            });
        });
        if close_window {
            self.show_edit_invoice_window = false;
        }
    }

    fn view_invoice_window(&mut self, ctx: &Context) {
        let mut close_window = false;
        if let Some(invoice) = &self.invoice_to_view {
            // Use invoice number in the ID to make it unique per invoice
            let window_id = Id::new(format!("view_invoice_window_{}", invoice.invoice_number));
            Window::new(format!("View Invoice #{}", invoice.invoice_number))
                .id(window_id) // Unique ID for the window
                .resizable(true)
                .collapsible(true)
                .default_width(500.0)
                .show(ctx, |ui| {
                ui.heading(format!("Invoice #{} for {}", invoice.invoice_number, invoice.customer.name));
                ui.separator();
                Grid::new("view_invoice_details_grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Customer:");
                        ui.label(format!("{} ({})", invoice.customer.name, invoice.customer.code));
                        ui.end_row();
                        ui.label("Date Issued:");
                        ui.label(invoice.date.format("%Y-%m-%d %H:%M").to_string());
                        ui.end_row();
                        ui.label("Date Due:");
                        ui.label(invoice.due_date.format("%Y-%m-%d").to_string());
                        ui.end_row();
                        ui.label("Status:");
                        ui.label(if invoice.paid { "Paid" } else { "Unpaid" });
                        ui.end_row();
                    });
                ui.separator();
                ui.heading("Items");
                // Use invoice number in the ScrollArea ID
                let scroll_id = Id::new(format!("view_invoice_items_scroll_{}", invoice.invoice_number));
                ScrollArea::vertical().id_source(scroll_id).max_height(200.0).show(ui, |ui| {
                    ui.push_id("view_items_scroll", |ui| { // Unique ID for the scroll area content (redundant? maybe remove)
                        Grid::new("view_invoice_items_grid")
                            .num_columns(4)
                            .spacing([10.0, 4.0])
                            .striped(true)
                            .min_col_width(100.0)
                            .show(ui, |ui| {
                                ui.label(RichText::new("Description").strong());
                                ui.label(RichText::new("Quantity").strong());
                                ui.label(RichText::new("Rate").strong());
                                ui.label(RichText::new("Amount").strong());
                                ui.end_row();
                                for (i, item) in invoice.items.iter().enumerate() {
                                    ui.push_id(format!("view_item_{}", i), |ui| { // Unique ID for each item row
                                        ui.label(&item.description);
                                        ui.label(item.quantity.to_string());
                                        ui.label(format!("{:.2}", item.rate));
                                        ui.label(format!("{:.2}", item.amount));
                                        ui.end_row();
                                    });
                                }
                            });
                    });
                });
                ui.separator();
                Grid::new("view_invoice_totals_grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Subtotal:");
                        ui.label(format!("{:.2}", invoice.subtotal));
                        ui.end_row();
                        // Add Tax/GST if applicable later
                        ui.label(RichText::new("Total:").strong());
                        ui.label(RichText::new(format!("{:.2}", invoice.total)).strong());
                        ui.end_row();
                    });
                if !invoice.notes.is_empty() {
                    ui.separator();
                    ui.label("Notes:");
                    ScrollArea::vertical().max_height(60.0).show(ui, |ui| {
                        ui.label(&invoice.notes);
                    });
                }
                ui.separator();
                if ui.button("Close").clicked() {
                    close_window = true;
                }
            });
        } else {
            // Should not happen if window is shown, but handle gracefully
            close_window = true;
        }
        if close_window {
            self.show_view_invoice_window = false;
            self.invoice_to_view = None;
        }
    }

    fn update_customer_list(&mut self) {
        self.customers = self.db.get_customers_vec();
    }

    fn update_invoice_list(&mut self) {
        if let Some(code) = &self.selected_customer_code {
            self.invoices_for_selected_customer = self.db.get_invoices_for_customer(code);
        } else {
            self.invoices_for_selected_customer.clear();
        }
        self.selected_invoice_number = None; // Deselect invoice when list updates
    }

    fn get_selected_customer_name(&self) -> Option<String> {
        self.selected_customer_code.as_ref().and_then(|code| {
            self.customers.iter().find(|c| c.code == *code).map(|c| c.name.clone())
        })
    }
}

impl eframe::App for KmattInvoiceApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) { // Changed frame to _frame as it's not used directly
        // Menu Bar
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Save Database").clicked() {
                        match self.db.save() {
                            Ok(_) => self.status_message = "Database saved successfully.".to_string(),
                            Err(e) => self.status_message = format!("Error saving database: {}", e),
                        }
                        ui.close_menu();
                    }
                    if ui.button("Exit").clicked() {
                        // Save is handled by Drop trait
                        // Use ViewportCommand to request close
                        ctx.send_viewport_cmd(ViewportCommand::Close);
                        ui.close_menu(); // Close menu after clicking exit
                    }
                });
                ui.menu_button("Customer", |ui| {
                    if ui.button("Add New Customer").clicked() {
                        self.add_customer_state = AddCustomerState::default();
                        self.show_add_customer_window = true;
                        ui.close_menu();
                    }
                    let edit_enabled = self.selected_customer_code.is_some();
                    if ui.add_enabled(edit_enabled, egui::Button::new("Edit Selected Customer")).clicked() {
                        if let Some(code) = &self.selected_customer_code {
                            if let Some(customer) = self.customers.iter().find(|c| c.code == *code) {
                                self.edit_customer_state = EditCustomerState {
                                    original_name: customer.name.clone(),
                                    name: customer.name.clone(),
                                    address: customer.address.clone(),
                                    phone: customer.phone.clone(),
                                    contact_person: customer.contact_person.clone(),
                                    contact_phone: customer.contact_phone.clone(),
                                    email: customer.email.clone(),
                                    code: customer.code.clone(),
                                    error_message: None,
                                };
                                self.show_edit_customer_window = true;
                            }
                        }
                        ui.close_menu();
                    }
                    let delete_enabled = self.selected_customer_code.is_some();
                    if ui.add_enabled(delete_enabled, egui::Button::new("Delete Selected Customer")).clicked() {
                        self.customer_to_delete_code = self.selected_customer_code.clone();
                        self.show_delete_customer_confirm_window = true;
                        ui.close_menu();
                    }
                });
                ui.menu_button("Invoice", |ui| {
                    let create_enabled = self.selected_customer_code.is_some();
                    if ui.add_enabled(create_enabled, egui::Button::new("Create New Invoice")).clicked() {
                        if let Some(code) = &self.selected_customer_code {
                            if let Some(customer) = self.customers.iter().find(|c| c.code == *code) {
                                self.create_invoice_state = CreateInvoiceState {
                                    customer_code: customer.code.clone(),
                                    customer_name: customer.name.clone(),
                                    ..Default::default()
                                };
                                self.show_create_invoice_window = true;
                            }
                        }
                        ui.close_menu();
                    }
                    let invoice_selected = self.selected_invoice_number.is_some();
                    if ui.add_enabled(invoice_selected, egui::Button::new("Edit Selected Invoice")).clicked() {
                        if let Some(inv_num) = &self.selected_invoice_number {
                            if let Some(invoice) = self.invoices_for_selected_customer.iter().find(|inv| inv.invoice_number == *inv_num) {
                                self.edit_invoice_state = EditInvoiceState {
                                    original_invoice_number: invoice.invoice_number.clone(),
                                    // Removed unused field: customer_code: invoice.customer.code.clone(),
                                    customer_name: invoice.customer.name.clone(),
                                    items: invoice.items.iter().map(|item| InvoiceItemState {
                                        description: item.description.clone(),
                                        quantity_str: item.quantity.to_string(),
                                        rate_str: format!("{:.2}", item.rate),
                                    }).collect(),
                                    notes: invoice.notes.clone(),
                                    due_date_str: invoice.due_date.format("%Y-%m-%d").to_string(),
                                    paid: invoice.paid,
                                    error_message: None,
                                };
                                self.show_edit_invoice_window = true;
                            }
                        }
                        ui.close_menu();
                    }
                    if ui.add_enabled(invoice_selected, egui::Button::new("Delete Selected Invoice")).clicked() {
                        self.invoice_to_delete_number = self.selected_invoice_number.clone();
                        self.show_delete_invoice_confirm_window = true;
                        ui.close_menu();
                    }
                });
            });
        });

        // Status Bar
        TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.label(&self.status_message);
        });

        // Left Panel (Customer List)
        SidePanel::left("left_panel").resizable(true).show(ctx, |ui| {
            ui.heading("Customers");
            let mut clicked_customer_code = None; // Variable to store clicked customer code
            ScrollArea::vertical().show(ui, |ui| {
                for customer in &self.customers {
                    let is_selected = self.selected_customer_code.as_ref() == Some(&customer.code);
                    if ui.selectable_label(is_selected, format!("{} ({})", customer.name, customer.code)).clicked() {
                        // Store the clicked code instead of updating immediately
                        clicked_customer_code = Some(customer.code.clone());
                    }
                }
            });

            // Update selection and invoice list *after* the loop
            if let Some(code) = clicked_customer_code {
                self.selected_customer_code = Some(code);
                self.update_invoice_list();
            }
        });

        // Central Panel (Invoice List for Selected Customer)
        CentralPanel::default().show(ctx, |ui| {
            if let Some(name) = self.get_selected_customer_name() {
                ui.heading(format!("Invoices for {}", name));
                ScrollArea::vertical().show(ui, |ui| {
                    Grid::new("invoice_list_grid")
                        .num_columns(6) // Added columns for Edit/Delete
                        .spacing([10.0, 4.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label(RichText::new("Number").strong());
                            ui.label(RichText::new("Date").strong());
                            ui.label(RichText::new("Total").strong());
                            ui.label(RichText::new("Status").strong());
                            ui.label(RichText::new("Actions").strong()); // Combined actions
                            ui.label(""); // PDF Action
                            ui.end_row();

                            let mut invoice_to_mark_paid = None;
                            let mut invoice_to_view_details = None;
                            let mut invoice_to_generate_pdf = None;
                            let mut invoice_to_edit = None; // For Edit button
                            let mut invoice_to_delete = None; // For Delete button

                            for invoice in &self.invoices_for_selected_customer {
                                let is_selected = self.selected_invoice_number.as_ref() == Some(&invoice.invoice_number);
                                let response = ui.selectable_label(is_selected, &invoice.invoice_number);
                                if response.clicked() {
                                    self.selected_invoice_number = Some(invoice.invoice_number.clone());
                                }
                                ui.label(invoice.date.format("%Y-%m-%d").to_string());
                                ui.label(format!("{:.2}", invoice.total));
                                ui.label(if invoice.paid { "Paid" } else { "Unpaid" });
                                
                                // Action buttons in one cell
                                ui.horizontal(|ui| {
                                    if ui.button("View").clicked() {
                                        invoice_to_view_details = Some(invoice.clone());
                                    }
                                    if !invoice.paid {
                                        if ui.button("Mark Paid").clicked() {
                                            invoice_to_mark_paid = Some(invoice.invoice_number.clone());
                                        }
                                    }
                                    // Edit Button
                                    if ui.button("Edit").clicked() {
                                        invoice_to_edit = Some(invoice.clone());
                                    }
                                    // Delete Button
                                    if ui.button("Delete").clicked() {
                                        invoice_to_delete = Some(invoice.invoice_number.clone());
                                    }
                                });
                                // PDF Button in separate cell
                                if ui.button("PDF").clicked() {
                                    invoice_to_generate_pdf = Some(invoice.invoice_number.clone());
                                }
                                ui.end_row();
                            }

                            if let Some(num) = invoice_to_mark_paid {
                                match self.db.mark_invoice_paid_gui(&num) {
                                    Ok(_) => {
                                        self.status_message = format!("Invoice #{} marked as paid.", num);
                                        self.update_invoice_list();
                                    },
                                    Err(e) => self.status_message = format!("Error marking invoice paid: {}", e),
                                }
                            }
                            if let Some(invoice) = invoice_to_view_details {
                                self.invoice_to_view = Some(invoice);
                                self.show_view_invoice_window = true;
                            }
                            if let Some(num) = invoice_to_generate_pdf {
                                if let Some(path) = FileDialog::new()
                                    .set_file_name(&format!("Invoice-{}.pdf", num))
                                    .add_filter("PDF", &["pdf"])
                                    .save_file() {
                                    match self.db.generate_pdf_gui(&num, path.to_str().unwrap_or_default()) {
                                        Ok(filename) => self.status_message = format!("PDF generated: {}", filename),
                                        Err(e) => self.status_message = format!("Error generating PDF: {}", e),
                                    }
                                } else {
                                    self.status_message = "PDF generation cancelled.".to_string();
                                }
                            }
                            // Handle Edit Invoice action
                            if let Some(invoice) = invoice_to_edit {
                                self.edit_invoice_state = EditInvoiceState {
                                    original_invoice_number: invoice.invoice_number.clone(),
                                    // Removed unused field: customer_code: invoice.customer.code.clone(),
                                    customer_name: invoice.customer.name.clone(),
                                    items: invoice.items.iter().map(|item| InvoiceItemState {
                                        description: item.description.clone(),
                                        quantity_str: item.quantity.to_string(),
                                        rate_str: format!("{:.2}", item.rate),
                                    }).collect(),
                                    notes: invoice.notes.clone(),
                                    due_date_str: invoice.due_date.format("%Y-%m-%d").to_string(),
                                    paid: invoice.paid,
                                    error_message: None,
                                };
                                self.show_edit_invoice_window = true;
                            }
                            // Handle Delete Invoice action
                            if let Some(num) = invoice_to_delete {
                                self.invoice_to_delete_number = Some(num);
                                self.show_delete_invoice_confirm_window = true;
                            }
                        });
                });
            } else {
                ui.label("Select a customer from the left panel to view invoices.");
            }
        });

        // Modal Windows
        if self.show_add_customer_window {
            self.add_customer_window(ctx);
        }
        if self.show_edit_customer_window {
            self.edit_customer_window(ctx);
        }
        if self.show_delete_customer_confirm_window {
            self.delete_customer_confirm_window(ctx);
        }
        if self.show_create_invoice_window {
            self.create_invoice_window(ctx);
        }
        if self.show_view_invoice_window {
            self.view_invoice_window(ctx);
        }
        if self.show_edit_invoice_window {
            self.edit_invoice_window(ctx);
        }
        if self.show_delete_invoice_confirm_window {
            self.delete_invoice_confirm_window(ctx);
        }
    }
}

// Implement Drop to save database on exit
impl Drop for KmattInvoiceApp {
    fn drop(&mut self) {
        println!("Saving database before exit...");
        if let Err(e) = self.db.save() {
            eprintln!("Error saving database on exit: {}", e);
        }
    }
}

