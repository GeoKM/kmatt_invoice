// Keep wrap_text as it might be useful, though not currently used by GUI
pub fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
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
                current_line.push(' '); // Corrected syntax
            }
            current_line.push_str(word);
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    lines
}

// Removed unused CLI helper functions:
// read_line
// read_non_empty_string
// read_multi_line
// read_optional_multi_line
// read_optional_non_empty
// read_customer_code
// read_optional_customer_code
// read_date

