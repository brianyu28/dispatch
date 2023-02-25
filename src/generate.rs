/// Generate sample configuration files

use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use crate::config::{DispatchConfig, Recipients};

pub fn generate() -> Result<(), Box<dyn Error>> {
    let email = prompt("Email address", None);
    let name = prompt("Name", None);
    let server = prompt("Server", Some("smtp.gmail.com"));
    let subject = prompt("Subject", Some("Hello"));

    let config_path = prompt("Config Filename", Some("config.json"));
    let body_path = prompt("Email Body Filename", Some("body.html"));
    let data_path = prompt("Data Filename", Some("data.csv"));

    let from = if name.len() == 0 {
        None
    } else {
        Some(format!("{name} <{email}>"))
    };

    let config = DispatchConfig {
        username: email,
        from,
        to: Some(Recipients::Individual("{email}".to_string())),
        cc: None,
        bcc: None,
        subject: subject,
        data_path: data_path.to_string(),
        body_path: body_path.to_string(),
        content_type: "html".to_string(),
        server,
    };

    let config_text = serde_json::to_string_pretty(&config).unwrap();
    write_file("configuration file", &config_path, &config_text)?;
    write_file("body text", &body_path, "Hi {name}")?;
    write_file("data file", &data_path, "name,email")?;

    Ok(())
}

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

fn write_file(name: &str, filename: &str, contents: &str) -> Result<(), Box<dyn Error>> {
    let path = Path::new(filename);
    if path.exists() {
        let confirmation = prompt(&format!("Overwrite {}?", filename), Some("y"));
        if !confirmation.to_lowercase().starts_with("y") {
            return Ok(());
        }
    }
    match fs::write(path, contents) {
        Ok(_) => {}
        Err(e) => return Err(format!("Failed to write {}: {}", name, e))?,
    };
    Ok(())
}
