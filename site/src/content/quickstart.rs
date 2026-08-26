//! `/docs/quickstart` — an end-to-end walkthrough of a real project.

use crate::content::kit::*;
use crate::nav;
use crate::ui::prelude::*;

pub fn doc() -> Doc {
    Doc::new("quickstart", "Quick start", "Guide", "Getting started")
        .tagline(
            "Ten minutes from an empty directory to an application running with its secrets \
             injected — then the two things you will actually do next: migrate an existing .env, \
             and get a colleague set up.",
        )
        .tags(["init → set → run", ".env migration", "team sharing"])
        .section(
            DocSection::new("create", "Create the manifest", create)
                .numbered("1.0")
                .summary("`kivro init` writes the one file you commit."),
        )
        .section(
            DocSection::new("declare", "Declare what the project needs", declare)
                .numbered("1.1")
                .summary("Names, requiredness, descriptions — no values, ever."),
        )
        .section(
            DocSection::new("store", "Store the values", store)
                .numbered("1.2")
                .summary("Prompted entry, or piped in for automation."),
        )
        .section(
            DocSection::new("check", "Check what is missing", check)
                .numbered("1.3")
                .summary("`status` is both a human report and a build gate."),
        )
        .section(
            DocSection::new("run", "Run the application", run_section)
                .numbered("1.4")
                .summary("The secrets exist for exactly as long as the child process does."),
        )
        .section(
            DocSection::new("migrate", "Migrate an existing .env", migrate)
                .numbered("2.0")
                .summary("Import, verify, delete the file, then stop git from ever taking it back."),
        )
        .section(
            DocSection::new("colleague", "Set up a colleague", colleague)
                .numbered("2.1")
                .summary("An encrypted bundle, and a passphrase sent by another route."),
        )
        .section(
            DocSection::new("environments", "Add a second environment", environments)
                .numbered("2.2")
                .summary("Staging and production layer over the base declarations."),
        )
        .section(
            DocSection::new("wire-in", "Wire it into the project", wire_in)
                .numbered("2.3")
                .summary("npm scripts, Makefiles, CI — so nobody has to remember the wrapper."),
        )
}

fn create() -> impl IntoView {
    view! {
        <Lead>
            "Run this in the root of the project, next to " <InlineCode>".git"</InlineCode> "."
        </Lead>
        <CommandLine command="kivro init" />

        <P>
            "The project name is derived from the directory name, sanitised to the allowed "
            "character set. Override it with " <InlineCode>"--name"</InlineCode>
            ", and the starting environment with " <InlineCode>"--default-env"</InlineCode>
            " (it defaults to " <InlineCode>"dev"</InlineCode> ")."
        </P>

        <CodeBlock
            language="toml"
            title=".kivro.toml — as written by init"
            code=r#"
                # Managed by `kivro`. This file is safe to commit.
                # It declares WHICH secrets this project needs, never their values.

                [meta]
                format = 1

                [project]
                name = "launcher"

                [environment]
                default = "dev"

                # Variables that apply to every environment.
                [variables]
                # DATABASE_URL = { required = true, description = "Primary Postgres DSN" }

                # Per-environment declarations override the ones above.
                # [environments.dev]
                # DATABASE_URL = { required = true }
            "#
        />

        <Callout tone=Tone::Warning title="the project name is part of the address">
            "Stored secrets are addressed by "
            <InlineCode>"namespace:project:environment"</InlineCode>
            ". Renaming the project in the manifest later does not move the values — it points at "
            "a different address, and the old ones become unreachable through kivro. Pick the name "
            "once."
        </Callout>
    }
}

