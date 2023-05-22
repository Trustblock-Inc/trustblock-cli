Trustblock CLI
==============

Trustblock CLI is a user-friendly command-line utility that helps you interact with Trustblock and publish audit reports.
Refer to the [Trustblock CLI documentation](https://docs.trustblock.run/technical-documentation/publish-an-audit#using-our-cli) for more detailed information.

Installation
------------

If you have cargo installed, you can install our CLI via:

```bash
cargo install trustblock-cli
```

For other installation methods please refer to: 

```bash
https://github.com/Trustblock-Inc/trustblock-cli/releases
```
Usage
-----

To see available subcommands, use the `help` flag:

```bash
trustblock help
```

For more information and help with specific subcommands, use the `--help` flag:

```bash
trustblock <subcommand> --help
```

### Setup

Before publishing an audit, run the following command to initialize the `~/.trustblock/.env` file:

```bash
trustblock init
```

Next, add your private key from the whitelisted wallet and JWT to the `~/.trustblock/.env` file. You can obtain an API key by navigating to your profile and clicking the "Edit my profile" button on the Trustblock website after authentication.

Note: Trustblock CLI can still be used without adding data to the `.env` file, as long as the required information is passed as arguments.

### Audit Publishing

To publish an audit, you can either attach pdf file or specify a web audit URL:

```bash
-r, --report-pdf <AUDIT_REPORT_PDF_FILE>
-u, --report-url <AUDIT_REPORT_URL>
```

Example: 

```bash
trustblock publish-audit -a audit.json -r ./Audit_Report.pdf
```

```bash
trustblock publish-audit -a audit.json -u https://trustblock.run/my_audit
```

You can obtain an example _audit.json_ file from https://github.com/Trustblock-Inc/trustblock-cli/blob/main/src/data/audit.json. You should fill in the fields with the appropriate information from your audit.


To include api key and private key:

```bash
-k, --api-key
-p, --private-key
```

To also publish to Smart Contracts, add the `--publish-sc` flag:

```bash
 -s, --publish-sc
```

Commands
--------

- `publish-audit`: Publishes an audit to Trustblock.
- `init`: Initializes the `.trustblock` folder.
- `clean`: Cleans the `.trustblock` folder.
- `help`: Print this message or the help of the given subcommand(s).

Audit JSON Schema
--------
```json
{
  "project": {
    "name": String,
    "links": {
      "website": URL String,
      "twitter": URL String
    },
    "contact": {
      "email": Email String
    }
  },
  "issues": {
    "FIXED": {
      "LOW": uint,
      "MEDIUM": uint,
      "HIGH": uint,
      "CRITICAL": uint
    },
    "RISK_ACCEPTED": {
      "LOW": uint,
      "MEDIUM": uint,
      "HIGH": uint,
      "CRITICAL": uint
    }
  },
    "tags": [ "TOKEN" | "FINANCE" | "COLLECTIBLES" | "GAMING" | "GOVERNANCE" | "SOCIAL" | "OTHER"],
    "contracts": [
        {
            "evmAddress": Evm Address String,
            "chain": "ETHEREUM" | "POLYGON" | "AVALANCHE" | "BNBCHAIN"
        }
    ],
    "description": {
        "summary": Markdown String
    },
    "name": String
}
```
