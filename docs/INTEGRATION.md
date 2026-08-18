# Integrating `kivro`

Three ways to get kivro into an application, in order of preference.

1. **Wrap the command** — `kivro run -- <your command>`. Nothing in your
   application changes; it reads environment variables exactly as it does today.
2. **Use the library** — Rust programs can load secrets directly, skipping the
   subprocess.
3. **Export a `.env`** — only for tools that genuinely cannot read anything else.

## Rust

### Wrapping (nothing to change)

```bash
kivro run -- cargo run
kivro run --env staging -- ./target/release/launcher
```

`std::env::var("DATABASE_URL")` keeps working.

### Using the library

```toml
[dependencies]
kivro = { git = "https://github.com/CameronGr/kivro" }
```

```rust
use kivro::Project;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let project = Project::discover()?;              // walks up for .kivro.toml
    let env = project.resolve_environment(None)?;    // --env / KIVRO_ENV / default
    let secrets = env.load()?;                       // fails if a required one is missing

    let database_url = secrets.get("DATABASE_URL")?; // &SecretString
    connect(database_url.expose_secret())?;

    Ok(())
}
```

`SecretString` has no `Display` and no `Serialize`, and its `Debug` is redacted,
so it cannot be logged by accident. `expose_secret()` is deliberately verbose:
every call site is greppable.

### Spawning a child yourself

```rust
let secrets = Project::discover()?.environment("dev")?.load()?;

std::process::Command::new("cargo")
    .args(["run"])
    .envs(secrets.environment())
    .spawn()?;
```

Or with signal handling and exit-code propagation already handled:

```rust
use kivro::run::{run, RunOptions};

let code = run("cargo", &["run".into()], &secrets, &RunOptions::default())?;
std::process::exit(code);
```

### Checking readiness in `build.rs` or a task runner

```rust
let status = Project::discover()?.environment("dev")?.status()?;
if !status.is_satisfied() {
    let missing: Vec<_> = status.missing_required().iter().map(|e| e.name.to_string()).collect();
    panic!("missing secrets: {}", missing.join(", "));
}
```

### Supplying your own store (tests)

```rust
use kivro::{config::Config, Project};
use kivro_core::MemoryStore;

let project = Project::new(manifest, Box::new(MemoryStore::new()), Config::default());
```

## Node / TypeScript

### Wrapping

`package.json`:

```json
{
  "scripts": {
    "dev": "kivro run -- vite dev",
    "test": "kivro run --env dev -- vitest",
    "deploy": "kivro run --env production -- node scripts/deploy.mjs"
  }
}
```

`process.env.DATABASE_URL` works as usual. Drop `dotenv` entirely — no
`import 'dotenv/config'`, no `.env` in the repository.

### Failing fast in CI

```json
{
  "scripts": {
    "preinstall": "kivro status --quiet"
  }
}
```

Exit code 3 stops the build with a list of what to set.

### Reading the status programmatically

```ts
import { execFileSync } from "node:child_process";

type Status = {
  project: string;
  environment: string;
  satisfied: boolean;
  missing: string[];
};

function secretsStatus(): Status {
  try {
    return JSON.parse(
      execFileSync("kivro", ["status", "--json"], { encoding: "utf8" }),
    );
  } catch (error: any) {
    // Exit code 3 still prints valid JSON on stdout.
    if (error.stdout) return JSON.parse(error.stdout);
    throw error;
  }
}

const status = secretsStatus();
if (!status.satisfied) {
  console.error(`Missing: ${status.missing.join(", ")}`);
  console.error(status.missing.map((n) => `  kivro set ${n}`).join("\n"));
  process.exit(1);
}
```

Never shell out to `kivro get --show` to build a config object: that puts
plaintext through a pipe and into your process's logs the first time someone
adds a debug print. Use `kivro run` and read `process.env`.

## Generic

Any language, any runtime — the interface is the process environment.

```bash
kivro run -- python manage.py runserver
kivro run -- go run ./cmd/server
kivro run -- docker compose up          # see the caveat below
kivro run -- make deploy
```

### Makefile

```make
RUN := kivro run --

.PHONY: check dev
check:
	@kivro status --quiet

dev: check
	$(RUN) cargo watch -x run
```

### CI

```yaml
- name: Load secrets
  run: |
    echo "${{ secrets.DATABASE_URL }}" | kivro set DATABASE_URL --stdin
    echo "${{ secrets.AUTH0_CLIENT_SECRET }}" | kivro set AUTH0_CLIENT_SECRET --stdin
- name: Verify
  run: kivro status
- name: Test
  run: kivro run -- cargo test
```

On a runner with no credential store, `KIVRO_STORE=file` with a path under the
job's temporary directory works — read
[SECURITY.md](SECURITY.md#the-insecure-file-store) first, and prefer your CI
provider's own secret storage as the source.

### Docker

`kivro run -- docker compose up` gives the secrets to the `docker` client, not
to the containers. Forward them explicitly:

```yaml
services:
  app:
    environment:
      DATABASE_URL: ${DATABASE_URL}
```

Compose interpolates from its own environment, which `kivro run` has populated.
Never bake secrets into an image layer.

### Shell session (last resort)

```bash
kivro run -- $SHELL
```

Starts an interactive shell with the secrets present. Convenient, and every
command you run in it inherits them — including anything that logs its
environment. Prefer wrapping the specific command.

## Anti-patterns

| Don't                                             | Do                                                               |
| ------------------------------------------------- | ---------------------------------------------------------------- |
| `export DB=$(kivro get DB --show)`                | `kivro run -- <cmd>`                                             |
| `kivro export` in a script                        | `kivro run -- <cmd>`                                             |
| Committing a bundle "for the team"                | `[sync]` pointing at a shared location, or `share` per developer |
| `kivro set NAME=value` (not supported, by design) | `kivro set NAME` or `--stdin`                                    |
| Reading `.env` as a fallback in your app          | Let `run` fail; exit code 3 says exactly what is missing         |
