"""
Dispatch
Brian Yu

A command-line mail merge tool.

Your data CSV file should include:
    - Row 0: each column is a keyword name
    - Rows 1+: each column defines values for a new email

Your configuration JSON file should include:
    - required: from, to, subject, body (unless provided separately)
    - optional: name, password, cc, bcc, server, port

You may also provide an optional body text file.

"""

import argparse
import csv
import getpass
import json
import smtplib
import sys
import termcolor
import time
import traceback

from email.header import Header
from email.mime.text import MIMEText
from email.utils import formataddr


def main():

    # Set up exception handler.
    sys.excepthook = excepthook

    # Parse command line arguments.
    parser = argparse.ArgumentParser()
    parser.add_argument("data", nargs="?",
                        help="csv data file with parameters")
    parser.add_argument("config", nargs="?",
                        help="configuration file for dispatcher")
    parser.add_argument("textfile", nargs="?",
                        help="text file containing email to send")
    parser.add_argument("-v", "--verbose", action="store_true",
                        help="show detailed output")
    parser.add_argument("-m", "--make", help="create configuration files")
    args = vars(parser.parse_args())

    # If -m flag used, create configuration files.
    if args["make"]:
        make_configuration(args["make"])
        return

    # Run dispatcher.
    dispatch(args)


def dispatch(args):
    """
    Dispatch mass emails.
    """

    if not all([args["config"], args["data"]]):
        raise Error("Configuration and data files must be specified.")

    # Parse configuration files, and get password if needed.
    config = parse_config(args["config"])
    config["password"] = config.get("password") or getpass.getpass()
    if args["textfile"]:
        body = open(args["textfile"]).read()
        config["body"] = body.replace("\n", "<br/>")
    headers, data = parse_data(args["data"])

    # Connect to mail server.
    server = mail_server(config)

    # Send one email for each row in the CSV file.
    for row in data:

        # Format the email parameters by substituting data from the CSV.
        params = {key: format_param(config[key], headers, row)
                  if key not in ["from", "password"] else config[key]
                  for key in config}

        # Prepare the message and send it.
        msg = prepare_message(params)
        server.send_message(msg)

        # Print confirmation.
        print_confirmation(params, args["verbose"])
        time.sleep(1)

    # Success!
    server.quit()
    termcolor.cprint("Dispatch complete!", "green")


def make_configuration(email):
    """
    Creates a simple standard set of configuration files.
    """

    # Create the JSON configuration file.
    with open("config.json", "w") as f:
        data = json.dumps({
            "from": email,
            "name": "",
            "to": "{email}",
            "subject": ""
        }, indent=4)
        f.write(data)

    # Create the CSV data file.
    with open("data.csv", "w") as f:
        f.write("name,email\n")

    # Create an email text file.
    with open("body.html", "w") as f:
        f.write("Hey {name}!")


def excepthook(type, value, tb):
    """
    Global exception handler.
    """

    if type is Error and str(value):
        termcolor.cprint(str(value), "red")
    else:
        traceback.print_exception(type, value, tb)


def format_param(contents, headers, row):
    """
    Substitutes values (rows) based on keywords (headers) present in contents.
    """

    def format_param_string(contents, headers, row):
        """
        Performs substitution upon a string.
        """

        # Create a dictionary where keys should be substituted for values.
        subs = {key: row[headers[key]] for key in headers}
        try:
            return contents.format(**subs)
        except KeyError as e:
            raise Error("Missing parameter in data file: {}".format(e.args[0]))

    # If contents is a list, format elements. Otherwise, format the string.
    if isinstance(contents, list):
        return [format_param_string(item, headers, row) for item in contents]
    else:
        return format_param_string(contents, headers, row)


def mail_server(config):
    """
    Configures SMTP server.
    """

    # Connect to server.
    server = smtplib.SMTP(
        config.get("server", "smtp.gmail.com"),
        int(config.get("port", 587))
    )
    server.ehlo()
    server.starttls()
    try:
        server.login(config.get("login_from", config.get("from", "")), config.get("password", ""))
    except smtplib.SMTPAuthenticationError:
        raise Error("Invalid username or password.")
    return server


def parse_config(filename):
    """
    Parses a JSON configuration file.
    """

    # Open the file.
    try:
        f = open(filename, "r")
    except FileNotFoundError:
        raise Error("Configuration file {} does not exist.".format(filename))

    # Read the contents and parse as JSON.
    contents = f.read()
    f.close()
    try:
        data = json.loads(contents)
    except json.decoder.JSONDecodeError as e:
        raise Error("Invalid JSON in configuration file.\nError: {}".format(e))
    return data


def parse_data(filename):
    """
    Parses a CSV data file.
    """

    # Open the file.
    try:
        f = open(filename, "r")
    except FileNotFoundError:
        raise Error("Data file {} does not exist.".format(filename))

    # Read the data as a CSV.
    data = list(csv.reader(f, delimiter=","))

    # Use the first row to create a mapping of keywords to their column.
    headers, data = data[0], data[1:]
    headers = {headers[i].strip(): i for i in range(len(headers))}
    return headers, data


def prepare_message(params):
    """
    Returns a MIMEText message.
    """

    # Prepare message.
    msg = MIMEText(
        params.get("body", "").replace("\n", "<br/>"),
        "html"
    )
    msg["Subject"] = params.get("subject", "")

    # For "from" field, check for sender's name.
    if params.get("name"):
        msg["From"] = formataddr((
            str(Header(params.get("name"), "utf-8")),
            params.get("from")
        ))
    else:
        msg["From"] = params.get("from", "")

    # Add recipients.
    for field in ["to", "cc", "bcc"]:
        addresses = params.get(field, [])
        if not isinstance(addresses, list):
            addresses = [addresses]
        msg[field.capitalize()] = ", ".join(addresses)

    # Add reply-to.
    if params.get("reply-to"):
        msg["reply-to"] = params.get("reply-to")

    return msg


def print_confirmation(params, verbose):
    """
    Prints message confirming that email was sent.
    """

    to = params.get("to", [])
    recipient = ", ".join(to) if isinstance(to, list) else to
    if verbose:
        print("Sent email to {} with parameters {}".format(
              recipient, params))
    else:
        print("Sent email to {}".format(recipient))


class Error(Exception):
    """
    Custom error class.
    """
    pass


if __name__ == "__main__":
    main()
