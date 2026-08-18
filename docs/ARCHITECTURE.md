# Architecture

## Overview

```text
                    ┌─────────────────┐
                    │  kivro-cli      │  argument parsing, prompts, formatting,
                    │  (binary)       │  exit codes
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │   kivro         │  Project · Environment · SecretSet
                    │   (facade lib)  │  env resolution · .env · process running
                    └────┬───┬───┬────┘
             ┌───────────┘   │   └───────────┐
   ┌─────────▼──────┐ ┌──────▼──────┐ ┌──────▼──────────┐
   │ kivro-         │ │ kivro-      │ │ kivros-sync     │
   │ manifest       │ │ keyring     │ │                 │
   │ .kivro.toml    │ │ OS backends │ │ SyncSource      │
   └─────────┬──────┘ └──────┬──────┘ └──────┬──────────┘
             │               │               │
             │               │        ┌──────▼──────────┐
             │               │        │ kivro-crypto    │  age bundles
             │               │        └──────┬──────────┘
             └───────────────┴───────────────┘
                             │
                    ┌────────▼────────┐
                    │  kivro-core     │  SecretString · names · SecretStore port
                    │                 │  Error — depends on nothing
                    └─────────────────┘
```

Dependencies point downward only. `kivro-core` is a leaf: it defines the
domain types and the `SecretStore` _port_, and the adapters implement it. That
inversion is what lets tests run against `MemoryStore` while production runs
against the Windows Credential Manager with no code in between caring which.

## Crates

| Crate            | Responsibility                                                               | Notable dependencies         |
| ---------------- | ---------------------------------------------------------------------------- | ---------------------------- |
| `kivro-core`     | `SecretString`, validated names, `SecretStore` trait, `MemoryStore`, `Error` | `zeroize`, `thiserror`       |
| `kivro-manifest` | `.kivro.toml` parsing, discovery, environment resolution                     | `toml`                       |
| `kivro-keyring`  | OS credential store adapter, insecure file store for tests                   | `keyring`                    |
| `kivro-crypto`   | encrypted bundle format                                                      | `age`                        |
| `kivro-sync`     | `SyncSource` trait, bundle-directory source                                  | `kivro-crypto`               |
| `kivro`          | library facade: `Project`, `Environment`, `SecretSet`, `run`                 | all of the above             |
| `kivro-cli`      | the `kivro` binary                                                           | `clap`, `rpassword`, `ctrlc` |

### Why seven crates and not six

The original sketch put the facade in `kivro-core`. That cannot work: the
facade needs the manifest _and_ the keyring, and both of those need the core
types, so the dependency would be circular. Splitting the facade into its own
`kivro` crate keeps `kivro-core` a leaf and gives library consumers a single
crate to depend on.

## Storage model

```text
<app namespace>            infinity-secrets   (configurable)
  └── <project>              infinity-launcher
        └── <environment>      dev
              ├── DATABASE_URL
              ├── AUTH0_CLIENT_ID
              └── AUTH0_CLIENT_SECRET
```

Rendered into the OS credential store as one credential per secret:

```text
service = "infinity-secrets:infinity-launcher:dev"
user    = "DATABASE_URL"
```

Project, environment and secret names are validated at construction and cannot
contain `:`, so the rendering is injective — two different scopes can never
produce the same service string. `DATABASE_URL` in two projects, or in two
environments of one project, are unrelated entries.

### Enumeration

No credential store offers a portable "list everything under this service" call.
Enumeration is served by a per-scope index credential stored under the reserved
user name `__index`, holding a JSON array of _names only_. Secret names are
uppercase by validation, so `__index` cannot collide with one.

The index is a cache, not the source of truth. `get` never consults it, and
`status` probes manifest-declared names directly, so a lost or stale index
degrades enumeration and nothing else.

## Environment resolution

1. `--env` on the command line
2. the `KIVRO_ENV` environment variable
3. `[environment] default` in the manifest
4. `[defaults] environment` in the global configuration
5. otherwise an error

The manifest outranks the global configuration deliberately: a per-machine
preference must never silently change which environment a shared project runs
against. Under `strict` (the default) an environment the manifest does not
declare is rejected, so `--env prod` for a project that spells it `production`
fails loudly instead of resolving to an empty secret set.

## Error handling

One `Error` enum in `kivro-core`, with `thiserror`. Two rules:

- **No variant carries a secret value.** Keyring errors are mapped
  variant-by-variant rather than through `to_string()`, because
  `keyring::Error::BadEncoding` carries the raw stored bytes and `Ambiguous`
  carries whole credentials.
- **Errors carry their fix.** `Error::hint()` returns the command to run.
  `Error::kind()` returns a stable string for `--json`. The CLI maps variants to
  documented exit codes in `main.rs`.

```text
error: 2 required secret(s) missing for infinity-launcher/dev: AUTH0_CLIENT_SECRET, S3_ACCESS_KEY

run:
    kivro set AUTH0_CLIENT_SECRET
    kivro set S3_ACCESS_KEY
```

## Testing strategy

97 tests, none of which require a real credential store.

- **Unit tests** live beside the code: name validation and namespace injectivity,
  manifest parsing and layering, `.env` parsing and rendering, bundle sealing,
  opening, tampering and version rejection, sync planning, config loading.
- **Integration tests** (`crates/kivro-cli/tests/cli.rs`) drive the real binary
  through `assert_cmd`, covering every command, exit codes, JSON output and the
  full share → accept and import → export round trips.
- **Store doubles**: `MemoryStore` (in `kivro-core`) for library tests, and the
  file store (`KIVRO_STORE=file`) for binary tests, which is how the CLI is
  exercised in containers with no D-Bus session.

Several tests exist specifically to catch leaks rather than logic errors:
`Debug` output redaction, JSON output containing no values, `list` printing no
values, and bundle text not containing plaintext.

## Extension points

Adding a sync backend means implementing `SyncSource` and one arm in
`kivro_sync::from_config`. Nothing above that layer changes, and the bundle
format is unaffected — the format is the compatibility surface, the transport is
not. Adding a credential backend means implementing `SecretStore` and one arm in
`kivro_keyring::open`.

Deliberately not built yet, but designed for: team sync over HTTP or object
storage, secret rotation and versioning (the bundle payload is already
versioned), audit logging, shell and IDE integration, and secret references.
