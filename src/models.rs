use chrono::{DateTime, Local};
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Company {
    pub name: String,
    pub abn: String,
    pub address: String,
    pub phone: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Customer {
    pub name: String,
    pub address: String,
    pub phone: String,
    pub contact_person: String,
    pub contact_phone: String,
    pub email: String,
    pub code: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InvoiceItem {
    pub description: String,
    pub quantity: u32,
    pub rate: f64,
    pub amount: f64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub invoice_number: String,
    pub date: DateTime<Local>,
    pub due_date: DateTime<Local>,
    pub customer: Customer,
    pub items: Vec<InvoiceItem>,
    pub subtotal: f64,
    pub total: f64,
    pub notes: String,
    pub paid: bool,
}
