//! `/docs/integration` — getting kivro into an existing project.

use crate::content::kit::*;
use crate::nav;
use crate::ui::prelude::*;

pub fn doc() -> Doc {
    Doc::new("integration", "Integration", "Guide", "Guides")
        .tagline(
            "Three ways in, in order of preference: wrap the command and change nothing, use the \
             library from Rust, or — only when a tool leaves you no choice — export a .env and \
             delete it afterwards.",
        )
        .tags(["Rust", "Node", "CI", "Docker"])
        .section(
            DocSection::new("three-ways", "Three ways in", three_ways)
                .numbered("1.0")
                .summary("Pick the least invasive one that works."),
        )
        .section(
            DocSection::new("rust", "Rust", rust)
                .numbered("1.1")
                .summary("Wrapping, the library, spawning children, and build-time checks."),
        )
        .section(
            DocSection::new("node", "Node and TypeScript", node)
                .numbered("1.2")
                .summary("npm scripts, and a readiness gate that reads --json."),
        )
        .section(
            DocSection::new("generic", "Any other runtime", generic)
                .numbered("1.3")
                .summary("The interface is the process environment, so everything works."),
        )
        .section(
            DocSection::new("make", "Make and task runners", make)
                .numbered("2.0")
                .summary("One variable, one gate, and nobody has to remember the wrapper."),
        )
        .section(
            DocSection::new("ci", "Continuous integration", ci)
                .numbered("2.1")
                .summary("Populating a store on a runner, and what to use when there is no keyring."),
        )
        .section(
            DocSection::new("docker", "Docker and Compose", docker)
                .numbered("2.2")
                .summary("The one place where wrapping does not do what it looks like it does."),
        )
        .section(
            DocSection::new("anti-patterns", "Anti-patterns", anti_patterns)
                .numbered("3.0")
                .summary("Five habits that put the plaintext back, and what to do instead."),
        )
}

fn three_ways() -> impl IntoView {
    view! {
        <div class="grid gap-4 md:grid-cols-3">
            <Card title="Wrap the command" eyebrow="preferred" icon=icons::TERMINAL>
                <InlineCode>"kivro run -- <your command>"</InlineCode>
                ". Nothing in the application changes; it reads environment variables exactly as "
                "it does today."
            </Card>
            <Card title="Use the library" eyebrow="Rust only" icon=icons::HASH tone=Tone::Info>
                "Load secrets directly and skip the subprocess. Useful when the program is already "
                "Rust and you want typed access."
            </Card>
            <Card title="Export a .env" eyebrow="last resort" icon=icons::TRIANGLE_ALERT tone=Tone::Warning>
                "Only for tools that genuinely cannot read anything else. Explicit, confirmed, and "
                "deleted afterwards."
            </Card>
        </div>

        <P>
            "The first option is not a compromise. Environment variables are the interface every "
            "runtime already agrees on, which is why wrapping needs no library, no plugin, and no "
            "change to the code being wrapped."
        </P>
    }
}

