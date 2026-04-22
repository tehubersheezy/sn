# sn

A fast, single-binary CLI for the ServiceNow Table API. Designed for LLM agents and human operators alike.

`sn` wraps ServiceNow's REST Table API and two schema-discovery endpoints into a predictable command-line interface with stable JSON output, structured error reporting, and deterministic exit codes.

## Installation

### Homebrew (macOS / Linux)

```bash
brew install ibrahimsafah/tap/sn
```

### Shell installer (macOS / Linux)

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/ibrahimsafah/sn/releases/latest/download/sn-installer.sh | sh
```

### PowerShell installer (Windows)

```powershell
powershell -ExecutionPolicy ByPass -c "irm https://github.com/ibrahimsafah/sn/releases/latest/download/sn-installer.ps1 | iex"
```

### Cargo (from source)

```bash
cargo install sn
```

### Pre-built binaries

Download from [Releases](https://github.com/ibrahimsafah/sn/releases). Binaries are available for:

- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`

## Setup

Connect to a ServiceNow instance:

```bash
sn init
# Instance (e.g. acme.service-now.com): mycompany.service-now.com
# Username: admin
# Password: ********
# profile 'default' saved and verified.
```

Add additional instances with named profiles:

```bash
sn init --profile prod --instance prod.service-now.com --username svc-user
sn init --profile dev  --instance dev.service-now.com  --username admin
```

Select a profile per command or set a default:

```bash
sn --profile prod table list incident --page-size 5
sn profile use prod                  # set as default
SN_PROFILE=dev sn table list change_request   # env override
```

Verify credentials:

```bash
sn auth test
```

## Usage

### Reading records

```bash
# List incidents (default: up to 1000 records)
sn table list incident

# Filter + select fields
sn table list incident \
  --query "active=true^priority=1" \
  --fields "number,short_description,state" \
  --page-size 10

# Get a single record
sn table get incident <sys_id>

# Display human-readable values instead of internal codes
sn table get incident <sys_id> --display-value all
```

### Writing records

```bash
# Create with key-value pairs
sn table create incident \
  --field short_description="Disk full on prod-db-01" \
  --field urgency=2

# Create with JSON
sn table create incident --data '{"short_description": "Server down", "priority": "1"}'

# Create from file
sn table create incident --data @body.json

# Update (PATCH) — only changes named fields
sn table update incident <sys_id> --field state=2

# Replace (PUT) — overwrites the entire record
sn table replace incident <sys_id> --data @full-record.json

# Delete
sn table delete incident <sys_id> --yes
```

### Pagination

```bash
# Stream all records as JSONL (one per line)
sn table list incident --query "active=true" --all

# Collect into a single JSON array
sn table list incident --query "active=true" --all --array

# Cap total records
sn table list incident --all --max-records 5000

# Process with jq
sn table list incident --all | jq -r '.number'
```

### Schema discovery

Discover tables and their structure without prior ServiceNow knowledge:

```bash
# Find tables matching a keyword
sn schema tables --filter incident

# List writable columns for a table
sn schema columns incident --writable

# Get valid values for a choice field
sn schema choices incident state
```

### Agent integration

`sn introspect` dumps the full command tree as structured JSON, suitable for auto-generating MCP tool definitions or function-call schemas:

```bash
sn introspect | jq '.subcommands[] | {name, about}'
```

## Output contract

| Verb | stdout |
|---|---|
| `table list` | JSON array of records |
| `table get` / `create` / `update` / `replace` | Single record object |
| `table delete` | Nothing (empty) |
| `schema tables` / `columns` / `choices` | JSON array |

- `--output raw` preserves ServiceNow's `{"result": ...}` envelope.
- Pretty-printed when stdout is a TTY; compact when piped. Override with `--pretty` / `--compact`.
- Errors are always JSON on stderr: `{"error": {"message", "detail?", "status_code?", "transaction_id?"}}`.

### Exit codes

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | Usage or config error |
| 2 | API error (4xx/5xx, non-auth) |
| 3 | Network / transport error |
| 4 | Auth error (401/403) |

## Parameters

Every ServiceNow `sysparm_*` parameter has both a friendly name and a raw alias:

| Friendly | Alias | Values |
|---|---|---|
| `--query` | `--sysparm-query` | Encoded query string |
| `--fields` | `--sysparm-fields` | Comma-separated field list |
| `--page-size` | `--limit`, `--sysparm-limit` | Records per page (default 1000) |
| `--offset` | `--sysparm-offset` | Starting offset |
| `--display-value` | `--sysparm-display-value` | `true`, `false`, `all` |
| `--exclude-reference-link` | `--sysparm-exclude-reference-link` | Boolean |
| `--view` | `--sysparm-view` | Named UI view |
| `--input-display-value` | `--sysparm-input-display-value` | Boolean (writes) |
| `--suppress-auto-sys-field` | `--sysparm-suppress-auto-sys-field` | Boolean (writes) |
| `--query-no-domain` | `--sysparm-query-no-domain` | Boolean |
| `--no-count` | `--sysparm-no-count` | Boolean |

## Configuration

Credentials are stored in two files (AWS CLI-style split):

| File | Contains | Location (Linux) |
|---|---|---|
| `config.toml` | Instance URLs, default profile | `~/.config/sn/` |
| `credentials.toml` | Usernames, passwords (chmod 600) | `~/.config/sn/` |

Environment variables override profile values:

```bash
SN_INSTANCE=https://myco.service-now.com \
SN_USERNAME=api-user \
SN_PASSWORD=secret \
  sn table list incident --page-size 1
```

## Debugging

```bash
sn -v table list incident       # show HTTP method, URL, status
sn -vv table list incident      # add response headers
sn -vvv table list incident     # add request/response bodies (auth masked)
```

## License

MIT OR Apache-2.0
