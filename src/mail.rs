/// Configure SMTP and create email messages
use std::collections::HashMap;
use std::error::Error;

use lettre::message::header::ContentType;
use lettre::message::Mailbox;
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
    body: &str,
    data: &Substitutions,
) -> Result<Message, Box<dyn Error>> {
    let mut builder = Message::builder()
        .subject(substitute(&config.subject, data))
        .header(if config.content_type == "html" {
            ContentType::TEXT_HTML
        } else if config.content_type == "text" {
            ContentType::TEXT_PLAIN
        } else {
            Err("Invalid content type - must be html or text")?
        });

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

    let body = if config.content_type == "html" {
        body.replace("\n", "<br/>")
    } else {
        body.to_string()
    };

    Ok(builder.body(substitute(&body, data)).unwrap())
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
