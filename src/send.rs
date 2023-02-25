/// Dispatch emails
use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::path::Path;
use std::{thread, time};

use csv::ReaderBuilder;
use lettre::Transport;

use crate::config::DispatchConfig;
use crate::mail::{create_message, get_mailer, Substitutions};
use crate::util::prompt;

pub fn send(config_path: &Path, dry_run: bool, verbose: bool) -> Result<(), Box<dyn Error>> {
    let config = DispatchConfig::read_from_path(&config_path)?;
    let body = read_body(config_path, &config.body_path)?;
    let data = read_data(config_path, &config.data_path)?;

    // Validate subject and body to ensure variables are replaced.
    // Substitutions can also occur in email addresses, but missing substitutions will be
    // caught by address validation errors.
    match data.first() {
        None => {
            Err(format!("No emails to send. Add rows to {} to send emails.", &config.data_path))?;
        },
        Some(row) => {
            validate_text(&config.subject, row)?;
            validate_text(&body, row)?;
        },
    }

    let mailer = get_mailer(&config.username, &config.server, dry_run)?;
    for (i, row) in data.iter().enumerate() {
        let message = create_message(&config, &body, &row)?;
        if dry_run || verbose {
            println!("---------------------------- {} of {}", i + 1, data.len());
            println!("{}\n", std::str::from_utf8(&message.formatted())?);
        }
        if !dry_run {
            let to = &message
                .envelope()
                .to()
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            match mailer.send(&message) {
                Ok(_) => {
                    println!("Sent email {} of {} to {}", i + 1, data.len(), to);
                }
                Err(e) => Err(format!("Error sending email to {}: {}", to, e))?,
            }
            thread::sleep(time::Duration::from_millis(1000));
        }
    }
    println!("----------------------------");
    if dry_run {
        println!("Dry run successful!");
    } else {
        println!("Emails sent successfully!");
    }
    Ok(())
}

pub fn read_body(config_path: &Path, body_path: &str) -> Result<String, Box<dyn Error>> {
    let local_path = config_path.parent().unwrap().join(body_path);
    let body_path = Path::new(body_path);
    let body_path = if body_path.is_absolute() {
        body_path
    } else {
        &local_path
    };
    match fs::read_to_string(body_path) {
        Ok(body) => Ok(body),
        Err(e) => Err(format!("Could not read body file: {}", e))?,
    }
}

pub fn read_data(config_path: &Path, data_path: &str) -> Result<Vec<Substitutions>, Box<dyn Error>> {
    let local_path = config_path.parent().unwrap().join(data_path);
    let data_path = Path::new(data_path);
    let data_path = if data_path.is_absolute() {
        data_path
    } else {
        &local_path
    };
    let file = match File::open(data_path) {
        Ok(file) => file,
        Err(e) => Err(format!("Could not open data file: {}", e))?,
    };
    let mut reader = ReaderBuilder::new().from_reader(file);
    let mut data = Vec::new();
    for result in reader.deserialize() {
        let record: HashMap<String, String> = result?;
        data.push(record);
    }
    Ok(data)
}

/// Check text for possibly invalid substitutions
pub fn validate_text(content: &str, substitutions: &Substitutions) -> Result<(), Box<dyn Error>> {
    let mut buffer = String::new();
    let mut tracking = false;
    for char in content.chars() {
        if char == '{' {
            tracking = true;
            buffer = String::new();
        } else if tracking {
            if char == ' ' {
                tracking = false;
            } else if char == '}' {
                if !substitutions.contains_key(&buffer) {
                    let confirmation = prompt(&format!("Found possibly unmatched field {{{}}}. Send anyways", &buffer), Some("n"));
                    if confirmation.to_lowercase().starts_with("n") {
                        return Err("Dispatch canceled")?;
                    }
                }
                tracking = false;
            } else {
                buffer.push(char);
            }
        }
    }
    Ok(())
}
