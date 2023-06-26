/// Configuration for a Dispatch email group
use std::{error::Error, fs, path::Path};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct DispatchConfig {
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<Recipients>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cc: Option<Recipients>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bcc: Option<Recipients>,
    pub subject: String,
    #[serde(rename = "data")]
    pub data_path: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "body_html")]
    pub body_html_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "body_text")]
    pub body_text_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_content: Option<Vec<RelatedContentConfig>>,
    #[serde(default = "DispatchConfig::default_server")]
    pub server: String,
}

#[derive(Serialize, Deserialize)]
pub struct RelatedContentConfig {
    pub content_id: String,
    pub mime_type: String,
    pub path: String,
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

    /// Use Gmail as default server if none provided
    /// For Amazon SES, use: email-smtp.us-east-1.amazonaws.com
    fn default_server() -> String {
        "smtp.gmail.com".to_string()
    }
}

/// Represents one or more email recipients
/// Must be formatted as either "name@domain.tld" or "Name <name@domain.tld>"
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Recipients {
    Individual(String),
    Multiple(Vec<String>),
}
