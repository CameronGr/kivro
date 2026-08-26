//! `/docs/architecture` — how the workspace is put together and why.

use crate::content::kit::*;
use crate::nav;
use crate::ui::prelude::*;

pub fn doc() -> Doc {
    Doc::new("architecture", "Architecture", "Reference", "Design & security")
        .tagline(
            "Seven crates, dependencies pointing one way, and a leaf that depends on nothing. \
             This page is the design rationale — the decisions, and the reasons they were made \
             that way rather than the obvious way.",
        )
        .tags(["7 crates", "ports and adapters", "97 tests"])
        .section(
            DocSection::new("overview-graph", "The dependency graph", overview_graph)
                .numbered("1.0")
                .summary("Dependencies point downward only; kivro-core is a leaf."),
        )
        .section(
            DocSection::new("crates", "The crates", crates)
                .numbered("1.1")
                .summary("What each one owns, and what it is allowed to depend on."),
        )
        .section(
            DocSection::new("seven", "Why seven crates and not six", seven)
                .numbered("1.2")
                .summary("The facade cannot live in the core without a cycle."),
        )
        .section(
            DocSection::new("storage-model", "Storage model", storage_model)
                .numbered("2.0")
                .summary("From a scope to a credential, and why the mapping is injective."),
        )
        .section(
            DocSection::new("enumeration-design", "Enumeration", enumeration_design)
                .numbered("2.1")
                .summary("A per-scope index credential, treated as a cache and nothing more."),
        )
        .section(
            DocSection::new("resolution-design", "Environment resolution", resolution_design)
                .numbered("2.2")
                .summary("Five sources in a fixed order, and why the manifest wins."),
        )
        .section(
            DocSection::new("error-design", "Error handling", error_design)
                .numbered("2.3")
                .summary("One enum, two rules, and errors that name their own fix."),
        )
        .section(
            DocSection::new("testing-strategy", "Testing strategy", testing_strategy)
                .numbered("3.0")
                .summary("97 tests, none of which need a real credential store."),
        )
        .section(
            DocSection::new("extension", "Extension points", extension)
                .numbered("3.1")
                .summary("Where a new backend plugs in, and what is deliberately not built."),
        )
}

fn overview_graph() -> impl IntoView {
    view! {
        <CodeBlock
            language="text"
            title="crate graph"
            code=r#"
                              ┌─────────────────┐
                              │   kivro-cli     │  argument parsing, prompts,
                              │   (binary)      │  formatting, exit codes
                              └────────┬────────┘
                                       │
                              ┌────────▼────────┐
                              │     kivro       │  Project · Environment · SecretSet
                              │  (facade lib)   │  env resolution · .env · running
                              └────┬───┬───┬────┘
                       ┌───────────┘   │   └───────────┐
             ┌─────────▼──────┐ ┌──────▼──────┐ ┌──────▼──────────┐
             │ kivro-manifest │ │ kivro-      │ │ kivro-sync      │
             │  .kivro.toml   │ │ keyring     │ │  SyncSource     │
             └─────────┬──────┘ └──────┬──────┘ └──────┬──────────┘
                       │               │               │
                       │               │        ┌──────▼──────────┐
                       │               │        │  kivro-crypto   │  age bundles
                       │               │        └──────┬──────────┘
                       └───────────────┴───────────────┘
                                       │
                              ┌────────▼────────┐
                              │   kivro-core    │  SecretString · names ·
                              │                 │  SecretStore port · Error
                              └─────────────────┘   depends on nothing
            "#
        />

        <P>
            "Dependencies point downward only. " <InlineCode>"kivro-core"</InlineCode>
            " is a leaf: it defines the domain types and the "
            <InlineCode>"SecretStore"</InlineCode> " " <em>"port"</em>
            ", and the adapters implement it. That inversion is what lets tests run against "
            <InlineCode>"MemoryStore"</InlineCode>
            " while production runs against the Windows Credential Manager, with no code in "
            "between caring which."
        </P>
    }
}