fn declare() -> impl IntoView {
    view! {
        <P>
            "Every variable the project reads gets a line. This is the part a reviewer looks at, "
            "so make it say something."
        </P>

        <CodeBlock
            language="toml"
            title=".kivro.toml"
            line_numbers=true
            highlight_lines=vec![4, 5]
            code=r#"
                [variables]
                DATABASE_URL        = { required = true, description = "Primary Postgres DSN" }
                AUTH0_CLIENT_ID     = { required = true }
                AUTH0_CLIENT_SECRET = { required = true }
                SENTRY_DSN          = { required = false, description = "Error reporting; optional locally" }
                LEGACY_API_TOKEN    = { required = false, deprecated = true }
            "#
        />

        {spec_table(
            &["Key", "Default", "Effect"],
            &[
                &["required", "true", "status and run fail when no value is stored"],
                &["description", "—", "shown in diagnostics; write it for the next person"],
                &["example", "—", "a NON-secret example value, for documentation only"],
                &["deprecated", "false", "doctor warns while a value is still stored"],
            ],
        )}

        <P>
            "Two shorthands exist for the common cases: "
            <InlineCode>"FEATURE_FLAG = true"</InlineCode> " means "
            <InlineCode>"{ required = true }"</InlineCode> ", and "
            <InlineCode>"LEGACY_TOKEN = false"</InlineCode> " means "
            <InlineCode>"{ required = false }"</InlineCode>
            ". Variable names must match " <InlineCode>"[A-Z_][A-Z0-9_]*"</InlineCode>
            " — the full grammar is on the "
            <DocLink to=nav::doc_path("manifest")>"manifest page"</DocLink> "."
        </P>
    }
}

fn store() -> impl IntoView {
    view! {
        <P>
            "Values are prompted for, twice, without echo. They cannot be passed as an argument: "
            "there is no " <InlineCode>"kivro set NAME=value"</InlineCode>
            " form, deliberately, because that is what puts credentials into shell history."
        </P>

        <CodeBlock
            language="bash"
            code=r#"
                kivro set DATABASE_URL
                kivro set AUTH0_CLIENT_ID
                kivro set AUTH0_CLIENT_SECRET
            "#
        />

        <H3>"Non-interactively"</H3>
        <P>
            <InlineCode>"--stdin"</InlineCode>
            " reads to EOF and strips exactly one trailing newline, so multi-line values such as "
            "PEM private keys survive intact."
        </P>
        <CodeBlock
            language="bash"
            code=r#"
                printf '%s' "$DATABASE_URL" | kivro set DATABASE_URL --stdin
                cat service-account.pem | kivro set GCP_PRIVATE_KEY --stdin
            "#
        />

        <Callout tone=Tone::Info title="undeclared names still work">
            "Setting a name the manifest does not declare warns but succeeds — sometimes you need "
            "a value before the manifest change lands. " <InlineCode>"kivro list --all"</InlineCode>
            " and " <InlineCode>"status"</InlineCode>
            " both report stored-but-undeclared names so they do not become invisible."
        </Callout>
    }
}

fn check() -> impl IntoView {
    view! {
        <CommandLine command="kivro status" />

        <CodeBlock
            language="text"
            title="output"
            code=r#"
                launcher / dev

                Required secrets:

                  ✓ AUTH0_CLIENT_ID
                  ✗ AUTH0_CLIENT_SECRET
                  ✓ DATABASE_URL

                Optional secrets:

                  - SENTRY_DSN

                1 secret missing.

                Run:
                    kivro set AUTH0_CLIENT_SECRET
            "#
        />

        <P>
            "The exit code is the machine-readable half: " <InlineCode>"0"</InlineCode>
            " when every required secret is present, " <InlineCode>"3"</InlineCode>
            " when one is not. That is the intended hook for build tooling, and it composes with "
            "anything that understands a failing command."
        </P>

        <CodeBlock
            language="bash"
            code=r#"
                kivro status --quiet || exit 1     # gate a script
                kivro status --json                # { "satisfied": false, "missing": [...] }
            "#
        />

        <P>
            "For a wider check — keyring reachable, manifest valid, no stray "
            <InlineCode>".env"</InlineCode> ", " <InlineCode>".gitignore"</InlineCode>
            " covering what it should — run " <InlineCode>"kivro doctor"</InlineCode> "."
        </P>
    }
}

