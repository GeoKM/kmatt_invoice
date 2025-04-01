use std::io::{self};
use chrono::NaiveDate;

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

pub fn read_line() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

pub fn read_non_empty_string(error_msg: &str) -> String {
    loop {
        let input = read_line();
        if !input.is_empty() {
            return input;
        }
        println!("{}", error_msg);
    }
}

pub fn read_multi_line(error_msg: &str) -> String {
    let mut lines = Vec::new();
    loop {
        let input = read_line();
        if input.is_empty() {
            if lines.is_empty() {
                println!("{}", error_msg);
                continue;
            }
            break;
        }
        lines.push(input);
    }
    lines.join("\n")
}

pub fn read_optional_multi_line(default: &str) -> String {
    let mut lines = Vec::new();
    loop {
        let input = read_line();
        if input.is_empty() {
            if lines.is_empty() {
                return default.to_string();
            }
            break;
        }
        lines.push(input);
    }
    lines.join("\n")
}

pub fn read_optional_non_empty(default: &str, error_msg: &str) -> String {
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

pub fn read_customer_code() -> String {
    loop {
        let input = read_line().to_uppercase();
        if input.len() >= 2 && input.len() <= 3 && input.chars().all(|c| c.is_ascii_alphabetic()) {
            return input;
        }
        println!("Code must be 2-3 letters!");
    }
}

pub fn read_optional_customer_code(default: &str) -> String {
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

pub fn read_yes_no() -> String {
    loop {
        let input = read_line().to_lowercase();
        if input == "y" || input == "n" {
            return input;
        }
        println!("Please enter 'y' or 'n'!");
    }
}

pub fn read_positive_u32() -> u32 {
    loop {
        match read_line().parse::<u32>() {
            Ok(n) if n > 0 => return n,
            _ => println!("Please enter a positive integer!"),
        }
    }
}

pub fn read_positive_f64() -> f64 {
    loop {
        match read_line().parse::<f64>() {
            Ok(n) if n > 0.0 => return n,
            _ => println!("Please enter a positive number!"),
        }
    }
}

pub fn read_date_after(current_date: NaiveDate) -> NaiveDate {
    loop {
        match read_line().parse::<NaiveDate>() {
            Ok(date) => {
                if date > current_date {
                    return date;
                } else {
                    println!("Due date must be after the current date. Please try again.");
                }
            }
            Err(_) => println!("Invalid date format. Use YYYY-MM-DD and try again."),
        }
    }
}
