//! `/` — the landing page.

use crate::content;
use crate::content::kit::prose_table;
use crate::nav;
use crate::shell::{go_to, use_go};
use crate::ui::prelude::*;
use crate::ui::style::{GLASS, TRANSITION};

#[component]
pub fn Home() -> impl IntoView {
    view! {
        <div class="mx-auto flex max-w-[1200px] flex-col gap-16 px-4 pb-8 pt-10 md:px-8 md:pt-16">
            <Hero />
            <HowItWorks />
            <Features />
            <Platforms />
            <Comparison />
            <DocsPreview />
        </div>
    }
}

#[component]
fn Hero() -> impl IntoView {
    let go = use_go();
    let start = go_to(go, nav::doc_path("quickstart"));
    let docs = go_to(go, nav::DOCS);

    view! {
        <section class="flex flex-col gap-10 lg:flex-row lg:items-start lg:gap-12">
            <div class="flex min-w-0 flex-1 flex-col gap-6">
                <div class="flex flex-wrap items-center gap-2">
                    <Badge tone=Tone::Accent size=Size::Sm>{format!("v{}", nav::VERSION)}</Badge>
                    <Badge tone=Tone::Neutral size=Size::Sm>"Rust"</Badge>
                    <Badge tone=Tone::Neutral size=Size::Sm>"MIT"</Badge>
                    <Badge tone=Tone::Info size=Size::Sm>"Windows · macOS · Linux"</Badge>
                </div>

                <h1 class="text-4xl font-semibold leading-[1.1] tracking-tight text-white md:text-5xl">
                    "Reimagine how "
                    <span class="text-accent-300">".env"</span>
                    " files work."
                </h1>

                <p class="max-w-2xl text-base leading-8 text-white/68 md:text-lg">
                    "kivro keeps the list of secrets a project needs in a file you commit, and the "
                    "values in your operating system's credential store. When you run something, "
                    "it injects them straight into the process — no intermediate file, no shell "
                    "argument, nothing left on disk."
                </p>

                <div class="flex flex-wrap items-center gap-3">
                    <Button
                        variant=Variant::Solid
                        size=Size::Lg
                        trailing_icon=icons::ARROW_RIGHT
                        on_click=start
                    >
                        "Quick start"
                    </Button>
                    <Button variant=Variant::Glass size=Size::Lg icon=icons::BOOK_OPEN on_click=docs>
                        "Documentation"
                    </Button>
                    <Button
                        variant=Variant::Ghost
                        size=Size::Lg
                        href=nav::REPO
                        target="_blank"
                        icon=icons::GITHUB
                        trailing_icon=icons::ARROW_UP_RIGHT
                    >
                        "GitHub"
                    </Button>
                </div>

                <CommandLine command="cargo install --git https://github.com/CameronGr/kivro kivro-cli" />

                <div class="grid gap-3 sm:grid-cols-3">
                    <Stat
                        label="Commands"
                        value="13"
                        detail="each with --json"
                        icon=icons::TERMINAL
                    />
                    <Stat
                        label="Tests"
                        value="97"
                        detail="none need a real keyring"
                        icon=icons::FLASK_CONICAL
                        tone=Tone::Info
                    />
                    <Stat
                        label="Plaintext files"
                        value="0"
                        detail="unless you ask for one"
                        icon=icons::SHIELD_ALERT
                        tone=Tone::Accent
                    />
                </div>
            </div>

            <div class="w-full lg:w-[520px] lg:shrink-0">
                <CodeBlock
                    language="bash"
                    title="the whole workflow"
                    line_numbers=true
                    highlight_lines=vec![5]
                    code=r#"
                        kivro init                 # writes .kivro.toml
                        $EDITOR .kivro.toml        # declare what you need
                        kivro set DATABASE_URL     # prompted, never echoed
                        kivro status               # what is missing
                        kivro run -- cargo run     # injected, then gone
                    "#
                />
                <div class="mt-4">
                    <Callout tone=Tone::Accent title="what your teammate sees">
                        "They clone the repository, run " <InlineCode>"kivro status"</InlineCode>
                        ", and get exit code 3 with the exact list of what to set — instead of a "
                        "stack trace at the first database call."
                    </Callout>
                </div>
            </div>
        </section>
    }
}