fn run_section() -> impl IntoView {
    view! {
        <P>
            "The secrets are loaded, the child is spawned with them in its environment, and its "
            "exit status is propagated. Nothing is written to disk at any point."
        </P>

        <CodeBlock
            language="bash"
            code=r#"
                kivro run -- cargo run
                kivro run -- npm run dev
                kivro run --env staging -- ./target/release/launcher
                kivro run --no-inherit -- ./deploy.sh
            "#
        />

        <P>
            "Everything after " <InlineCode>"--"</InlineCode>
            " belongs to the child, including its own flags. If a required secret is missing, the "
            "child is never started: " <InlineCode>"run"</InlineCode>
            " fails first, with the list of names and the commands that would fix it."
        </P>

        {definitions(
            &[
                (
                    "--no-inherit",
                    "start the child with only the project secrets plus a minimal passthrough \
                     (PATH, HOME, locale, and the Windows equivalents)",
                ),
                (
                    "Ctrl-C",
                    "ignored by the parent so the child receives it directly; its exit status is \
                     still propagated",
                ),
                (
                    "exit status",
                    "the child's own code, or 128 + signal on Unix when it was killed by one",
                ),
            ],
        )}

        <Callout tone=Tone::Info title="docker compose">
            <InlineCode>"kivro run -- docker compose up"</InlineCode>
            " gives the secrets to the docker client, not to the containers. Forward them "
            "explicitly with " <InlineCode>"environment: { DATABASE_URL: ${DATABASE_URL} }"</InlineCode>
            " in the compose file — see the "
            <DocLink to=nav::doc_path("integration")>"integration page"</DocLink> "."
        </Callout>
    }
}

fn migrate() -> impl IntoView {
    view! {
        <Lead>"Four commands, in this order."</Lead>
        <CodeBlock
            language="bash"
            line_numbers=true
            code=r#"
                kivro import .env       # parses and stores every valid entry
                kivro status            # confirm everything landed
                rm .env                 # the file is never deleted for you
                kivro doctor --fix-gitignore
            "#
        />

        <P>
            "The parser accepts what a " <InlineCode>".env"</InlineCode>
            " file normally contains: comments, blank lines, an optional "
            <InlineCode>"export "</InlineCode>
            " prefix, and single- or double-quoted values. Keys that are not valid secret names "
            "are reported as skipped rather than silently dropped, and existing stored values are "
            "kept unless you pass " <InlineCode>"--force"</InlineCode> "."
        </P>

        <Callout tone=Tone::Warning title="the file is yours to delete">
            <InlineCode>"import"</InlineCode> " never removes the source. "
            <InlineCode>"--delete-source"</InlineCode>
            " asks before it does. Until the file is gone, it is still a plaintext credential "
            "sitting in your working tree, and " <InlineCode>"doctor"</InlineCode>
            " will keep saying so."
        </Callout>

        <P>
            "The " <InlineCode>"--fix-gitignore"</InlineCode> " step appends "
            <InlineCode>".env"</InlineCode> ", " <InlineCode>".env.*"</InlineCode> " and "
            <InlineCode>"*.kivro"</InlineCode>
            " after confirmation — never silently. If the file was already committed at some "
            "point, rotate those credentials: they are in the history whatever you do to the "
            "working tree now."
        </P>
    }
}

