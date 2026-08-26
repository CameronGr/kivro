//! `/docs/overview` — what kivro is and the shape of the problem it solves.

use crate::content::kit::*;
use crate::nav;
use crate::ui::prelude::*;

pub fn doc() -> Doc {
    Doc::new("overview", "Overview", "Guide", "Getting started")
        .tagline(
            "kivro is a secret manager for project environment variables. The list of variables a \
             project needs lives in a file you commit; the values live in the operating system \
             credential store, and are injected straight into the process that needs them.",
        )
        .tags(["No .env in the repo", "OS credential store", "age bundles"])
        .section(
            DocSection::new("model", "The model", model)
                .numbered("1.0")
                .summary("Two halves: a committed manifest of names, and a keyring of values."),
        )
        .section(
            DocSection::new("why", "Why not just use .env", why)
                .numbered("1.1")
                .summary("What a plaintext file costs, and which of those costs kivro removes."),
        )
        .section(
            DocSection::new("lifecycle", "The lifecycle", lifecycle)
                .numbered("1.2")
                .summary("Declare, store, check, run — and how a second developer joins."),
        )
        .section(
            DocSection::new("scope", "What it is not", scope)
                .numbered("1.3")
                .summary("Deliberate limits, stated up front rather than discovered later."),
        )
        .section(
            DocSection::new("next", "Where to go next", next)
                .numbered("1.4")
                .summary("The rest of the documentation, in reading order."),
        )
}

fn model() -> impl IntoView {
    view! {
        <Lead>
            "A project needs " <InlineCode>"DATABASE_URL"</InlineCode>
            " to run. That is a fact about the project, and it belongs in the repository. The "
            "value of " <InlineCode>"DATABASE_URL"</InlineCode>
            " is a credential, and it belongs somewhere the repository cannot reach. kivro splits "
            "those two things apart and keeps them apart."
        </Lead>

        <CodeBlock
            language="text"
            title="the split"
            code=r#"
                .kivro.toml                       OS credential store
                (committed to git)                (never committed, never in the repo)

                  which variables exist       +     what their values are
                  which are required                one credential per secret
                  which environments                scoped to project + environment

                                     kivro run -- <your command>

                  the child process receives exactly those variables, and nothing else
            "#
        />

        <div class="grid gap-4 md:grid-cols-2">
            <Card title="The manifest" eyebrow="committed" icon=icons::FILE_TEXT>
                <InlineCode>".kivro.toml"</InlineCode>
                " declares names, whether each one is required, which environments exist, and "
                "optionally a description and a non-secret example. It is reviewed like any other "
                "file in the repository. It cannot hold a value."
            </Card>
            <Card
                title="The credential store"
                eyebrow="local to each machine"
                icon=icons::SHIELD_ALERT
                tone=Tone::Info
            >
                "Windows Credential Manager, macOS Keychain, or the Linux Secret Service, reached "
                "through one trait. Values are written and read one at a time, under a namespace "
                "derived from the project and environment names."
            </Card>
        </div>

        <Callout tone=Tone::Accent title="the whole idea">
            "Losing your laptop is a bad day. Pushing a commit that contains "
            <InlineCode>"AUTH0_CLIENT_SECRET"</InlineCode>
            " is a bad quarter. The manifest is safe to commit precisely because there is nothing "
            "in it worth stealing."
        </Callout>
    }
}

fn why() -> impl IntoView {
    view! {
        <P>
            "There is nothing wrong with a " <InlineCode>".env"</InlineCode>
            " file until it leaves your machine — and the ways it leaves are all boring. It gets "
            "committed on a branch nobody reviews. It gets pasted into a chat thread so a new "
            "starter can get running. It ends up in a screenshot. Each of those is a plaintext "
            "credential with no expiry and no audit trail."
        </P>

        {prose_table(
            &["Failure", "With a .env file", "With kivro"],
            &[
                &[
                    "Committed by accident",
                    "The values are in the file, so they are in the history forever",
                    "The committed file holds names only; there is nothing to leak",
                ],
                &[
                    "Shared with a colleague",
                    "Sent as plaintext over whatever channel is convenient",
                    "kivro share writes an age-encrypted bundle; the passphrase travels separately",
                ],
                &[
                    "Someone is missing a variable",
                    "The app starts, then fails at the first request or silently misbehaves",
                    "kivro run refuses to start and names what is missing; status exits 3",
                ],
                &[
                    "A variable is added to the project",
                    "Everyone finds out when their branch breaks",
                    "The manifest change arrives with the code change, and status reports it",
                ],
                &[
                    "Left on disk",
                    "Readable by anything that can read your home directory",
                    "Held by the OS credential store, protected the way the platform protects it",
                ],
            ],
        )}

        <Callout tone=Tone::Info title="who this is for">
            "Small teams. A hosted secret manager is the right answer once you have platform "
            "engineers to run it; before that, most teams have a shared password vault and a habit "
            "of pasting files. kivro targets that gap: no server, no account, and no sync service "
            "that has to stay up for your build to work."
        </Callout>
    }
}

