# Dispatch

Dispatch is a command-line mail merge tool for sending custom bulk emails to a list of recipients.

## Usage

Dispatch requires two files to start a mail merge:

* A configuration file, typically called `config.json`, which contains the sender's email address, password, recipients (to, cc, bcc), a subject line, and a body. Fields may be parameterized with a keyword placeholder between `{` and `}`, which will be substituted for by the dispatcher.
  * The configuration file may also specify a custom server and port (Gmail is the default).
  * The "from" address and the password cannot be parameterized.
  * If no password is specified, the user is prompted to type it in.
* A data file, typically called `data.csv`, which contains the information to substitute into the placehodlers. The first row of the CSV file should define column headings that match up with the keyword placeholders present in the configuration JSON file. Every subsequent row defines one email to send, and may parameterize the keywords in any way.

A sample `config.json` file is below:

```json
{
    "from": "from@example.com",
    "password": "PASSWORD",
    "to": ["{email}"],
    "subject": "Hello to {name}!",
    "body": "Dear {name},<br/><br/>Hello, world!"
}
```

And a corresponding sample `data.csv` file is below:

```
email,name
person1@example.com,Person 1
person2@example.com,Person 2
```
