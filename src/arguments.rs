use clap::{Parser, Subcommand};

/// Utility for sending parameterized batch emails
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Send a new dispatch based on configuration file
    Send {
        /// Path to the dispatch configuration file
        #[arg(id = "CONFIG")]
        config_path: String,

        /// Test what will be sent without sending any emails
        #[arg(short, long)]
        dry_run: bool,

        /// Log content of all emails sent to stdout
        #[arg(short, long)]
        verbose: bool,
    },

    /// Generate configuration file, body file, and data file
    Generate {},
}