fn rust() -> impl IntoView {
    view! {
        <H3>"Wrapping — nothing to change"</H3>
        <CodeBlock
            language="bash"
            code=r#"
                kivro run -- cargo run
                kivro run --env staging -- ./target/release/launcher
            "#
        />
        <P>
            <InlineCode>"std::env::var(\"DATABASE_URL\")"</InlineCode> " keeps working."
        </P>

        <H3>"Using the library"</H3>
        <CodeBlock
            language="rust"
            code=r#"
                use kivro::Project;

                fn main() -> Result<(), Box<dyn std::error::Error>> {
                    let project = Project::discover()?;              // walks up for .kivro.toml
                    let env = project.resolve_environment(None)?;    // --env / KIVRO_ENV / default
                    let secrets = env.load()?;                       // fails if a required one is missing

                    let database_url = secrets.get("DATABASE_URL")?; // &SecretString
                    connect(database_url.expose_secret())?;
                    Ok(())
                }
            "#
        />
        <P>
            <InlineCode>"SecretString"</InlineCode> " has no " <InlineCode>"Display"</InlineCode>
            " and no " <InlineCode>"Serialize"</InlineCode> ", and its "
            <InlineCode>"Debug"</InlineCode> " is redacted, so it cannot be logged by accident. "
            <InlineCode>"expose_secret()"</InlineCode>
            " is deliberately verbose: every call site is greppable."
        </P>

        <H3>"Spawning a child yourself"</H3>
        <CodeBlock
            language="rust"
            code=r#"
                let secrets = Project::discover()?.environment("dev")?.load()?;

                std::process::Command::new("cargo")
                    .args(["run"])
                    .envs(secrets.environment())
                    .spawn()?;

                // Or with signal handling and exit-code propagation already done:
                use kivro::run::{run, RunOptions};
                let code = run("cargo", &["run".into()], &secrets, &RunOptions::default())?;
                std::process::exit(code);
            "#
        />

        <H3>"Checking readiness at build time"</H3>
        <CodeBlock
            language="rust"
            title="build.rs"
            code=r#"
                let status = Project::discover()?.environment("dev")?.status()?;
                if !status.is_satisfied() {
                    let missing: Vec<_> = status
                        .missing_required()
                        .iter()
                        .map(|e| e.name.to_string())
                        .collect();
                    panic!("missing secrets: {}", missing.join(", "));
                }
            "#
        />
        <P>
            "The full API surface is on the "
            <DocLink to=nav::doc_path("library")>"library page"</DocLink> "."
        </P>
    }
}

fn node() -> impl IntoView {
    view! {
        <H3>"Wrapping"</H3>
        <CodeBlock
            language="json"
            title="package.json"
            code=r#"
                {
                  "scripts": {
                    "dev": "kivro run -- vite dev",
                    "test": "kivro run --env dev -- vitest",
                    "deploy": "kivro run --env production -- node scripts/deploy.mjs"
                  }
                }
            "#
        />
        <P>
            <InlineCode>"process.env.DATABASE_URL"</InlineCode>
            " works as usual. Drop " <InlineCode>"dotenv"</InlineCode> " entirely — no "
            <InlineCode>"import 'dotenv/config'"</InlineCode> ", no "
            <InlineCode>".env"</InlineCode> " in the repository."
        </P>

        <H3>"Failing fast"</H3>
        <CodeBlock
            language="json"
            code=r#"
                { "scripts": { "preinstall": "kivro status --quiet" } }
            "#
        />
        <P>"Exit code 3 stops the build with a list of what to set."</P>

        <H3>"Reading the status programmatically"</H3>
        <CodeBlock
            language="ts"
            code=r#"
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
            "#
        />

        <Callout tone=Tone::Danger title="never build a config object from `get --show`">
            "Shelling out to " <InlineCode>"kivro get --show"</InlineCode>
            " puts plaintext through a pipe and into your process's logs the first time someone "
            "adds a debug print. Use " <InlineCode>"kivro run"</InlineCode> " and read "
            <InlineCode>"process.env"</InlineCode> "."
        </Callout>
    }
}

fn generic() -> impl IntoView {
    view! {
        <P>"Any language, any runtime — the interface is the process environment."</P>
        <CodeBlock
            language="bash"
            code=r#"
                kivro run -- python manage.py runserver
                kivro run -- go run ./cmd/server
                kivro run -- bundle exec rails server
                kivro run -- dotnet run
                kivro run -- make deploy
            "#
        />
        <P>
            "There is no SDK to install and no client library to keep up to date, because there "
            "is nothing for one to do. If your runtime can read an environment variable, it is "
            "already integrated."
        </P>
    }
}

fn make() -> impl IntoView {
    view! {
        <CodeBlock
            language="bash"
            title="Makefile"
            code=r#"
                RUN := kivro run --

                .PHONY: check dev test release migrate

                check:
                	@kivro status --quiet

                dev: check
                	$(RUN) cargo watch -x run

                test: check
                	$(RUN) cargo test

                # A different environment for one target.
                migrate:
                	kivro run --env production -- ./scripts/migrate.sh
            "#
        />

        <P>
            <InlineCode>"kivro status"</InlineCode>
            " exits 3 when a required secret is missing, so it composes with make's own failure "
            "handling without any parsing. The same shape works in "
            <InlineCode>"just"</InlineCode> ", " <InlineCode>"task"</InlineCode> ", npm scripts, "
            "or a shell function."
        </P>
    }
}

