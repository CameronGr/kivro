//! `/docs/library` — the Rust API the CLI is built on.

use crate::content::kit::*;
use crate::nav;
use crate::ui::prelude::*;

pub fn doc() -> Doc {
    Doc::new("library", "Library API", "Reference", "Reference")
        .tagline(
            "The `kivro` crate is the API the CLI is built on, and the one other tools should \
             use. Nothing CLI-specific — argument parsing, prompting, terminal formatting, exit \
             codes — appears in it.",
        )
        .tags(["kivro", "no unsafe", "SecretString"])
        .section(
            DocSection::new("install-lib", "Adding the dependency", install_lib)
                .numbered("1.0")
                .summary("One crate to depend on; the rest are re-exported through it."),
        )
        .section(
            DocSection::new("project-type", "Project", project_type)
                .numbered("1.1")
                .summary("Discovery, construction from parts, and environment selection."),
        )
        .section(
            DocSection::new("environment-type", "Environment", environment_type)
                .numbered("1.2")
                .summary("One environment of one project: the read/write surface."),
        )
        .section(
            DocSection::new("secretset", "SecretSet", secretset)
                .numbered("1.3")
                .summary("Loaded values, and the one place they become plain strings."),
        )
        .section(
            DocSection::new("status-types", "Status types", status_types)
                .numbered("1.4")
                .summary("EnvironmentStatus and SecretStatus, for readiness checks."),
        )
        .section(
            DocSection::new("secretstring", "SecretString", secretstring)
                .numbered("1.5")
                .summary("What it does not implement is the point."),
        )
        .section(
            DocSection::new("run-module", "kivro::run", run_module)
                .numbered("2.0")
                .summary("Spawning a child with the secrets injected, exit codes included."),
        )
        .section(
            DocSection::new("config-module", "kivro::config", config_module)
                .numbered("2.1")
                .summary("Loading and saving the per-machine configuration."),
        )
        .section(
            DocSection::new("envfile-module", "kivro::envfile", envfile_module)
                .numbered("2.2")
                .summary("Parsing and rendering .env text, plus the heuristic doctor uses."),
        )
        .section(
            DocSection::new("errors", "Errors", errors)
                .numbered("2.3")
                .summary("One enum, a stable kind string, and a hint that names the fix."),
        )
        .section(
            DocSection::new("testing", "Testing against a fake store", testing)
                .numbered("3.0")
                .summary("MemoryStore, and why the port is a trait in the first place."),
        )
}

fn install_lib() -> impl IntoView {
    view! {
        <CodeBlock
            language="toml"
            title="Cargo.toml"
            code=r#"
                [dependencies]
                kivro = { git = "https://github.com/CameronGr/kivro" }
            "#
        />

        <P>"The facade re-exports the crates you would otherwise have to add by hand:"</P>

        {spec_table(
            &["Re-export", "Is"],
            &[
                &["kivro::core", "kivro_core — SecretString, validated names, the SecretStore port"],
                &["kivro::manifest", "kivro_manifest — Manifest, VariableSpec, ResolvedEnvironment"],
                &["kivro::crypto", "kivro_crypto — Bundle, seal, open, peek"],
                &["kivro::Secret", "an alias for SecretString"],
                &["kivro::SecretsError", "an alias for the crate-wide Error"],
            ],
        )}

        <Callout tone=Tone::Info title="why a separate facade crate">
            "The facade needs the manifest " <em>"and"</em>
            " the keyring, and both of those need the core types. Putting the facade in "
            <InlineCode>"kivro-core"</InlineCode>
            " would make the dependency circular, so " <InlineCode>"kivro-core"</InlineCode>
            " stays a leaf and " <InlineCode>"kivro"</InlineCode>
            " is the single crate consumers depend on."
        </Callout>
    }
}

fn project_type() -> impl IntoView {
    view! {
        <CodeBlock
            language="rust"
            code=r#"
                use kivro::Project;

                fn main() -> Result<(), Box<dyn std::error::Error>> {
                    let project = Project::discover()?;            // walks up for .kivro.toml
                    let env = project.resolve_environment(None)?;  // --env / KIVRO_ENV / default
                    let secrets = env.load()?;                     // errors if a required one is missing

                    let database_url = secrets.get("DATABASE_URL")?;
                    connect(database_url.expose_secret())?;
                    Ok(())
                }
            "#
        />

        {spec_table(
            &["Method", "Signature", "Notes"],
            &[
                &["Project::discover", "() -> Result<Project>", "Walks up from the current directory."],
                &["Project::discover_from", "(impl AsRef<Path>) -> Result<Project>", "Walks up from a path you choose."],
                &["Project::new", "(Manifest, Box<dyn SecretStore>, Config) -> Project", "Assemble from parts. For tests and embedders."],
                &["project.manifest", "() -> &Manifest", "The parsed manifest."],
                &["project.name", "() -> &ProjectName", "Project identity."],
                &["project.store", "() -> &dyn SecretStore", "The backing store."],
                &["project.config", "() -> &Config", "The loaded global configuration."],
                &["project.environment", "(&str) -> Result<Environment>", "Select by name. Validates and resolves against the manifest."],
                &["project.resolve_environment", "(Option<&str>) -> Result<Environment>", "Follows the documented precedence."],
            ],
        )}

        <P>
            <InlineCode>"discover"</InlineCode>
            " also loads the global configuration and opens the store selected by "
            <InlineCode>"KIVRO_STORE"</InlineCode>
            ", so an unreachable keyring surfaces here rather than at first use."
        </P>
    }
}

