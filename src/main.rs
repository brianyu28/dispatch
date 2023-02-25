mod arguments;
mod config;
mod generate;
mod mail;
mod send;

use std::path::Path;

use clap::Parser;

use arguments::{Args, Command};

fn main() {
    let args = Args::parse();
    let result = match args.command {
        Command::Send {
            config_path,
            dry_run,
            verbose,
        } => {
            let path = Path::new(&config_path);
            send::send(path, dry_run, verbose)
        }
        Command::Generate {} => generate::generate(),
    };

    match result {
        Ok(_) => {}
        Err(e) => println!("Error: {}", e),
    };
    return;

    // match mailer.send(&email) {
    // Ok(_) => println!("Email sent successfully!"),
    // Err(e) => panic!("Could not send email: {:?}", e),
    // }
}