fn lifecycle() -> impl IntoView {
    view! {
        <H3>"The developer who starts the project"</H3>
        <CodeBlock
            language="bash"
            code=r#"
                kivro init                 # writes .kivro.toml -- commit it
                $EDITOR .kivro.toml        # declare the variables the project needs
                kivro set DATABASE_URL     # prompts twice, stores in the OS keyring
                kivro status               # what is present, what is missing
                kivro run -- cargo run     # start the app with the values injected
            "#
        />

        <H3>"The developer who joins on Tuesday"</H3>
        <CodeBlock
            language="bash"
            code=r#"
                git clone git@github.com:acme/launcher.git && cd launcher
                kivro status                        # exit code 3: required secrets missing
                kivro accept ./launcher.dev.kivro   # a bundle a colleague sent, or...
                kivro set DATABASE_URL              # ...set them by hand
                kivro run -- cargo run
            "#
        />

        <P>
            "Nothing in the application changes. " <InlineCode>"std::env::var"</InlineCode> ", "
            <InlineCode>"process.env"</InlineCode> ", " <InlineCode>"os.environ"</InlineCode>
            " all keep working, because the values arrive the way they always did: in the process "
            "environment. What changes is where they came from and who else can read them."
        </P>

        <Callout tone=Tone::Warning title="the one rule">
            "A value is never passed as a command-line argument. Not to "
            <InlineCode>"kivro set"</InlineCode>
            ", not to the child process. Arguments land in shell history and in "
            <InlineCode>"ps"</InlineCode>
            " output, which is exactly the leak this tool exists to close."
        </Callout>
    }
}

fn scope() -> impl IntoView {
    view! {
        <P>
            "The security page states the threat model in full. In short, the following are "
            "outside what any local secret manager can offer, and kivro does not pretend "
            "otherwise:"
        </P>

        {definitions(
            &[
                (
                    "A compromised machine",
                    "anything that can run code as you can ask the credential store for the same \
                     values kivro asks for",
                ),
                (
                    "What the child process does",
                    "kivro run hands the secrets to the command you named; it cannot police that \
                     command or its dependencies",
                ),
                (
                    "Per-secret access control",
                    "there is none in 0.1 — anyone you hand a bundle to has every value in it",
                ),
                (
                    "Rotation and audit",
                    "not built yet; the bundle payload is versioned so that it can be, but 0.1 \
                     does neither",
                ),
                (
                    "Guaranteed memory hygiene",
                    "values are zeroized on drop, which narrows the window but is not a boundary \
                     you can rely on",
                ),
            ],
        )}

        <P>
            "Read " <DocLink to=nav::doc_path("security")>"the security model"</DocLink>
            " before you put production credentials anywhere near this — including the parts that "
            "say what is not protected."
        </P>
    }
}

fn next() -> impl IntoView {
    view! {
        <div class="grid gap-4 md:grid-cols-2">
            <Tile
                title="Install"
                eyebrow="5 minutes"
                tagline="Build the CLI, and what each platform needs from you first."
                icon=icons::DOWNLOAD
                bullets=vec![
                    "cargo install, from a path or from git".to_string(),
                    "Linux: a keyring daemon and libdbus-1-dev".to_string(),
                ]
                cta="Read install"
                href=nav::doc_path("install")
            />
            <Tile
                title="Quick start"
                eyebrow="10 minutes"
                tagline="An empty directory to a running command, including the .env migration."
                icon=icons::SPARKLES
                bullets=vec![
                    "init, declare, set, run".to_string(),
                    "import an existing .env, then delete it".to_string(),
                ]
                cta="Start here"
                href=nav::doc_path("quickstart")
            />
            <Tile
                title="CLI reference"
                eyebrow="reference"
                tagline="Every command, flag and exit code, plus the environment variables that change behaviour."
                icon=icons::TERMINAL
                bullets=vec![
                    "13 commands".to_string(),
                    "documented exit codes for scripting".to_string(),
                ]
                cta="Open reference"
                href=nav::doc_path("cli")
            />
            <Tile
                title="Security model"
                eyebrow="read before production"
                tagline="What is protected, what is not, and where the honest limits are."
                icon=icons::SHIELD_ALERT
                bullets=vec![
                    "threat model, in and out of scope".to_string(),
                    "memory handling without the hand-waving".to_string(),
                ]
                cta="Read the model"
                href=nav::doc_path("security")
            />
        </div>
    }
}