fn environment_type() -> impl IntoView {
    view! {
        {spec_table(
            &["Method", "Signature", "Notes"],
            &[
                &["name", "() -> &EnvironmentName", "Resolved environment name."],
                &["project_name", "() -> &ProjectName", "Owning project."],
                &["scope", "() -> Scope", "The storage scope this environment addresses."],
                &["declarations", "() -> &BTreeMap<SecretName, VariableSpec>", "Effective declarations after layering."],
                &["set", "(&SecretName, &SecretString) -> Result<()>", "Store a value, overwriting."],
                &["get", "(&SecretName) -> Result<Option<SecretString>>", "Fetch one value if present."],
                &["remove", "(&SecretName) -> Result<bool>", "Delete one value; returns whether it existed."],
                &["stored_names", "() -> Result<Vec<SecretName>>", "Names present in the store, declared or not."],
                &["status", "() -> Result<EnvironmentStatus>", "Per-variable presence, for status/list/doctor."],
                &["load", "() -> Result<SecretSet>", "Every declared secret; errors if a required one is missing."],
                &["load_available", "() -> Result<SecretSet>", "Whatever is present, without enforcing required."],
                &["load_all_stored", "() -> Result<SecretSet>", "Every stored secret in the scope, declared or not."],
            ],
        )}

        <Callout tone=Tone::Accent title="Debug renders identity only">
            <InlineCode>"Environment"</InlineCode>
            " holds a store handle, never values, and its "
            <InlineCode>"Debug"</InlineCode>
            " prints the project and environment names and nothing else. The same discipline "
            "applies to every type in the workspace that can reach a value."
        </Callout>

        <P>
            <InlineCode>"stored_names"</InlineCode>
            " merges the store's enumeration index with a direct probe of every declared name, so "
            "a stale index degrades listing but never correctness."
        </P>
    }
}

fn secretset() -> impl IntoView {
    view! {
        {spec_table(
            &["Method", "Signature", "Notes"],
            &[
                &["get", "(&str) -> Result<&SecretString>", "Errors with MissingSecret if absent."],
                &["find", "(&str) -> Option<&SecretString>", "The non-erroring form."],
                &["environment", "() -> Vec<(String, String)>", "Ready for std::process::Command::envs."],
                &["values", "() -> &BTreeMap<SecretName, SecretString>", "Borrow the map."],
                &["into_values", "(self) -> BTreeMap<SecretName, SecretString>", "Consume the set."],
                &["names", "() -> Vec<SecretName>", "Names held."],
                &["len / is_empty", "() -> usize / bool", "Count of values."],
                &["from_values", "(ProjectName, EnvironmentName, BTreeMap<..>) -> SecretSet", "Build a set directly, e.g. from a decrypted bundle."],
            ],
        )}

        <CodeBlock
            language="rust"
            code=r#"
                let secrets = Project::discover()?.environment("dev")?.load()?;

                std::process::Command::new("cargo")
                    .args(["run"])
                    .envs(secrets.environment())
                    .spawn()?;
            "#
        />

        <Callout tone=Tone::Warning title="environment() is the exit from the type system">
            "It is the one method that turns secrets back into plain "
            <InlineCode>"String"</InlineCode> "s, because "
            <InlineCode>"Command::envs"</InlineCode> " takes "
            <InlineCode>"AsRef<OsStr>"</InlineCode>
            ". Call it as late as possible and hand the result straight to the process API — do "
            "not keep it around."
        </Callout>
    }
}

