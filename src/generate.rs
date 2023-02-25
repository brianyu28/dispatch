use std::{error::Error, fs};

use crate::config::{DispatchConfig, Recipients};

pub fn generate() -> Result<(), Box<dyn Error>> {
    let config = DispatchConfig {
        username: "".to_string(),
        from: None,
        to: Some(Recipients::Individual("{email}".to_string())),
        cc: None,
        bcc: None,
        subject: "Hello".to_string(),
        data_path: "data.csv".to_string(),
        body_path: "body.html".to_string(),
        content_type: "html".to_string(),
        server: "smtp.gmail.com".to_string(),
    };
    let config_text = serde_json::to_string_pretty(&config).unwrap();
    match fs::write("config.json", config_text) {
        Ok(_) => {}
        Err(e) => return Err(format!("Failed to write configuration file: {}", e))?,
    };

    let body_text = "Hello, {name}!";
    match fs::write("body.html", body_text) {
        Ok(_) => {}
        Err(e) => return Err(format!("Failed to write body file: {}", e))?,
    };

    let data_path = "name,email";
    match fs::write("data.csv", data_path) {
        Ok(_) => {}
        Err(e) => return Err(format!("Failed to write data file: {}", e))?,
    };

    Ok(())
}
