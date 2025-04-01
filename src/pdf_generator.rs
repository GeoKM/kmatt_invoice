use printpdf::*;
use crate::models::Invoice;
use crate::utils::wrap_text;
use prettytable::{Table, Row, Cell, format};
use std::io::BufWriter;

pub fn generate_pdf(invoice: &Invoice, company_name: &str, company_abn: &str, company_address: &str, company_phone: &str) {
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

    // Bill To and Payment Terms on same lines, left-justified and right-justified respectively
    let bill_to_y = y_pos; // Store y-position for Bill To
    // Bill To (Helvetica, left-justified at 15mm)
    add_text(&layer, "Bill To:", Mm(15.0), bill_to_y, &helvetica_font);
    y_pos -= line_height;
    for line in invoice.customer.name.split('\n') {
        add_text(&layer, line, Mm(15.0), y_pos, &helvetica_font);
        y_pos -= line_height;
    }
    add_text(&layer, &invoice.customer.address, Mm(15.0), y_pos, &helvetica_font);
    y_pos -= line_height;
    add_text(&layer, &format!("Phone: {}", invoice.customer.phone), Mm(15.0), y_pos, &helvetica_font);
    y_pos -= line_height;

    // Add contact person, contact phone, and email, handling multi-line fields
    let contact_lines: Vec<&str> = invoice.customer.contact_person.split('\n').collect();
    let email_lines: Vec<&str> = invoice.customer.email.split('\n').collect();

    // Add "Attn - " followed by the first line of contact person
    let mut attn_line = String::from("Attn - ");
    if !contact_lines.is_empty() {
        attn_line.push_str(contact_lines[0]);
    }
    let wrapped_attn_lines = wrap_text(&attn_line, 80);
    for line in wrapped_attn_lines {
        add_text(&layer, &line, Mm(15.0), y_pos, &helvetica_font);
        y_pos -= line_height;
    }

    // Add contact phone on a new line
    add_text(&layer, &format!("Contact Phone: {}", invoice.customer.contact_phone), Mm(15.0), y_pos, &helvetica_font);
    y_pos -= line_height;

    // Add email on a new line, handling multi-line emails
    if !email_lines.is_empty() {
        let email_line = format!("Email: {}", email_lines[0]);
        let wrapped_email_lines = wrap_text(&email_line, 80);
        for line in wrapped_email_lines {
            add_text(&layer, &line, Mm(15.0), y_pos, &helvetica_font);
            y_pos -= line_height;
        }
    }

    // Add remaining lines of contact person, indented
    for i in 1..contact_lines.len() {
        add_text(&layer, &format!("       {}", contact_lines[i]), Mm(15.0), y_pos, &helvetica_font);
        y_pos -= line_height;
    }

    // Add remaining lines of email, indented
    for i in 1..email_lines.len() {
        add_text(&layer, &format!("       {}", email_lines[i]), Mm(15.0), y_pos, &helvetica_font);
        y_pos -= line_height;
    }

    // Reset y_pos to align Payment Terms with Bill To
    let bill_to_y_end = y_pos; // Store the lowest y_pos after Bill To section (removed `mut`)
    y_pos = bill_to_y;
    // Payment Terms (Helvetica, right-justified at 150mm)
    add_text(&layer, "Payment Terms: Net 30 Days", Mm(150.0), y_pos, &helvetica_font);
    y_pos -= line_height;
    add_text(&layer, &format!("Due Date: {}", invoice.due_date.format("%b %d, %Y")), Mm(150.0), y_pos, &helvetica_font);
    y_pos -= line_height;
    add_text(&layer, &format!("Balance Due: AU ${:.2}", invoice.total), Mm(150.0), y_pos, &helvetica_font);

    // Use the lowest y_pos between Bill To and Payment Terms for the next section
    y_pos = bill_to_y_end.min(y_pos);
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

    // Render table lines into PDF using Courier font for alignment
    for line in table_lines {
        add_text(&layer, line, Mm(15.0), y_pos, &courier_font);
        y_pos -= line_height;
    }

    // Add Total on a separate line, right-justified with separate strings
    y_pos -= 3.0 * line_height; // Extra spacing to ensure a fresh line
    add_text(&layer, "Total:", Mm(73.0), y_pos, &helvetica_font); // Position "Total:" with 8mm gap before amount
    add_text(&layer, &format!("AU ${:.2}", invoice.total), Mm(87.0), y_pos, &courier_font); // Right edge at 97mm

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
    let file = std::fs::File::create(format!("invoice_{}.pdf", invoice.invoice_number)).unwrap();
    doc.save(&mut BufWriter::new(file)).unwrap();
    println!("PDF generated: invoice_{}.pdf", invoice.invoice_number);
}