fn status_types() -> impl IntoView {
    view! {
        <CodeBlock
            language="rust"
            title="a readiness check in build.rs or a task runner"
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

        {spec_table(
            &["Item", "Type", "Meaning"],
            &[
                &["EnvironmentStatus::project", "ProjectName", "Project the report is for."],
                &["EnvironmentStatus::environment", "EnvironmentName", "Environment the report is for."],
                &["EnvironmentStatus::entries", "Vec<SecretStatus>", "One entry per declared or stored secret, sorted by name."],
                &["missing_required", "() -> Vec<&SecretStatus>", "Required entries with no stored value."],
                &["is_satisfied", "() -> bool", "Whether every required secret is present."],
                &["undeclared", "() -> Vec<&SecretStatus>", "Stored values with no matching declaration."],
                &["SecretStatus", "{ name, required, present, declared, deprecated, description }", "Presence of one secret. Carries no value."],
            ],
        )}
    }
}

fn secretstring() -> impl IntoView {
    view! {
        <P>
            "Secret values are never a plain " <InlineCode>"String"</InlineCode>
            " anywhere in the workspace. "
            <InlineCode>"SecretString"</InlineCode>
            " wraps a " <InlineCode>"zeroize::Zeroizing<String>"</InlineCode>
            ", so the heap buffer is overwritten on drop — and, more importantly, it refuses to "
            "implement the traits that leak."
        </P>

        {prose_table(
            &["Trait", "Implemented", "Why"],
            &[
                &["Display", "No — deliberately", "So a value cannot be interpolated into a log line by accident."],
                &["Debug", "Yes, redacted", "Prints SecretString(<redacted>), including inside containing structs."],
                &["Serialize", "No — deliberately", "So it cannot be written into JSON output."],
                &["Deserialize", "Yes", "Moving data into a more protected representation is the safe direction."],
                &["Clone, PartialEq, Eq", "Yes", "Ordinary value semantics."],
            ],
        )}

        <CodeBlock
            language="rust"
            code=r#"
                let secret = SecretString::new("hunter2");

                secret.expose_secret();   // &str — deliberately verbose, so every call site greps
                secret.len();             // 7
                secret.describe();        // "<7 bytes, redacted>"

                println!("{secret:?}");   // SecretString(<redacted>)
                // println!("{secret}");  // does not compile
            "#
        />

        <Callout tone=Tone::Warning title="zeroization is defence in depth, not a boundary">
            "String growth reallocates and leaves old bytes behind; moves and clones copy bytes "
            "nothing tracks; the OS can page memory to swap or a core dump. Read the "
            <DocLink to=nav::doc_path("security")>"memory handling section"</DocLink>
            " before treating this as a guarantee."
        </Callout>
    }
}

fn run_module() -> impl IntoView {
    view! {
        <CodeBlock
            language="rust"
            code=r#"
                use kivro::run::{run, command, RunOptions};

                let secrets = Project::discover()?.environment("dev")?.load()?;

                // Signal handling and exit-code propagation already handled:
                let code = run("cargo", &["run".into()], &secrets, &RunOptions::default())?;
                std::process::exit(code);

                // Or build the Command yourself and take it from there:
                let mut cmd = command("cargo", &["run".into()], &secrets, &RunOptions::default());
            "#
        />

        {spec_table(
            &["Item", "Meaning"],
            &[
                &["RunOptions::inherit_environment", "Pass the parent environment through. Default true."],
                &["RunOptions::extra", "Extra NON-secret variables to set on the child."],
                &["command(..)", "Builds a std::process::Command with the environment assembled."],
                &["run(..)", "Spawns, waits, and returns the child's exit code (128 + signal on Unix)."],
            ],
        )}

        <P>
            "With " <InlineCode>"inherit_environment: false"</InlineCode>
            " the child starts from a cleared environment plus a minimal passthrough: "
            <InlineCode>"PATH"</InlineCode> ", " <InlineCode>"HOME"</InlineCode> ", "
            <InlineCode>"USER"</InlineCode> ", " <InlineCode>"SHELL"</InlineCode> ", "
            <InlineCode>"TMPDIR"</InlineCode> ", " <InlineCode>"TEMP"</InlineCode> ", "
            <InlineCode>"TMP"</InlineCode> ", " <InlineCode>"LANG"</InlineCode> ", "
            <InlineCode>"LC_ALL"</InlineCode> ", " <InlineCode>"TERM"</InlineCode>
            ", and the Windows equivalents."
        </P>
    }
}

fn config_module() -> impl IntoView {
    view! {
        <CodeBlock
            language="rust"
            code=r#"
                use kivro::config::Config;

                let config = Config::load()?;              // missing file -> defaults
                let path   = Config::path()?;              // platform-appropriate location
                let dir    = Config::directory()?;         // honours KIVRO_CONFIG_DIR

                let mut config = Config::default();
                config.defaults.environment = Some("dev".into());
                config.save()?;                            // creates the directory if needed
            "#
        />

        {spec_table(
            &["Field", "Type", "Default"],
            &[
                &["defaults.environment", "Option<String>", "None"],
                &["ui.color", "bool", "true"],
                &["storage.namespace", "String", "\"kivro-secrets\""],
            ],
        )}

        <P>
            "A missing configuration file yields defaults; an invalid one is an error carrying its "
            "path and the parser's message, with " <InlineCode>"kind() == \"config_invalid\""</InlineCode>
            "."
        </P>
    }
}

