# Dispatch

A command-line tool for sending parameterized emails

## Usage

Generate a new email configuration with:

```
$ dispatch generate
```

The `generate` command will prompt for information and then create the following
files:

- The **configuration file** is a JSON file containing the information needed to
generate and send email: the sender's name, address, email server; the to, cc,
and bcc fields; the subject line; and paths to the body and data files.
- The **body file** is a file containing the body of the email. It can
optionally contain parameters of the form `{varname}` that will be substituted
during dispatch. The body can be a single HTML file, a single text file,
or both an HTML file and a text file for a `multipart/alternative` email.
- The **data file** is a CSV file containing the data for the emails to send. A
header row is required: it should specify one column for each of the `{varname}`
parameters present in the email body or sender/recipient addresses. The data file
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

To build and install from source (requires Rust 1.60 or newer):

```
$ cargo install --path .
```

## Configuration

### Configuration File

The configuration file is a JSON file (called `config.json` by default). The
file can have the following fields:

- `username`: The username to use for SMTP credentials when sending messages.
  Usually, this is your email address.
- `subject`: The subject line of the email.
- `data_path`: The path to the data file. This can be an absolute path or a
  relative path starting from the directory where the configuration file is
  stored.
- `body_path`: The path to the body file. This can be an absolute path or a
  relative path starting from the directory where the configuration file is
  stored.
- `server` (optional): The SMTP server to use. If none provided,
  `smtp.gmail.com` is used.
- `from` (optional): Who the email is from. This can be just an email address
  (`name@example.com`) or can have a name (`Example Name <name@example.com>`).
  Often, this is the same email address as `username`, but an alias may be used
  as well. If no `from` address is specificed, the `username` is used.
- `reply_to` (optional): The email address to use when a recipient replies to a
  message. Must be an email address, with or without a name.
- `to` (optional): The email address(es) for the "To" header of the email. Can
  be a single address or an array of addresses. Each address may have a name.
- `cc` (optional): The email address(es) for the "Cc" header of the email. Can
  be a single address or an array of addresses. Each address may have a name.
- `bcc` (optional): The email address(es) for the "Bcc" header of the email. Can
  be a single address or an array of addresses. Each address may have a name.
- `content_type` (optional): The content type of the email body. Must be `html`
  or `text`. If none provided, `html` is used.

The subject field (`subject`) and mailbox fields (`from`, `reply_to`, `to`,
`cc`, `bcc`) of the configuration file file may include parameters of the form
`{varname}`, to be substituted with values from the data file.

Example configuration file:

```json
{
  "username": "name@example.com",
  "from": "Example Name <name@example.com>",
  "to": "{email}",
  "subject": "Hello",
  "data": "data.csv",
  "body": "body.html",
  "content_type": "html",
  "server": "smtp.gmail.com"
}
```

### Body File

The body file is an HTML or text file specifying the body of the email. The body
file may include parameters of the form `{varname}`, to be substituted with
values from the data file.

When using `"content_type": "html"`, newlines in the body file are converted to
`<br/>`.

Example body file:

```
Hi {name},

This is an example email.
```

### Data File

The data file is a CSV file specifying how to parameterize each email.

- A header row is required. The column names in the header row should correspond
  to the parameters in the configuration file and in the body file.
- There should be one additional row for each email to send, specifying what
  values to use for each of the parameters.

Example data file:

```csv
email,name
person1@example.com,Person 1
person2@example.com,Person 2
```

## Notes

- For Google email addresses with 2-factor authentication, you'll need to create
and use an [App
Password](https://security.google.com/settings/security/apppasswords) to use
Dispatch.
- This tool used to be a Python script. If you're still looking for that version
of the tool, you can find it tagged as
[v0.1.0](https://github.com/brianyu28/dispatch/tree/v0.1.0).

## Authors

- [Brian Yu](https://github.com/brianyu28)

## License

- [GNU GPLv3](https://www.gnu.org/licenses/gpl-3.0.en.html)
