# Kivro

A secret manager that stores secrets in the OS keyring and injects into processes on demand, replacing the need for .env files

```text
.kivro.toml (you can commit this)           OS Keyring (not committed)
     gives us a list of what is needed  +     the values of those secrets

the secrets are injected into the process via:
kivro run -- start command for application
```

the child process is granted access to the secrets your app specified and nothing more.

## Why

There is no issues with traditional `.env` files, unless you commit it. The reason for building this entire tool is more to target smaller teams. This prevents users from sharing raw .env files or secrets in unsafe ways like over discord. Using kivro allows us to maintain a database of secrets that can be shared using proper encryption and then used easily by multiple devs and across multiple projects.

## Install

```bash
cargo install --path crates/kivro-cli
cargo install --git https://github.com/CameronGr/kivro kivro-cli
```

Requires rust 1.96 or never. On linux, the service needs a running keyring daemon, this can be GNOME keyring, KWallet, keepassxc with Secret Service enabled, etc. note `libdbus-1-dev` is needed at build time.

| Platform | Backend            |
| -------- | ------------------ |
| Windows  | Cretendial Manager |
| macOS    | Keychain           |
| Linux    | Secret Service     |

## Quick start

```bash

kivro init              # creates .kivro.toml, you can commit this
$EDITOR .kivro.toml     # declare the variables that you need for your project

kivro set MY_VARIABLE   # set your var value
kivro status            # what variables are present, what is needed
kivro run -- cargo run  # start a cargo project with the variables injected
```

Migrating a project with an existing .env

```bash
kivro import .env
kivro status
rm .env
```

Sharing your secrets with a colleague:

```bash
# your pc
kivro share  # outputs project_name.dev.secrets (an encrypted file, safe to share by whatever means)


kivro accept ./file.dev.secrets
kivro status
# friend pc
```

The bundle is encrypted with [age](https://age-encryption.org/v1) and is safe to
send over any channel. Send the passphrase over a _different_ channel or else that kinda defeats the purpose huh.

## Commands

| Command             | Purpose                                              |
| ------------------- | ---------------------------------------------------- |
| `kivro init`        | create `.kivro.toml`                                 |
| `kivro set NAME`    | store a value (prompted, or `--stdin` for CI)        |
| `kivro get NAME`    | check presence; `--show` to print the value          |
| `kivro list`        | names and presence, never values                     |
| `kivro remove NAME` | delete a value                                       |
| `kivro status`      | required-secret report; exit 3 when incomplete       |
| `kivro doctor`      | diagnose manifest, keyring, environment, git hygiene |
| `kivro run -- CMD`  | run a command with secrets injected                  |
| `kivro import FILE` | import a `.env`                                      |
| `kivro export`      | write a `.env` (explicit, warns, never automatic)    |
| `kivro share`       | create an encrypted bundle                           |
| `kivro accept FILE` | import an encrypted bundle                           |
| `kivro sync`        | compare manifest, local store and a sync source      |

Every command accepts `--env`, `--project`, `--json`, `--quiet` and `--no-color`.
See [docs/CLI.md](docs/CLI.md) for the full specification and exit codes.

## Using it as a library

The CLI is one consumer of the `kivro` crate, not the only interface:

```rust
use kivro::Project;

let project = Project::discover()?;
let secrets = project.environment("dev")?.load()?;
let database_url = secrets.get("DATABASE_URL")?;

std::process::Command::new("cargo")
  .args(["run"])
  .envs(secrets.environment())
  .spawn()?;
```

See [docs/INTEGRATION.md](docs/INTEGRATION.md) for Rust, TS and generic integration patterns, and `examples/` for working code.

## Documentation

| Document                                     | Contents                                      |
| -------------------------------------------- | --------------------------------------------- |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | crate layout, storage model, design decisions |
| [docs/MANIFEST.md](docs/MANIFEST.md)         | `.kivro.toml` specification                   |
| [docs/BUNDLE.md](docs/BUNDLE.md)             | encrypted bundle format specification         |
| [docs/SECURITY.md](docs/SECURITY.md)         | security model and threat model               |
| [docs/CLI.md](docs/CLI.md)                   | command reference and exit codes              |
| [docs/INTEGRATION.md](docs/INTEGRATION.md)   | integrating `kivro run`                       |

## Status

Version 0.1. The manifest and bundle formats are versioned and will be read by
future releases; see the compatibility sections in their specifications.