fn envfile_module() -> impl IntoView {
    view! {
        {spec_table(
            &["Item", "Signature", "Notes"],
            &[
                &["parse", "(&Path, &str) -> Result<EnvFile>", "Comments, blank lines, export prefix, quoted values."],
                &["render", "(&BTreeMap<SecretName, SecretString>) -> String", "The inverse. Used by kivro export."],
                &["looks_like_secrets", "(&str) -> bool", "The heuristic doctor uses to escalate a stray .env to a failure."],
                &["EnvFile::entries", "BTreeMap<SecretName, SecretString>", "Entries whose keys are valid secret names."],
                &["EnvFile::skipped", "Vec<(String, usize)>", "Keys that were not valid names, with line numbers. Reported, never dropped."],
            ],
        )}

        <P>
            "A malformed line is an error carrying the path and line number, not a silently "
            "skipped entry — a " <InlineCode>".env"</InlineCode>
            " you cannot fully parse is one you cannot safely claim to have imported."
        </P>
    }
}

fn errors() -> impl IntoView {
    view! {
        <P>
            "One " <InlineCode>"Error"</InlineCode> " enum lives in "
            <InlineCode>"kivro-core"</InlineCode> ", built with "
            <InlineCode>"thiserror"</InlineCode> ", and it is "
            <InlineCode>"#[non_exhaustive]"</InlineCode> ". Two rules hold across every variant:"
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
                    "hint() returns the command to run; kind() returns a stable string for \
                     machine-readable output",
                ),
            ],
        )}

        <CodeBlock
            language="rust"
            code=r#"
                match project.environment("prod") {
                    Ok(env) => { /* ... */ }
                    Err(e) => {
                        eprintln!("{e}");                    // human-readable
                        eprintln!("{:?}", e.kind());         // "unknown_environment"
                        if let Some(hint) = e.hint() {
                            eprintln!("{hint}");             // "declared environments: dev, production"
                        }
                    }
                }
            "#
        />

        <P>
            "The CLI maps variants to the documented exit codes in one function in "
            <InlineCode>"main.rs"</InlineCode>
            ", which is the only place the mapping exists. Every "
            <InlineCode>"kind"</InlineCode> " value and its fix is listed on the "
            <DocLink to=nav::doc_path("troubleshooting")>"troubleshooting page"</DocLink> "."
        </P>
    }
}

fn testing() -> impl IntoView {
    view! {
        <P>
            <InlineCode>"SecretStore"</InlineCode> " is a trait — a "
            <em>"port"</em>
            " — defined in the leaf crate, and the OS backends are adapters implementing it. That "
            "inversion is what lets tests run against an in-memory store while production runs "
            "against Credential Manager, with no code in between caring which."
        </P>

        <CodeBlock
            language="rust"
            code=r##"
                use kivro::{config::Config, Project};
                use kivro::core::MemoryStore;
                use kivro::manifest::Manifest;

                let manifest = Manifest::parse(".kivro.toml", r#"
                    [project]
                    name = "test-project"
                    [environment]
                    default = "dev"
                    [variables]
                    DATABASE_URL = { required = true }
                "#)?;

                let project = Project::new(manifest, Box::new(MemoryStore::new()), Config::default());
                let env = project.environment("dev")?;

                env.set(&SecretName::new("DATABASE_URL")?, &SecretString::new("postgres://x"))?;
                assert!(env.status()?.is_satisfied());
            "##
        />

        {spec_table(
            &["Trait method", "Signature"],
            &[
                &["backend", "() -> &str"],
                &["check_available", "() -> Result<()>"],
                &["is_secure", "() -> bool"],
                &["get", "(&StoreKey) -> Result<Option<SecretString>>"],
                &["set", "(&StoreKey, &SecretString) -> Result<()>"],
                &["delete", "(&StoreKey) -> Result<bool>"],
                &["list", "(&Scope) -> Result<Vec<SecretName>>"],
            ],
        )}

        <Callout tone=Tone::Info title="the workspace's own test suite">
            "97 tests, none of which require a real credential store. Library tests use "
            <InlineCode>"MemoryStore"</InlineCode> "; the CLI integration tests drive the real "
            "binary with " <InlineCode>"KIVRO_STORE=file"</InlineCode>
            ", which is how it is exercised in containers with no D-Bus session."
        </Callout>
    }
}
