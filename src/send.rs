/// Send emails
use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::path::Path;
use std::{thread, time};

use csv::ReaderBuilder;
use lettre::Transport;

use crate::config::DispatchConfig;
use crate::mail::{create_message, get_mailer};

pub fn send(config_path: &Path, dry_run: bool, verbose: bool) -> Result<(), Box<dyn Error>> {
    let config = DispatchConfig::read_from_path(&config_path)?;
    let body = read_body(config_path, &config.body_path)?;
    let data = read_data(config_path, &config.data_path)?;

    let mailer = get_mailer(&config.username, &config.server)?;
    for row in data {
        let message = create_message(&config, &body, &row)?;
        if dry_run || verbose {
            println!("----------------------------");
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
                    println!("Sent email to {}\n", to);
                }
                Err(e) => Err(format!("Error sending email to {}: {}\n", to, e))?,
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
    let body_path = config_path.parent().unwrap().join(body_path);
    Ok(fs::read_to_string(body_path)?)
}

pub fn read_data(
    config_path: &Path,
    data_path: &str,
) -> Result<Vec<HashMap<String, String>>, Box<dyn Error>> {
    let data_path = config_path.parent().unwrap().join(data_path);
    let file = File::open(data_path)?;
    let mut reader = ReaderBuilder::new().from_reader(file);
    let mut data = Vec::new();
    for result in reader.deserialize() {
        let record: HashMap<String, String> = result?;
        data.push(record);
    }
    Ok(data)
}