fn colleague() -> impl IntoView {
    view! {
        <SideBySide
            left=|| {
                view! {
                    <Card title="On your machine" eyebrow="sender" icon=icons::COPY>
                        <CodeBlock
                            language="bash"
                            dense=true
                            code=r#"
                                kivro share
                                # writes launcher.dev.kivro
                                # prompts for a passphrase
                            "#
                        />
                    </Card>
                }
            }
            right=|| {
                view! {
                    <Card
                        title="On theirs"
                        eyebrow="recipient"
                        icon=icons::DOWNLOAD
                        tone=Tone::Info
                    >
                        <CodeBlock
                            language="bash"
                            dense=true
                            code=r#"
                                kivro accept ./launcher.dev.kivro
                                kivro status
                                rm ./launcher.dev.kivro
                            "#
                        />
                    </Card>
                }
            }
        />

        <P>
            "The bundle is a single text file encrypted with "
            <Link href="https://age-encryption.org/v1" external=true>"age"</Link>
            ". It is safe to send over any channel — email, chat, a shared drive. The passphrase "
            "is not: send it a different way, or the encryption bought you nothing."
        </P>

        <P>
            "For anything automated, use recipients instead of a passphrase. "
            <InlineCode>"--recipient"</InlineCode>
            " takes one or more age public keys, and the recipient decrypts with their identity "
            "file:"
        </P>
        <CodeBlock
            language="bash"
            code=r#"
                kivro share --recipient age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p
                kivro accept ./launcher.dev.kivro --identity ~/.config/age/keys.txt
            "#
        />

        <Callout tone=Tone::Accent title="accept verifies the project">
            "The authenticated payload carries the project and environment it was created for, and "
            <InlineCode>"accept"</InlineCode>
            " compares that against your local manifest. A bundle from another project is refused "
            "even if its filename and its unencrypted hint both claim otherwise."
        </Callout>
    }
}

fn environments() -> impl IntoView {
    view! {
        <P>
            "Environments are separate storage scopes. "
            <InlineCode>"DATABASE_URL"</InlineCode> " in " <InlineCode>"dev"</InlineCode>
            " and " <InlineCode>"DATABASE_URL"</InlineCode> " in "
            <InlineCode>"production"</InlineCode>
            " are unrelated entries that cannot see each other."
        </P>

        <CodeBlock
            language="toml"
            title=".kivro.toml"
            code=r#"
                [environment]
                default = "dev"
                list    = ["dev", "staging", "production"]
                strict  = true

                [variables]
                DATABASE_URL = { required = true }
                SENTRY_DSN   = { required = false }

                # Layered over [variables]: SENTRY_DSN becomes mandatory in production,
                # and two more variables appear that dev never needs.
                [environments.production]
                SENTRY_DSN    = { required = true }
                S3_ACCESS_KEY = { required = true }
                S3_SECRET_KEY = { required = true }
            "#
        />

        <CodeBlock
            language="bash"
            code=r#"
                kivro status --env production
                kivro set S3_ACCESS_KEY --env production
                KIVRO_ENV=staging kivro run -- ./smoke-test.sh
            "#
        />

        <P>
            "Selection order is " <InlineCode>"--env"</InlineCode> ", then "
            <InlineCode>"KIVRO_ENV"</InlineCode> ", then the manifest default, then the global "
            "config default. Under " <InlineCode>"strict"</InlineCode>
            " (the default) an environment the manifest does not declare is rejected, so "
            <InlineCode>"--env prod"</InlineCode> " on a project that spells it "
            <InlineCode>"production"</InlineCode>
            " fails loudly instead of resolving to an empty set of secrets."
        </P>
    }
}

fn wire_in() -> impl IntoView {
    view! {
        <P>
            "The wrapper only helps if nobody has to remember it. Put it in whatever your project "
            "already uses to start things."
        </P>

        <CodeBlock
            language="json"
            title="package.json"
            code=r#"
                {
                  "scripts": {
                    "dev": "kivro run -- vite dev",
                    "test": "kivro run -- vitest",
                    "preinstall": "kivro status --quiet"
                  }
                }
            "#
        />

        <CodeBlock
            language="bash"
            title="Makefile"
            code=r#"
                RUN := kivro run --

                check:
                	@kivro status --quiet

                dev: check
                	$(RUN) cargo watch -x run
            "#
        />

        <P>
            "Language-specific patterns, CI recipes and the anti-patterns worth knowing about are "
            "on the " <DocLink to=nav::doc_path("integration")>"integration page"</DocLink> "."
        </P>
    }
}
