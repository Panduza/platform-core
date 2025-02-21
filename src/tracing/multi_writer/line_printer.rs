use std::fmt::Write;

use colored::Colorize;

/// Color words in quotes
///
fn color_words_in_quotes(input: &str) -> String {
    let mut in_quotes = false;
    let mut result = String::new();
    let mut word = String::new();
    let mut prev_char = '\0';

    for c in input.chars() {
        match c {
            '"' if prev_char != '\\' => {
                word.push(c);
                in_quotes = !in_quotes;
                if !in_quotes {
                    result.push_str(&word.yellow().to_string());
                    word.clear();
                }
            }
            _ => {
                if in_quotes {
                    word.push(c);
                } else {
                    result.push(c);
                }
            }
        }
        prev_char = c;
    }

    result
}

///
///
///
pub fn print_log_line(buf: &[u8], enable_broker_log: bool, debug: bool, trace: bool) {
    //
    // Convert into string to help usage
    let source_string = String::from_utf8_lossy(buf);

    //
    // Split data
    let data: Vec<&str> = source_string.split(';').collect();

    //
    // Basic checks
    if data.len() < 9 {
        return;
    }

    //
    // Get class name
    let class = if data[3].is_empty() {
        "Broker"
    } else {
        data[3].trim_matches('"')
    };

    //
    // Skip broker logs if not requested
    if class == "Broker" && !enable_broker_log {
        return;
    }

    //
    // Filter terminal level
    let level = data[2];
    if !trace {
        if level == "TRACE" {
            return;
        }
    }
    if !debug && !trace {
        if level == "DEBUG" {
            return;
        }
    }

    //
    //
    let mut log_message = String::new();

    //
    //
    match class {
        "Broker" => {
            write!(&mut log_message, "{}", "[B]".to_string().on_blue()).unwrap();
            write!(&mut log_message, " ").unwrap();
        }
        "Platform" => {
            write!(&mut log_message, "{}", "[P] ".to_string().red()).unwrap();
        }
        "Service" => {
            let f = format!("[P> {}] ", data[4],);
            write!(&mut log_message, "{}", f.red()).unwrap();
        }
        "Runtime" => {
            write!(&mut log_message, "{}", "[R] ".to_string().red()).unwrap();
        }
        "Factory" => {
            write!(&mut log_message, "{}", "[F] ".to_string().magenta()).unwrap();
        }
        "Driver" => {
            let f = format!("[{}/{}/{}] ", data[4], data[5], data[6],);
            write!(&mut log_message, "{}", f.purple()).unwrap();
        }
        "Instance" => {
            let f = format!("[{}] ", data[4]);
            write!(&mut log_message, "{}", f.green()).unwrap();
        }
        "Class" => {
            let f = format!("[{}/{}] ", data[4], data[5],);
            write!(&mut log_message, "{}", f.blue()).unwrap();
        }
        "Attribute" => {
            if data[5].is_empty() {
                let f = format!("[{}/{}] ", data[4], data[6],);
                write!(&mut log_message, "{}", f.blue()).unwrap();
            } else {
                let f = format!("[{}/{}/{}] ", data[4], data[5], data[6],);
                write!(&mut log_message, "{}", f.blue()).unwrap();
            }
        }
        "Interface" => {
            let f = format!("[{}/{}/{}] ", data[4], data[5], data[6],);
            write!(&mut log_message, "{}", f.bright_cyan()).unwrap();
        }
        "SDK" => {
            let f = format!("[{}/{}/{}] ", data[4], data[5], data[6],);
            write!(&mut log_message, "{}", f.on_bright_cyan()).unwrap();
        }
        _ => {}
    }

    //
    // Level
    match level {
        "WARN" => {
            write!(&mut log_message, "{}", "[WARN]".to_string().on_yellow()).unwrap();
            write!(&mut log_message, " ").unwrap();
        }
        "ERROR" => {
            write!(&mut log_message, "{}", "[ERROR]".to_string().on_red()).unwrap();
            write!(&mut log_message, " ").unwrap();
        }
        _ => {}
    }

    //
    // message
    write!(&mut log_message, "{}", color_words_in_quotes(data[7])).unwrap();

    //
    //
    println!("{}", log_message);
}
