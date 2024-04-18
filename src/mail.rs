/// Configure SMTP and create email messages
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;

use lettre::message::header::ContentType;
use lettre::message::{Attachment, Body, Mailbox, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport};

use crate::config::{DispatchConfig, Recipients, RelatedContentConfig};

pub type Substitutions = HashMap<String, String>;

pub fn get_mailer(
    username: &str,
    server: &str,
    dry_run: bool,
) -> Result<SmtpTransport, Box<dyn Error>> {
    if username.len() == 0 {
        return Err("Username missing from config")?;
    }
    let password = if dry_run {
        String::new()
    } else {
        rpassword::prompt_password(format!("Password for {}: ", username))?
    };
    let credentials = Credentials::new(username.to_string(), password.to_string());
    let mailer = SmtpTransport::relay(&server)
        .unwrap()
        .credentials(credentials)
        .build();
    Ok(mailer)
}

pub fn create_message(
    config_path: &Path,
    config: &DispatchConfig,
    body_html_template: &Option<String>,
    body_text_template: &Option<String>,
    data: &Substitutions,
) -> Result<Message, Box<dyn Error>> {
    let body_html = match body_html_template {
        Some(template) => Some(substitute(template, data)),
        None => None,
    };
    let body_text = match body_text_template {
        Some(template) => Some(substitute(template, data)),
        None => None,
    };

    let mut builder = Message::builder().subject(substitute(&config.subject, data));

    let from: &str = match &config.from {
        Some(from) => from,
        None => &config.username,
    };
    builder = builder.from(mailbox_from_address(from, data)?);

    if let Some(reply_to) = &config.reply_to {
        builder = builder.reply_to(mailbox_from_address(reply_to, data)?);
    }

    for to in mailboxes_from_recipients(&config.to, data)? {
        builder = builder.to(to);
    }
    for cc in mailboxes_from_recipients(&config.cc, data)? {
        builder = builder.cc(cc);
    }
    for bcc in mailboxes_from_recipients(&config.bcc, data)? {
        builder = builder.bcc(bcc);
    }

    // Get all related content files
    let mut related_contents: Vec<RelatedContent> = Vec::new();
    match &config.related_content {
        None => {}
        Some(related_content_configs) => {
            for config in related_content_configs {
                let related_content = RelatedContent::new(config_path, config, data)?;
                related_contents.push(related_content);
            }
        }
    };

    // Build a `multipart/related` message if the HTML message contains related content (e.g. embedded images).
    let multipart_related_content = if related_contents.len() == 0 {
        None
    } else if let Some(body_html) = &body_html {
        let mut multipart_builder =
            MultiPart::related().singlepart(SinglePart::html(String::from(body_html)));
        for related_content in related_contents {
            multipart_builder = multipart_builder.singlepart(
                Attachment::new_inline(String::from(&related_content.content_id)).body(
                    related_content.body.clone(),
                    related_content.mime_type.parse().unwrap(),
                ),
            )
        }
        Some(multipart_builder)
    } else {
        None
    };

    let message = match (body_html, body_text) {
        (Some(body_html), Some(body_text)) => {
            let alternative = MultiPart::alternative().singlepart(SinglePart::plain(body_text));
            let alternative = match multipart_related_content {
                None => alternative.singlepart(SinglePart::html(body_html)),
                Some(multipart_related_content) => alternative.multipart(multipart_related_content),
            };
            builder.multipart(alternative).unwrap()
        }
        (Some(body_html), None) => match multipart_related_content {
            None => builder
                .header(ContentType::TEXT_HTML)
                .body(body_html)
                .unwrap(),
            Some(multipart_related_content) => {
                builder.multipart(multipart_related_content).unwrap()
            }
        },
        (None, Some(body_text)) => builder
            .header(ContentType::TEXT_PLAIN)
            .body(body_text)
            .unwrap(),
        (None, None) => Err("Email must have either an HTML or text body")?,
    };

    Ok(message)
}

fn create_file_body(config_path: &Path, file_path: &str) -> Result<Body, Box<dyn Error>> {
    let local_path = config_path.parent().unwrap().join(file_path);
    let file_path = Path::new(file_path);
    let file_path = if file_path.is_absolute() {
        file_path
    } else {
        &local_path
    };
    let file = fs::read(file_path)?;
    Ok(Body::new(file))
}

fn substitute(text: &str, substitutions: &Substitutions) -> String {
    let mut result = text.to_string();
    for (placeholder, replacement) in substitutions.iter() {
        result = result.replace(&format!("{{{}}}", placeholder), replacement);
    }
    result
}

fn mailboxes_from_recipients(
    recipients: &Option<Recipients>,
    substitutions: &Substitutions,
) -> Result<Vec<Mailbox>, Box<dyn Error>> {
    match recipients {
        Some(Recipients::Individual(recipient)) => {
            Ok(vec![mailbox_from_address(&recipient, substitutions)?])
        }
        Some(Recipients::Multiple(recipients)) => {
            let mut mailboxes = vec![];
            for recipient in recipients {
                mailboxes.push(mailbox_from_address(&recipient, substitutions)?);
            }
            Ok(mailboxes)
        }
        None => Ok(vec![]),
    }
}

fn mailbox_from_address(
    address: &str,
    substitutions: &Substitutions,
) -> Result<Mailbox, Box<dyn Error>> {
    match substitute(address, substitutions).parse() {
        Ok(address) => Ok(address),
        Err(e) => Err(format!("Could not parse address {}: {}", address, e))?,
    }
}

pub struct RelatedContent {
    pub content_id: String,
    pub mime_type: String,
    pub body: Body,
}

impl RelatedContent {
    pub fn new(
        config_path: &Path,
        config: &RelatedContentConfig,
        substitutions: &Substitutions,
    ) -> Result<RelatedContent, Box<dyn Error>> {
        let content_path = substitute(&config.path, substitutions);
        let Ok(body) = create_file_body(config_path, &content_path) else {
            return Err(format!(
                "Could not read related content file {}",
                &content_path
            ))?;
        };
        Ok(RelatedContent {
            content_id: String::from(&config.content_id),
            mime_type: String::from(&config.mime_type),
            body,
        })
    }
}
