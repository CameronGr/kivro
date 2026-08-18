# `.kivro.toml` specification

The manifest declares **what a project needs**. It is committed to source
control and must never contain a secret value.

## Example

```toml
[meta]
format = 1
min_cli_version = "0.1.0"

[project]
name = "infinity-launcher"

[environment]
default = "dev"
list = ["dev", "staging", "production"]
strict = true

[variables]
DATABASE_URL        = { required = true, description = "Primary Postgres DSN" }
AUTH0_CLIENT_ID     = { required = true }
AUTH0_CLIENT_SECRET = { required = true }
SENTRY_DSN          = { required = false }

[environments.production]
SENTRY_DSN    = { required = true }
S3_ACCESS_KEY = { required = true }

[sync]
kind = "file"
path = "team-secrets"
```

## Sections

### `[meta]`

| Key               | Type    | Default | Meaning                                                                             |
| ----------------- | ------- | ------- | ----------------------------------------------------------------------------------- |
| `format`          | integer | `1`     | Manifest format version. A value above the CLI's supported version is a hard error. |
| `min_cli_version` | string  | —       | Minimum CLI version; older CLIs refuse to run.                                      |

### `[project]`

| Key    | Type   | Required | Meaning                                                                                                              |
| ------ | ------ | -------- | -------------------------------------------------------------------------------------------------------------------- |
| `name` | string | yes      | Project identity. Also the second level of the storage namespace, so changing it makes existing secrets unreachable. |

Must match `[A-Za-z0-9][A-Za-z0-9._-]*`, at most 64 characters.

### `[environment]`

| Key       | Type             | Default | Meaning                                                                                                          |
| --------- | ---------------- | ------- | ---------------------------------------------------------------------------------------------------------------- |
| `default` | string           | —       | Environment used when nothing else selects one.                                                                  |
| `list`    | array of strings | —       | Closed set of valid environments. When present, `default` and every `[environments.*]` section must be a member. |
| `strict`  | boolean          | `true`  | Reject environments the manifest does not declare.                                                               |

Environment names must match `[A-Za-z0-9][A-Za-z0-9._-]*`, at most 32 characters.

### `[variables]`

Declarations that apply to every environment.

```toml
[variables]
DATABASE_URL = { required = true }
FEATURE_FLAG = true          # shorthand for { required = true }
LEGACY_TOKEN = false         # shorthand for { required = false }
```

Each declaration accepts:

| Key           | Type    | Default | Meaning                                      |
| ------------- | ------- | ------- | -------------------------------------------- |
| `required`    | boolean | `true`  | Whether `status` and `run` fail without it.  |
| `description` | string  | —       | Shown in diagnostics.                        |
| `example`     | string  | —       | A **non-secret** example value.              |
| `deprecated`  | boolean | `false` | `doctor` warns when a value is still stored. |

Variable names must match `[A-Z_][A-Z0-9_]*`, at most 128 characters.

### `[environments.<name>]`

Per-environment declarations, layered _over_ `[variables]` — a base declaration
stays in effect unless the environment overrides that exact name.

Two equivalent forms:

```toml
[environments.production]
SENTRY_DSN = { required = true }

# or, when you also want environment settings:
[environments.production]
description = "customer-facing"
[environments.production.variables]
SENTRY_DSN = { required = true }
```

Keys inside `[environments.<name>]` are disambiguated by case:
**UPPERCASE keys are variables, lowercase keys are settings.** Since variable
names are validated as uppercase, the two sets can never overlap, and new
settings keys can be added in future versions without shadowing anyone's
variable. Reserved settings keys today: `description`, `inherit`, `variables`.

### `[sync]`

Where `kivro sync` looks for secrets it is missing. Non-secret settings only.

| Key    | Type   | Meaning                                                                  |
| ------ | ------ | ------------------------------------------------------------------------ |
| `kind` | string | Backend discriminator. `file` is the only one in 0.1.                    |
| `path` | string | For `kind = "file"`: directory of bundles, relative to the project root. |

## Compatibility

The format is designed to evolve in three ways, in increasing order of severity:

1. **Additive keys** — unknown keys and unknown sections are _ignored_, and
   collected for `kivro doctor` to report as advisory warnings. An older CLI
   keeps working with a newer manifest whenever the addition is optional.
2. **`min_cli_version`** — for additions that are syntactically ignorable but
   semantically required. Older CLIs stop with a clear upgrade message rather
   than running with a partial understanding.
3. **`format`** — for changes that would make an old CLI misread the file.
   Anything above the supported version is refused outright.

Guarantees for future versions:

- `format = 1` manifests will keep parsing.
- Variable names will stay uppercase-only, so the case-based disambiguation
  above remains stable.
- The storage namespace derivation (`app:project:environment` + name) will not
  change without a format bump, because changing it would orphan stored secrets.

## What must never appear

Secret values, in any form — including "temporary" ones, examples that happen to
be real, and base64 of the above. `kivro doctor` cannot detect this for you;
review your manifest in code review like any other committed file.
