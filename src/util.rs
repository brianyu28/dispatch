use std::io::{self, Write};

pub fn prompt(description: &str, default: Option<&str>) -> String {
    print!("{description}");
    if let Some(default) = default {
        print!(" (default: {})", default);
    }
    print!("? ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    if let Some(default) = default {
        if input.trim().len() == 0 {
            return default.to_string();
        }
    }

    input.trim().to_string()
}