#[component]
fn HowItWorks() -> impl IntoView {
    view! {
        <section class="flex flex-col gap-6">
            <div class="flex flex-col gap-2">
                <Eyebrow>"How it works"</Eyebrow>
                <h2 class="text-2xl font-semibold tracking-tight text-white md:text-3xl">
                    "Two halves that never meet"
                </h2>
                <p class="max-w-3xl text-sm leading-7 text-white/62">
                    "The names live in the repository, because they are a fact about the project. "
                    "The values live in the credential store, because they are credentials. "
                    "Nothing in the design lets one become the other."
                </p>
            </div>

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
                <div class=cn!(GLASS, "rounded-2xl p-6")>
                    <div class="mb-3 flex items-center gap-2">
                        <Icon icon=icons::FILE_TEXT class="h-4 w-4 text-accent-400" />
                        <span class="text-sm font-semibold text-white">
                            ".kivro.toml — you commit this"
                        </span>
                    </div>
                    <CodeBlock
                        language="toml"
                        dense=true
                        code=r#"
                            [project]
                            name = "launcher"

                            [environment]
                            default = "dev"

                            [variables]
                            DATABASE_URL        = { required = true }
                            AUTH0_CLIENT_SECRET = { required = true }
                            SENTRY_DSN          = { required = false }
                        "#
                    />
                </div>
                <div class=cn!(GLASS, "rounded-2xl p-6")>
                    <div class="mb-3 flex items-center gap-2">
                        <Icon icon=icons::SHIELD_ALERT class="h-4 w-4 text-accent-400" />
                        <span class="text-sm font-semibold text-white">
                            "The keyring — nobody commits this"
                        </span>
                    </div>
                    <CodeBlock
                        language="text"
                        dense=true
                        code=r#"
                            service = "kivro-secrets:launcher:dev"
                            user    = "DATABASE_URL"

                            service = "kivro-secrets:launcher:dev"
                            user    = "AUTH0_CLIENT_SECRET"

                            # one credential per secret, per environment,
                            # addressed so two projects can never collide
                        "#
                    />
                </div>
            </div>
        </section>
    }
}

#[component]
fn Features() -> impl IntoView {
    view! {
        <section class="flex flex-col gap-6">
            <div class="flex flex-col gap-2">
                <Eyebrow>"What you get"</Eyebrow>
                <h2 class="text-2xl font-semibold tracking-tight text-white md:text-3xl">
                    "Small tool, specific promises"
                </h2>
            </div>

            <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                <Card
                    title="Nothing in the repository"
                    eyebrow="the point"
                    icon=icons::FILE_TEXT
                >
                    "The manifest holds names, requiredness and descriptions. There is no field it "
                    "could put a value in, so there is nothing to leak when it is committed."
                </Card>
                <Card
                    title="Injected, not written"
                    eyebrow="kivro run"
                    icon=icons::TERMINAL
                    tone=Tone::Info
                >
                    "Values go into the child process environment block and nowhere else. No "
                    "temporary file, no argv, no shell history."
                </Card>
                <Card
                    title="Fails before it starts"
                    eyebrow="exit code 3"
                    icon=icons::CIRCLE_ALERT
                    tone=Tone::Warning
                >
                    "A missing required secret stops the command with a list of names and the "
                    "exact lines that would fix it — not a null connection string at runtime."
                </Card>
                <Card
                    title="Encrypted handover"
                    eyebrow="age"
                    icon=icons::USER
                    tone=Tone::Accent
                >
                    "kivro share writes one age-encrypted file, safe to send over any channel. "
                    "accept verifies it belongs to your project before it stores anything."
                </Card>
                <Card
                    title="Diagnoses itself"
                    eyebrow="kivro doctor"
                    icon=icons::WRENCH
                    tone=Tone::Neutral
                >
                    "Manifest, keyring, environment, required secrets, deprecated values, a stray "
                    ".env, missing .gitignore entries, bundles left in the project root."
                </Card>
                <Card
                    title="A library, not just a binary"
                    eyebrow="kivro crate"
                    icon=icons::HASH
                    tone=Tone::Dev
                >
                    "The CLI is one consumer of the API. Rust programs can load secrets directly, "
                    "with a SecretString that refuses to implement Display or Serialize."
                </Card>
            </div>
        </section>
    }
}

