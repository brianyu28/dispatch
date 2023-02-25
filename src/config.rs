use std::{error::Error, fs, path::Path};

/// Configuration for a Dispatch email group
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct DispatchConfig {
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<Recipients>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cc: Option<Recipients>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bcc: Option<Recipients>,
    pub subject: String,
    #[serde(rename = "data")]
    pub data_path: String,
    #[serde(rename = "body")]
    pub body_path: String,
    #[serde(default = "DispatchConfig::default_content_type")]
    pub content_type: String,
    #[serde(default = "DispatchConfig::default_server")]
    pub server: String,
}

impl DispatchConfig {
    pub fn read_from_path(path: &Path) -> Result<DispatchConfig, Box<dyn Error>> {
        if let Ok(contents) = fs::read_to_string(&path) {
            match serde_json::from_str(&contents) {
                Ok(config) => Ok(config),
                Err(e) => Err(format!("Invalid configuration file format: {}", e))?,
            }
        } else {
            Err(format!(
                "Could not read configuration file: {}",
                path.to_str().unwrap()
            ))?
        }
    }

    fn default_content_type() -> String {
        "html".to_string()
    }

    /// Use Gmail as default server if none provided
    fn default_server() -> String {
        "smtp.gmail.com".to_string()
    }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Recipients {
    Individual(String),
    Multiple(Vec<String>),
}
