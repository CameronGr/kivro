//! `/docs/roadmap` — where 0.1 stands, and what it promises about 0.2.

use crate::content::kit::*;
use crate::nav;
use crate::ui::prelude::*;

pub fn doc() -> Doc {
    Doc::new("roadmap", "Status and roadmap", "Reference", "Design & security")
        .tagline(
            "Version 0.1. Everything documented here works today; everything not documented here \
             does not exist yet. The manifest and bundle formats are versioned and will be read \
             by future releases.",
        )
        .tags(["0.1.0", "format = 1", "pre-1.0"])
        .section(
            DocSection::new("shipped", "What 0.1 ships", shipped)
                .numbered("1.0")
                .summary("The changelog, in slightly more words."),
        )
        .section(
            DocSection::new("promises", "Compatibility promises", promises)
                .numbered("1.1")
                .summary("What a future release will still be able to read."),
        )
        .section(
            DocSection::new("not-yet", "Designed for, not built", not_yet)
                .numbered("2.0")
                .summary("Five things the architecture leaves room for."),
        )
        .section(
            DocSection::new("stability", "Stability of the surfaces", stability)
                .numbered("2.1")
                .summary("Which interfaces are safe to script against before 1.0."),
        )
        .section(
            DocSection::new("contributing", "Repository", contributing)
                .numbered("3.0")
                .summary("Where the code, the specs and the examples live."),
        )
}

fn shipped() -> impl IntoView {
    view! {
        <div class="grid gap-4 sm:grid-cols-3">
            <Stat label="Version" value="0.1.0" detail="first release" icon=icons::SPARKLES />
            <Stat label="Crates" value="7" detail="one to depend on" icon=icons::HASH tone=Tone::Info />
            <Stat
                label="Commands"
                value="13"
                detail="all with --json"
                icon=icons::TERMINAL
                tone=Tone::Accent
            />
        </div>

        <List>
            <ListItem>
                "OS credential store backends: Windows Credential Manager, macOS Keychain, Linux "
                "Secret Service, behind a " <InlineCode>"SecretStore"</InlineCode> " port."
            </ListItem>
            <ListItem>
                <InlineCode>".kivro.toml"</InlineCode>
                " manifest with versioned, forward-compatible parsing, and discovery by walking up "
                "from the working directory."
            </ListItem>
            <ListItem>
                "A deterministic " <InlineCode>"app:project:environment:NAME"</InlineCode>
                " storage namespace."
            </ListItem>
            <ListItem>
                "The CLI: " <InlineCode>"init"</InlineCode> ", " <InlineCode>"set"</InlineCode>
                ", " <InlineCode>"get"</InlineCode> ", " <InlineCode>"list"</InlineCode> ", "
                <InlineCode>"remove"</InlineCode> ", " <InlineCode>"status"</InlineCode> ", "
                <InlineCode>"doctor"</InlineCode> ", " <InlineCode>"run"</InlineCode> ", "
                <InlineCode>"import"</InlineCode> ", " <InlineCode>"export"</InlineCode> ", "
                <InlineCode>"sync"</InlineCode> ", " <InlineCode>"share"</InlineCode> ", "
                <InlineCode>"accept"</InlineCode> "."
            </ListItem>
            <ListItem>
                "age-based encrypted bundles for developer-to-developer sharing, by passphrase or "
                "X25519 recipients."
            </ListItem>
            <ListItem>
                "A " <InlineCode>"SyncSource"</InlineCode> " abstraction with a file-based bundle "
                "source."
            </ListItem>
            <ListItem>
                "The library API the CLI is built on, and 97 tests, none of which require a real "
                "credential store."
            </ListItem>
        </List>
    }
}

fn promises() -> impl IntoView {
    view! {
        {definitions(
            &[
                (
                    "format = 1 manifests will keep parsing",
                    "a future release reads what 0.1 wrote, without migration",
                ),
                (
                    "format = 1 bundles will keep opening",
                    "the envelope and payload versions are separate, and both are checked on read",
                ),
                (
                    "Variable names stay uppercase-only",
                    "the case rule that separates variables from settings inside \
                     [environments.<name>] depends on it",
                ),
                (
                    "The namespace derivation will not change without a format bump",
                    "changing it would orphan every stored secret, which is not something a patch \
                     release may do",
                ),
            ],
        )}

        <Callout tone=Tone::Info title="how a breaking change would arrive">
            "Three mechanisms, in increasing severity: unknown keys are ignored and reported by "
            <InlineCode>"doctor"</InlineCode> "; " <InlineCode>"min_cli_version"</InlineCode>
            " stops an old CLI that would misunderstand a newer file; and "
            <InlineCode>"format"</InlineCode>
            " refuses outright. All three are described on the "
            <DocLink to=nav::doc_path("manifest")>"manifest page"</DocLink> "."
        </Callout>
    }
}