fn ci() -> impl IntoView {
    view! {
        <CodeBlock
            language="text"
            title=".github/workflows/test.yml"
            code=r#"
                jobs:
                  test:
                    runs-on: ubuntu-latest
                    env:
                      # Linux runners have no Secret Service. Read the security page first:
                      # this is plaintext on disk, scoped to the job's temp directory.
                      KIVRO_STORE: file
                      KIVRO_STORE_FILE: ${{ runner.temp }}/kivro-store.json
                    steps:
                      - uses: actions/checkout@v4

                      - name: Install kivro
                        run: cargo install --git https://github.com/CameronGr/kivro kivro-cli

                      - name: Populate the store
                        run: |
                          printf '%s' "${{ secrets.DATABASE_URL }}" | kivro set DATABASE_URL --stdin
                          printf '%s' "${{ secrets.AUTH0_CLIENT_ID }}" | kivro set AUTH0_CLIENT_ID --stdin

                      # Exit code 3 fails the job with a list of what is missing.
                      - name: Verify
                        run: kivro status

                      - name: Test
                        run: kivro run -- cargo test
            "#
        />

        <Callout tone=Tone::Warning title="your CI provider is still the source of truth">
            "kivro does not replace CI secret storage. What it adds is one declaration of what a "
            "job needs — the manifest, which is the same file developers use — and one failure "
            "mode when something is missing, instead of a job that runs for nine minutes and then "
            "fails on a null connection string."
        </Callout>

        <P>
            "For bundle-based flows in CI, prefer " <InlineCode>"--recipient"</InlineCode>
            " with an age public key over " <InlineCode>"KIVRO_PASSPHRASE"</InlineCode>
            ": a passphrase in the environment is visible to every child process."
        </P>
    }
}

fn docker() -> impl IntoView {
    view! {
        <Callout tone=Tone::Warning title="wrapping gives secrets to the client, not the containers">
            <InlineCode>"kivro run -- docker compose up"</InlineCode>
            " populates the environment of the " <InlineCode>"docker"</InlineCode>
            " command. Containers do not inherit it. Forward the variables explicitly."
        </Callout>

        <CodeBlock
            language="text"
            title="compose.yaml"
            code=r#"
                services:
                  app:
                    image: launcher
                    environment:
                      DATABASE_URL: ${DATABASE_URL}
                      AUTH0_CLIENT_ID: ${AUTH0_CLIENT_ID}
            "#
        />

        <P>
            "Compose interpolates " <InlineCode>"${DATABASE_URL}"</InlineCode>
            " from its own environment, which " <InlineCode>"kivro run"</InlineCode>
            " has populated. Never bake a secret into an image layer: layers are cached, pushed, "
            "and readable by anyone who can pull the image."
        </P>
    }
}

fn anti_patterns() -> impl IntoView {
    view! {
        {prose_table(
            &["Don't", "Do", "Because"],
            &[
                &[
                    "export DB=$(kivro get DB --show)",
                    "kivro run -- <cmd>",
                    "The value lands in your shell's environment, its history, and every process you start afterwards",
                ],
                &[
                    "kivro export in a script",
                    "kivro run -- <cmd>",
                    "It writes the plaintext file this tool exists to remove",
                ],
                &[
                    "Committing a bundle for the team",
                    "[sync] pointing at a shared location, or share per developer",
                    "A committed bundle is one leaked passphrase away from being a committed .env",
                ],
                &[
                    "kivro set NAME=value",
                    "kivro set NAME, or --stdin",
                    "Not supported, by design — arguments are visible in ps and shell history",
                ],
                &[
                    "Reading .env as a fallback in your app",
                    "Let run fail; exit code 3 says exactly what is missing",
                    "A fallback path means the plaintext file has to keep existing",
                ],
            ],
        )}

        <P>
            "Each of these is a way of turning a managed secret back into an unmanaged one. They "
            "are all reasonable-looking at 5pm on a Friday, which is exactly why they are worth "
            "writing down."
        </P>
    }
}
