# CLI reference

## Global options

Accepted by every subcommand.

| Option                   | Meaning                                                                                     |
| ------------------------ | ------------------------------------------------------------------------------------------- |
| `-e`, `--env <NAME>`     | Environment to operate on. Highest precedence.                                              |
| `-p`, `--project <PATH>` | Project directory or manifest path, instead of discovery.                                   |
| `--json`                 | Machine-readable output. Never contains values unless the command explicitly requests them. |
| `--no-color`             | Disable colour. `NO_COLOR` in the environment does the same.                                |
| `-q`, `--quiet`          | Suppress informational output. Warnings and errors still print.                             |

## Exit codes

| Code | Meaning                                           |
| ---- | ------------------------------------------------- |
| 0    | Success                                           |
| 1    | Generic failure                                   |
| 2    | Usage error (bad arguments)                       |
| 3    | Required secrets missing                          |
| 4    | Credential store unavailable                      |
| 5    | Manifest missing, invalid, or newer than this CLI |
| 6    | Bundle failed to decrypt or verify                |
| 7    | `doctor` found problems                           |
| 8    | Cancelled by the user                             |
| *    | For `run`: the child's exit code, or 128 + signal |

`status` returning 3 is the intended build-tooling hook:

```bash
kivro status --quiet || exit 1
```

## Commands

### `kivro init [--name NAME] [--default-env NAME] [--force]`

Creates `.kivro.toml` in the current (or `--project`) directory. Derives the
project name from the directory unless `--name` is given. Never creates a `.env`,
never generates values, and refuses to overwrite an existing manifest without
`--force`.

### `kivro set NAME [--stdin] [--no-confirm]`

Stores a value. Prompts twice without echoing by default. A value cannot be
passed as an argument — that is what puts secrets in shell history and `ps`.

```bash
kivro set AUTH0_CLIENT_SECRET            # interactive
echo "$SECRET" | kivro set AUTH0_CLIENT_SECRET --stdin   # CI
```

`--stdin` reads to EOF and strips exactly one trailing newline, so multi-line
values such as private keys survive intact.

Warns when the name is not declared in the manifest.

### `kivro get NAME [--show]`

Without `--show`, reports presence and length only. With `--show`, writes the
value to stdout and warns if stdout is a terminal.

### `kivro list [--all]`

Names and presence. Never values. `--all` includes stored secrets the manifest
does not declare.

### `kivro remove NAME [--yes]`

Deletes a value after confirmation.

### `kivro status`

Required/optional/undeclared report. Exit code 3 when required secrets are
missing.

```text
infinity-launcher / dev

Required secrets:

  ✓ AUTH0_CLIENT_ID
  ✗ AUTH0_CLIENT_SECRET
  ✓ DATABASE_URL

1 secret missing.

Run:
    kivro set AUTH0_CLIENT_SECRET
```

### `kivro doctor [--fix-gitignore]`

Checks: manifest found, manifest valid, project identity, CLI version
compatibility, unrecognised manifest keys, keyring availability, environment
resolution, required secrets present, deprecated secrets still stored, stray
`.env`, `.gitignore` coverage, stray bundles.

Exit code 7 on failures; warnings alone exit 0. `--fix-gitignore` appends
recommended entries after confirmation — never silently.

### `kivro run [--no-inherit] -- COMMAND...`

Loads the environment's secrets (failing before the child starts if a required
one is missing), spawns the command, and propagates its exit status. Never writes
a `.env`.

```bash
kivro run -- cargo run
kivro run --env staging -- npm run dev
kivro run --no-inherit -- ./deploy.sh
```

`--no-inherit` starts the child with only the project's secrets plus a minimal
passthrough (`PATH`, `HOME`, locale, and the Windows equivalents).

Ctrl-C is ignored in the parent so the child receives it directly and its exit
status is still propagated.

### `kivro import [PATH] [--force] [--delete-source]`

Parses a `.env` (default `.env`), stores each valid entry, and reports keys it
had to skip. Existing values are kept unless `--force`. The source file is never
deleted automatically; `--delete-source` still asks.

### `kivro export [-o PATH] [--force] [--yes]`

Writes a `.env`. Warns, asks for confirmation, refuses to overwrite without
`--force`, and creates the file `0600`. See the warning in
[SECURITY.md](SECURITY.md#kivro-export).

### `kivro share [-o PATH] [--recipient KEY]... [--all] [--hint-names] [--force]`

Creates an encrypted bundle, by default at
`<project>.<environment>.kivro`. Passphrase mode by default; `--recipient`
takes one or more `age1…` public keys instead. `--all` includes stored secrets
the manifest does not declare. `--hint-names` records the variable names in the
file's unencrypted header.

### `kivro accept PATH [--identity PATH] [--force]`

Decrypts a bundle into the credential store. Verifies that the bundle's
authenticated project matches the local manifest. Existing values are kept
unless `--force`.

### `kivro sync [--apply]`

Compares the manifest, the local store and the configured `[sync]` source.
Reports what is present, missing, and fetchable. Writes nothing without
`--apply`.

## Environment variables

| Variable           | Meaning                                                                    |
| ------------------ | -------------------------------------------------------------------------- |
| `KIVRO_ENV`        | Environment to use, below `--env` in precedence.                           |
| `KIVRO_STORE`      | `keyring` (default), `file`, or `memory`.                                  |
| `KIVRO_STORE_FILE` | Path for `KIVRO_STORE=file`.                                               |
| `KIVRO_CONFIG_DIR` | Override the configuration directory.                                      |
| `KIVRO_PASSPHRASE` | Bundle passphrase for non-interactive use. See [SECURITY.md](SECURITY.md). |
| `NO_COLOR`         | Disable colour.                                                            |

## Global configuration

`~/.config/kivro/config.toml` (platform-appropriate; `KIVRO_CONFIG_DIR`
overrides). Non-secret settings only.

```toml
[defaults]
environment = "dev"      # used only when the manifest declares no default

[ui]
color = true

[storage]
namespace = "infinity-secrets"   # first level of the storage namespace
```
