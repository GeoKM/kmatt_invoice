use printpdf::*;
use crate::models::Invoice;
use crate::utils::wrap_text;
use prettytable::{Table, Row, Cell, format};
use std::io::BufWriter;
use std::fs::File;
use std::error::Error;

// Updated function signature to accept filename and return Result
pub fn generate_pdf(
    invoice: &Invoice, 
    company_name: &str, 
    company_abn: &str, 
    company_address: &str, 
    company_phone: &str,
    filename: &str // Added filename parameter
) -> Result<(), Box<dyn Error>> { // Return Result
    // Initialize PDF document (A4 size: 210mm x 297mm)
    let (doc, page1, layer1) = PdfDocument::new(
        format!("Invoice #{}", invoice.invoice_number),
        Mm(210.0),
        Mm(297.0),
        "Layer 1",
    );
    let layer = doc.get_page(page1).get_layer(layer1);
    let helvetica_font = doc.add_builtin_font(BuiltinFont::Helvetica).map_err(|e| e.to_string())?; // Use map_err for error conversion
    let courier_font = doc.add_builtin_font(BuiltinFont::Courier).map_err(|e| e.to_string())?;
    let font_size = 10.0;
    let line_height = 4.23; // ~12pt for 10pt font (1pt = 0.3527mm)
    let mut y_pos = 280.0; // Start near top of page

    // Helper to add text at specific positions with specified font
    let add_text = |layer: &PdfLayerReference, text: &str, x: Mm, y: f32, font: &IndirectFontRef| {
        layer.use_text(text, font_size, x, Mm(y), font);
    };

    // Company Header (Helvetica)
    add_text(&layer, company_name, Mm(15.0), y_pos, &helvetica_font);
    y_pos -= line_height;
    add_text(&layer, &format!("A.B.N. {}", company_abn), Mm(15.0), y_pos, &helvetica_font);
    y_pos -= line_height;
    add_text(&layer, company_address, Mm(15.0), y_pos, &helvetica_font);
    y_pos -= line_height;
    add_text(&layer, &format!("Ph: {}", company_phone), Mm(15.0), y_pos, &helvetica_font);
    y_pos -= line_height;
    add_text(&layer, &format!("Invoice #{}", invoice.invoice_number), Mm(15.0), y_pos, &helvetica_font);
    y_pos -= line_height;
    add_text(&layer, &format!("Date: {}", invoice.date.format("%b %d, %Y")), Mm(15.0), y_pos, &helvetica_font);
    y_pos -= 2.0 * line_height; // Extra spacing

    // Bill To and Payment Terms
    let bill_to_y = y_pos;
    add_text(&layer, "Bill To:", Mm(15.0), bill_to_y, &helvetica_font);
    y_pos -= line_height;
    // Corrected: Use string literal "\n\n" for split
    for line in invoice.customer.name.split("\n\n") {
        add_text(&layer, line, Mm(15.0), y_pos, &helvetica_font);
        y_pos -= line_height;
    }
    add_text(&layer, &invoice.customer.address, Mm(15.0), y_pos, &helvetica_font);
    y_pos -= line_height;
    add_text(&layer, &format!("Phone: {}", invoice.customer.phone), Mm(15.0), y_pos, &helvetica_font);
    y_pos -= line_height;

    // Corrected: Use string literal "\n\n" for split
    let contact_lines: Vec<&str> = invoice.customer.contact_person.split("\n\n").collect();
    // Corrected: Use string literal "\n\n" for split
    let email_lines: Vec<&str> = invoice.customer.email.split("\n\n").collect();

    let mut attn_line = String::from("Attn - ");
    if !contact_lines.is_empty() {
        attn_line.push_str(contact_lines[0]);
    }
    let wrapped_attn_lines = wrap_text(&attn_line, 80);
    for line in wrapped_attn_lines {
        add_text(&layer, &line, Mm(15.0), y_pos, &helvetica_font);
        y_pos -= line_height;
    }

    add_text(&layer, &format!("Contact Phone: {}", invoice.customer.contact_phone), Mm(15.0), y_pos, &helvetica_font);
    y_pos -= line_height;

    if !email_lines.is_empty() {
        let email_line = format!("Email: {}", email_lines[0]);
        let wrapped_email_lines = wrap_text(&email_line, 80);
        for line in wrapped_email_lines {
            add_text(&layer, &line, Mm(15.0), y_pos, &helvetica_font);
            y_pos -= line_height;
        }
    }

    for i in 1..contact_lines.len() {
        add_text(&layer, &format!("       {}", contact_lines[i]), Mm(15.0), y_pos, &helvetica_font);
        y_pos -= line_height;
    }

    for i in 1..email_lines.len() {
        add_text(&layer, &format!("       {}", email_lines[i]), Mm(15.0), y_pos, &helvetica_font);
        y_pos -= line_height;
    }

    let bill_to_y_end = y_pos;
    y_pos = bill_to_y;
    add_text(&layer, "Payment Terms: Net 30 Days", Mm(150.0), y_pos, &helvetica_font);
    y_pos -= line_height;
    add_text(&layer, &format!("Due Date: {}", invoice.due_date.format("%b %d, %Y")), Mm(150.0), y_pos, &helvetica_font);
    y_pos -= line_height;
    add_text(&layer, &format!("Balance Due: AU ${:.2}", invoice.total), Mm(150.0), y_pos, &helvetica_font);

    y_pos = bill_to_y_end.min(y_pos);
    y_pos -= 2.0 * line_height;
    add_text(&layer, &format!("(Current Date: {})", invoice.date.format("%b %d, %Y")), Mm(15.0), y_pos, &helvetica_font);
    y_pos -= 2.0 * line_height;

    // Create the table
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_CLEAN);
    table.set_titles(Row::new(vec![
        Cell::new("#"),
        Cell::new("Item"),
        Cell::new("Qty"),
        Cell::new("Rate"),
        Cell::new("Amount"),
    ]));

    for (idx, item) in invoice.items.iter().enumerate() {
        let line_num = idx + 1;
        let description_lines = wrap_text(&item.description, 50);
        for (i, line) in description_lines.iter().enumerate() {
            if i == 0 {
                table.add_row(Row::new(vec![
                    Cell::new(&format!("{:>3}", line_num)),
                    Cell::new(line),
                    Cell::new(&format!("{:>6}", item.quantity)),
                    Cell::new(&format!("AU ${:>6.2}", item.rate)),
                    Cell::new(&format!("AU ${:>6.2}", item.amount)),
                ]));
            } else {
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

    let table_string = table.to_string();
    let table_lines: Vec<&str> = table_string.lines().collect();

    for line in table_lines {
        add_text(&layer, line, Mm(15.0), y_pos, &courier_font);
        y_pos -= line_height;
    }

    y_pos -= 3.0 * line_height;
    add_text(&layer, "Total:", Mm(73.0), y_pos, &helvetica_font);
    add_text(&layer, &format!("AU ${:.2}", invoice.total), Mm(87.0), y_pos, &courier_font);

    y_pos -= 2.0 * line_height;

    // Notes
    add_text(&layer, "Notes:", Mm(15.0), y_pos, &helvetica_font);
    y_pos -= line_height;
    add_text(&layer, &invoice.notes, Mm(15.0), y_pos, &helvetica_font);
    y_pos -= 2.0 * line_height;

    // Payment Instructions
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

    // Save PDF using the provided filename
    let file = File::create(filename)?; // Use filename parameter and propagate error
    doc.save(&mut BufWriter::new(file)).map_err(|e| e.to_string())?; // Propagate save error
    
    Ok(())
}

