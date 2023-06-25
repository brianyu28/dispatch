/// Configure SMTP and create email messages
use std::collections::HashMap;
use std::error::Error;

use lettre::message::header::ContentType;
use lettre::message::{Mailbox, MultiPart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport};

use crate::config::{DispatchConfig, Recipients};

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

    let message = match (body_html, body_text) {
        (Some(body_html), Some(body_text)) => builder
            .multipart(MultiPart::alternative_plain_html(body_text, body_html))
            .unwrap(),
        (Some(body_html), None) => builder
            .header(ContentType::TEXT_HTML)
            .body(body_html)
            .unwrap(),
        (None, Some(body_text)) => builder
            .header(ContentType::TEXT_PLAIN)
            .body(body_text)
            .unwrap(),
        (None, None) => Err("Email must have either an HTML or text body")?,
    };

    Ok(message)
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