#[component]
fn Platforms() -> impl IntoView {
    view! {
        <section class="flex flex-col gap-6">
            <div class="flex flex-col gap-2">
                <Eyebrow>"Storage backends"</Eyebrow>
                <h2 class="text-2xl font-semibold tracking-tight text-white md:text-3xl">
                    "Whatever your OS already protects"
                </h2>
                <p class="max-w-3xl text-sm leading-7 text-white/62">
                    "One trait, three adapters. An unavailable store is an error — never a silent "
                    "fallback to something weaker."
                </p>
            </div>

            {prose_table(
                &["Platform", "Backend", "Extra setup"],
                &[
                    &["Windows", "Credential Manager", "None"],
                    &["macOS", "Keychain", "None"],
                    &[
                        "Linux",
                        "Secret Service (GNOME Keyring, KWallet, KeePassXC)",
                        "A running keyring daemon; libdbus-1-dev at build time",
                    ],
                    &[
                        "CI / containers",
                        "KIVRO_STORE=file — plaintext, warned about on every command",
                        "Read the security page before using it anywhere else",
                    ],
                ],
            )}
        </section>
    }
}

#[component]
fn Comparison() -> impl IntoView {
    view! {
        <section class="flex flex-col gap-6">
            <div class="flex flex-col gap-2">
                <Eyebrow>"Why bother"</Eyebrow>
                <h2 class="text-2xl font-semibold tracking-tight text-white md:text-3xl">
                    "The .env failure modes, one by one"
                </h2>
            </div>

            {prose_table(
                &["Failure", "With a .env file", "With kivro"],
                &[
                    &[
                        "Committed by accident",
                        "The values are in the history forever",
                        "The committed file holds names only",
                    ],
                    &[
                        "Shared with a colleague",
                        "Plaintext over whatever channel is convenient",
                        "An age-encrypted bundle; the passphrase travels separately",
                    ],
                    &[
                        "Someone is missing a variable",
                        "The app starts, then fails somewhere confusing",
                        "The command refuses to start and names what is missing",
                    ],
                    &[
                        "A new variable is added",
                        "Everyone finds out when their branch breaks",
                        "The manifest change arrives with the code change",
                    ],
                    &[
                        "Left on disk",
                        "Readable by anything that can read your home directory",
                        "Held by the OS credential store",
                    ],
                ],
            )}

            <Callout tone=Tone::Info title="who this is for">
                "Small teams without a platform engineer to run a hosted secret manager. No "
                "server, no account, and no sync service that has to stay up for your build to "
                "work."
            </Callout>
        </section>
    }
}

#[component]
fn DocsPreview() -> impl IntoView {
    let go = use_go();

    view! {
        <section class="flex flex-col gap-6">
            <div class="flex flex-wrap items-end justify-between gap-4">
                <div class="flex flex-col gap-2">
                    <Eyebrow>"Documentation"</Eyebrow>
                    <h2 class="text-2xl font-semibold tracking-tight text-white md:text-3xl">
                        "Thirteen pages, written from the source"
                    </h2>
                </div>
                <Button
                    variant=Variant::Glass
                    trailing_icon=icons::ARROW_RIGHT
                    on_click=go_to(go, nav::DOCS)
                >
                    "Browse all"
                </Button>
            </div>

            <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
                {content::groups()
                    .into_iter()
                    .map(|(label, entries)| {
                        view! {
                            <div class=cn!(GLASS, "flex flex-col gap-3 rounded-2xl p-5")>
                                <Eyebrow>{label}</Eyebrow>
                                <ul class="flex flex-col gap-2">
                                    {entries
                                        .into_iter()
                                        .map(|entry| {
                                            let click = go_to(go, nav::doc_path(entry.slug));
                                            view! {
                                                <li>
                                                    <button
                                                        type="button"
                                                        class=cn!(
                                                            "flex w-full items-start gap-2.5 rounded-xl px-2 py-1.5 text-left",
                                                            TRANSITION,
                                                            "hover:bg-white/[0.04]",
                                                        )
                                                        on:click=move |_| click.run(())
                                                    >
                                                        <Icon
                                                            icon=entry.icon
                                                            class="mt-0.5 h-4 w-4 shrink-0 text-accent-400/70"
                                                        />
                                                        <span class="min-w-0">
                                                            <span class="block text-sm font-medium text-white">
                                                                {entry.title}
                                                            </span>
                                                            <span class="block text-xs leading-5 text-white/42">
                                                                {entry.blurb}
                                                            </span>
                                                        </span>
                                                    </button>
                                                </li>
                                            }
                                        })
                                        .collect_view()}
                                </ul>
                            </div>
                        }
                    })
                    .collect_view()}
            </div>
        </section>
    }
}