fn not_yet() -> impl IntoView {
    view! {
        <P>
            "The architecture leaves room for each of these, and none of them ships in 0.1. They "
            "are listed so that nobody plans around a feature that does not exist:"
        </P>

        <div class="grid gap-4 md:grid-cols-2">
            <Card title="Team sync over the network" eyebrow="SyncSource" icon=icons::PLANE>
                "HTTP, object storage, or git-backed. One trait implementation and one arm in "
                <InlineCode>"from_config"</InlineCode>
                "; the bundle format is unaffected because the format is the compatibility "
                "surface, not the transport."
            </Card>
            <Card title="Rotation and versioning" eyebrow="payload is versioned" icon=icons::LOADER tone=Tone::Info>
                "The bundle payload already carries a format version, which is what a rotation "
                "history would need. Nothing consumes it that way yet."
            </Card>
            <Card title="Audit logging" eyebrow="not started" icon=icons::FILE_TEXT tone=Tone::Neutral>
                "Who read what, when. Meaningful only alongside some notion of team identity, "
                "which 0.1 also does not have."
            </Card>
            <Card title="Per-secret access control" eyebrow="not started" icon=icons::USER tone=Tone::Neutral>
                "There is none today: anyone you hand a bundle to has every value in it. This is "
                "stated in the " <DocLink to=nav::doc_path("security")>"threat model"</DocLink>
                " rather than left to be discovered."
            </Card>
            <Card title="Shell and IDE integration" eyebrow="not started" icon=icons::TERMINAL tone=Tone::Neutral>
                "Completions, a direnv-style hook, editor run configurations. All of it sits above "
                "the library API, so none of it needs format changes."
            </Card>
            <Card title="Secret references" eyebrow="not started" icon=icons::LINK_2 tone=Tone::Neutral>
                "One declaration pointing at another value, so a connection string can be composed "
                "rather than duplicated."
            </Card>
        </div>
    }
}

fn stability() -> impl IntoView {
    view! {
        {prose_table(
            &["Surface", "Stability in 0.1", "Notes"],
            &[
                &[
                    "Exit codes",
                    "Stable — script against them",
                    "Documented, tested, and mapped in one place in the CLI",
                ],
                &[
                    "Error kind strings",
                    "Stable — script against them",
                    "Exist specifically so --json consumers do not parse messages",
                ],
                &[
                    "--json shapes",
                    "Additive changes expected",
                    "Fields may be added; existing fields will not change meaning",
                ],
                &[
                    "Manifest format 1",
                    "Stable",
                    "Future releases read it; unknown keys are ignored and reported",
                ],
                &[
                    "Bundle format 1",
                    "Stable",
                    "Envelope and payload versioned separately",
                ],
                &[
                    "Library API",
                    "Pre-1.0, may change",
                    "The crate is not published to crates.io yet; pin a git revision",
                ],
                &[
                    "Human-readable output",
                    "Not an interface",
                    "Use --json for anything a machine reads",
                ],
            ],
        )}
    }
}

fn contributing() -> impl IntoView {
    view! {
        <P>
            "The repository holds the crates, the specifications this site is written from, and "
            "runnable examples for Rust, Node, Make and GitHub Actions."
        </P>

        {spec_table(
            &["Path", "Contents"],
            &[
                &["crates/", "The seven crates, plus the CHANGELOG"],
                &["docs/", "The source specifications: architecture, manifest, bundle, security, CLI, integration"],
                &["examples/", "rust-app, node-app, a Makefile, a CI workflow, and an annotated manifest"],
                &["site/", "This documentation site — Leptos, compiled to wasm, no server runtime"],
            ],
        )}

        <ButtonGroup>
            <Button
                variant=Variant::Soft
                href=nav::REPO
                target="_blank"
                icon=icons::GITHUB
                trailing_icon=icons::ARROW_UP_RIGHT
            >
                "Repository"
            </Button>
            <Button
                variant=Variant::Glass
                href=nav::repo_file("crates/CHANGELOG.md")
                target="_blank"
                icon=icons::FILE_TEXT
            >
                "Changelog"
            </Button>
            <Button
                variant=Variant::Glass
                href=nav::repo_file("examples")
                target="_blank"
                icon=icons::FLASK_CONICAL
            >
                "Examples"
            </Button>
        </ButtonGroup>
    }
}