fn crates() -> impl IntoView {
    view! {
        {spec_table(
            &["Crate", "Responsibility", "Notable dependencies"],
            &[
                &["kivro-core", "SecretString, validated names, the SecretStore trait, MemoryStore, Error", "zeroize, thiserror"],
                &["kivro-manifest", ".kivro.toml parsing, discovery, environment resolution", "toml"],
                &["kivro-keyring", "OS credential store adapter, insecure file store for tests", "keyring"],
                &["kivro-crypto", "The encrypted bundle format", "age"],
                &["kivro-sync", "SyncSource trait, bundle-directory source", "kivro-crypto"],
                &["kivro", "Library facade: Project, Environment, SecretSet, run", "all of the above"],
                &["kivro-cli", "The kivro binary", "clap, rpassword, ctrlc"],
            ],
        )}

        <Callout tone=Tone::Info title="the keyring adapter is feature-gated">
            <InlineCode>"kivro-keyring"</InlineCode> " has an " <InlineCode>"os-keyring"</InlineCode>
            " feature, on by default, that pulls in the platform backends. Building without it "
            "leaves the memory and file stores — useful for a build that must not link the "
            "platform libraries, and the reason an unavailable backend is a typed error rather "
            "than a panic."
        </Callout>
    }
}

fn seven() -> impl IntoView {
    view! {
        <P>
            "The original sketch put the facade in " <InlineCode>"kivro-core"</InlineCode>
            ". That cannot work: the facade needs the manifest " <em>"and"</em>
            " the keyring, and both of those need the core types, so the dependency would be "
            "circular."
        </P>
        <P>
            "Splitting the facade into its own " <InlineCode>"kivro"</InlineCode>
            " crate keeps " <InlineCode>"kivro-core"</InlineCode>
            " a leaf and gives library consumers a single crate to depend on. The extra crate is "
            "the price of the acyclic graph, and it is a cheap one."
        </P>
    }
}

fn storage_model() -> impl IntoView {
    view! {
        <CodeBlock
            language="text"
            code=r#"
                <app namespace>            kivro-secrets      (configurable)
                  └── <project>              infinity-launcher
                        └── <environment>      dev
                              ├── DATABASE_URL
                              ├── AUTH0_CLIENT_ID
                              └── AUTH0_CLIENT_SECRET
            "#
        />

        <P>"Rendered into the OS credential store as one credential per secret:"</P>

        <CodeBlock
            language="text"
            code=r#"
                service = "kivro-secrets:infinity-launcher:dev"
                user    = "DATABASE_URL"
            "#
        />

        <P>
            "Project, environment and secret names are validated at construction and cannot "
            "contain " <InlineCode>":"</InlineCode>
            ", so the rendering is injective — two different scopes can never produce the same "
            "service string. " <InlineCode>"DATABASE_URL"</InlineCode>
            " in two projects, or in two environments of one project, are unrelated entries."
        </P>

        <Callout tone=Tone::Accent title="validation is what makes this safe">
            "The injectivity argument rests entirely on the name grammars. That is why validation "
            "lives in the leaf crate, happens at construction, and is not something a caller can "
            "skip."
        </Callout>
    }
}

fn enumeration_design() -> impl IntoView {
    view! {
        <P>
            "No credential store offers a portable " <em>"list everything under this service"</em>
            " call. Enumeration is served by a per-scope index credential stored under the "
            "reserved user name " <InlineCode>"__index"</InlineCode>
            ", holding a JSON array of names only. Secret names are uppercase by validation, so "
            <InlineCode>"__index"</InlineCode> " cannot collide with one."
        </P>

        <Callout tone=Tone::Info title="the index is a cache, not the source of truth">
            <InlineCode>"get"</InlineCode> " never consults it, and " <InlineCode>"status"</InlineCode>
            " probes manifest-declared names directly, so a lost or stale index degrades "
            "enumeration and nothing else."
        </Callout>

        <P>
            "This is a deliberate trade. The alternative — treating the index as authoritative — "
            "would make one corrupt credential able to hide a secret that is present, which turns "
            "a cosmetic failure into a correctness failure."
        </P>
    }
}

fn resolution_design() -> impl IntoView {
    view! {
        <List ordered=true>
            <ListItem><InlineCode>"--env"</InlineCode> " on the command line"</ListItem>
            <ListItem>"the " <InlineCode>"KIVRO_ENV"</InlineCode> " environment variable"</ListItem>
            <ListItem><InlineCode>"[environment] default"</InlineCode> " in the manifest"</ListItem>
            <ListItem>
                <InlineCode>"[defaults] environment"</InlineCode> " in the global configuration"
            </ListItem>
            <ListItem>"otherwise an error"</ListItem>
        </List>

        <P>
            "The manifest outranks the global configuration deliberately: a per-machine preference "
            "must never silently change which environment a shared project runs against. Under "
            <InlineCode>"strict"</InlineCode>
            " (the default) an environment the manifest does not declare is rejected, so "
            <InlineCode>"--env prod"</InlineCode>
            " for a project that spells it " <InlineCode>"production"</InlineCode>
            " fails loudly instead of resolving to an empty secret set."
        </P>
    }
}

