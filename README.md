# Dispatch

A command-line tool for sending parameterized emails

## Usage

Generate a new email configuration with:

```
$ dispatch generate
```

The `generate` command will prompt for information and then create three files:

- The **configuration file** is a JSON file containing the information needed to
generate and send email: the sender's name, address, email server; the to, cc,
and bcc fields; the subject line; and paths to the body and data files.
- The **body file** is a file containing the body of the email. It can
optionally contain variables of the form `{varname}` that will be substituted
during dispatch. By default, this is an HTML file, but it can be plain text as
well.
- The **data file** is a CSV file containing the data for the emails to send. A
header row is required: it should specify one column for each of the `{varname}`
variables present in the email body or sender/recipient addresses. The data file
should then include one row for each email to be sent.

Once the emails are configured, send them with:

```
$ dispatch send <CONFIG_PATH>
```

The `send` command will prompt you for an email password and then send the
emails.

Optionally, add the `--dry-run` flag to view the contents of the emails before
sending them.

## Installation

To build from source, requires Rust 1.60 or newer.

```
$ cargo install --path .
```

## Notes

- For Google email addresses with 2-factor authentication, you'll need to create
and use an [App
Password](https://security.google.com/settings/security/apppasswords) to use
Dispatch.