fn error_design() -> impl IntoView {
    view! {
        <P>
            "One " <InlineCode>"Error"</InlineCode> " enum in " <InlineCode>"kivro-core"</InlineCode>
            ", with " <InlineCode>"thiserror"</InlineCode> ". Two rules:"
        </P>

        {definitions(
            &[
                (
                    "No variant carries a secret value",
                    "keyring errors are mapped variant by variant rather than through to_string(), \
                     because keyring::Error::BadEncoding carries the raw stored bytes and \
                     Ambiguous carries whole credentials",
                ),
                (
                    "Errors carry their fix",
                    "hint() returns the command to run, kind() returns a stable string for --json, \
                     and the CLI maps variants to documented exit codes in one place",
                ),
            ],
        )}

        <CodeBlock
            language="text"
            code=r#"
                error: 2 required secret(s) missing for infinity-launcher/dev: AUTH0_CLIENT_SECRET, S3_ACCESS_KEY

                run:
                    kivro set AUTH0_CLIENT_SECRET
                    kivro set S3_ACCESS_KEY
            "#
        />

        <P>
            "The mapping from variant to exit code lives in exactly one function in "
            <InlineCode>"main.rs"</InlineCode>
            ", which is what keeps the documented codes and the implementation from drifting apart."
        </P>
    }
}

fn testing_strategy() -> impl IntoView {
    view! {
        <div class="grid gap-4 sm:grid-cols-3">
            <Stat label="Tests" value="97" detail="none need a real keyring" icon=icons::FLASK_CONICAL />
            <Stat
                label="Store doubles"
                value="2"
                detail="MemoryStore and the file store"
                icon=icons::WRENCH
                tone=Tone::Info
            />
            <Stat
                label="Leak tests"
                value="5"
                detail="assert on what is absent"
                icon=icons::SHIELD_ALERT
                tone=Tone::Warning
            />
        </div>

        {definitions(
            &[
                (
                    "Unit tests",
                    "beside the code: name validation and namespace injectivity, manifest parsing \
                     and layering, .env parsing and rendering, bundle sealing, opening, tampering \
                     and version rejection, sync planning, config loading",
                ),
                (
                    "Integration tests",
                    "crates/kivro-cli/tests/cli.rs drives the real binary through assert_cmd, \
                     covering every command, exit codes, JSON output, and the full share → accept \
                     and import → export round trips",
                ),
                (
                    "Store doubles",
                    "MemoryStore for library tests, and the file store (KIVRO_STORE=file) for \
                     binary tests, which is how the CLI is exercised in containers with no D-Bus \
                     session",
                ),
            ],
        )}

        <P>
            "Several tests exist specifically to catch leaks rather than logic errors: "
            <InlineCode>"Debug"</InlineCode>
            " output redaction, JSON output containing no values, "
            <InlineCode>"list"</InlineCode>
            " printing no values, and bundle text not containing plaintext."
        </P>
    }
}

fn extension() -> impl IntoView {
    view! {
        <SideBySide
            left=|| {
                view! {
                    <Card title="A new sync backend" eyebrow="one trait" icon=icons::PLUS>
                        "Implement " <InlineCode>"SyncSource"</InlineCode> " and add one arm to "
                        <InlineCode>"kivro_sync::from_config"</InlineCode>
                        ". Nothing above that layer changes, and the bundle format is unaffected — "
                        "the format is the compatibility surface, the transport is not."
                    </Card>
                }
            }
            right=|| {
                view! {
                    <Card
                        title="A new credential backend"
                        eyebrow="one trait"
                        icon=icons::PLUS
                        tone=Tone::Info
                    >
                        "Implement " <InlineCode>"SecretStore"</InlineCode> " and add one arm to "
                        <InlineCode>"kivro_keyring::open"</InlineCode>
                        ". The rest of the workspace addresses stores through the port only."
                    </Card>
                }
            }
        />

        <H3>"Deliberately not built yet"</H3>
        <P>"Designed for, but absent in 0.1:"</P>
        <List>
            <ListItem>"Team sync over HTTP or object storage."</ListItem>
            <ListItem>
                "Secret rotation and versioning — the bundle payload is already versioned."
            </ListItem>
            <ListItem>"Audit logging."</ListItem>
            <ListItem>"Shell and IDE integration."</ListItem>
            <ListItem>"Secret references (one value pointing at another)."</ListItem>
        </List>

        <P>
            "The current status and what a 0.2 would contain are on the "
            <DocLink to=nav::doc_path("roadmap")>"status page"</DocLink> "."
        </P>
    }
}
