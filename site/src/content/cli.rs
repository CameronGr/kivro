//! `/docs/cli` — the complete command reference.

use crate::content::kit::*;
use crate::nav;
use crate::ui::prelude::*;

pub fn doc() -> Doc {
    Doc::new("cli", "CLI reference", "Reference", "Reference")
        .tagline(
            "Thirteen commands, five global options, nine exit codes. Every command accepts \
             --env, --project, --json, --quiet and --no-color, and no command ever prints a \
             secret value unless you explicitly ask it to.",
        )
        .tags(["13 commands", "documented exit codes", "--json everywhere"])
        .section(
            DocSection::new("global", "Global options", global)
                .numbered("1.0")
                .summary("Accepted by every subcommand, before or after it."),
        )
        .section(
            DocSection::new("exit-codes", "Exit codes", exit_codes)
                .numbered("1.1")
                .summary("Stable, documented, and the intended interface for build tooling."),
        )
        .section(
            DocSection::new("init", "kivro init", init)
                .numbered("2.0")
                .summary("Create the manifest. Never creates a .env, never invents a value."),
        )
        .section(
            DocSection::new("values", "set · get · list · remove", values)
                .numbered("2.1")
                .summary("The four operations on a single stored value."),
        )
        .section(
            DocSection::new("status", "kivro status", status)
                .numbered("2.2")
                .summary("The required-secret report, and the exit code build systems key off."),
        )
        .section(
            DocSection::new("doctor", "kivro doctor", doctor)
                .numbered("2.3")
                .summary("Twelve checks across manifest, store, environment and git hygiene."),
        )
        .section(
            DocSection::new("run", "kivro run", run_cmd)
                .numbered("2.4")
                .summary("Load, spawn, propagate. The command you will type most."),
        )
        .section(
            DocSection::new("envfiles", "import · export", envfiles)
                .numbered("2.5")
                .summary("Getting off .env, and the escape hatch back to one."),
        )
        .section(
            DocSection::new("bundles", "share · accept", bundles)
                .numbered("2.6")
                .summary("Encrypted developer-to-developer transfer."),
        )
        .section(
            DocSection::new("sync", "kivro sync", sync)
                .numbered("2.7")
                .summary("Compare the manifest, the local store, and a configured source."),
        )
        .section(
            DocSection::new("env-vars", "Environment variables", env_vars)
                .numbered("3.0")
                .summary("Six variables that change behaviour without touching a file."),
        )
        .section(
            DocSection::new("config", "Global configuration", config)
                .numbered("3.1")
                .summary("Per-machine preferences. Non-secret settings only."),
        )
        .section(
            DocSection::new("json", "JSON output", json)
                .numbered("3.2")
                .summary("What each command emits under --json, including on failure."),
        )
}

fn global() -> impl IntoView {
    view! {
        {spec_table(
            &["Option", "Meaning"],
            &[
                &["-e, --env <NAME>", "Environment to operate on. Highest precedence of all the environment sources."],
                &["-p, --project <PATH>", "A project directory or a manifest path, instead of discovery from the working directory."],
                &["--json", "Machine-readable output. Never contains a value unless the command explicitly requests one."],
                &["--no-color", "Disable colour. NO_COLOR in the environment does the same."],
                &["-q, --quiet", "Suppress informational output. Warnings and errors still print."],
            ],
        )}

        <P>
            "Discovery walks up from the working directory looking for "
            <InlineCode>".kivro.toml"</InlineCode> ", so any subdirectory of the project works. "
            <InlineCode>"--project"</InlineCode>
            " short-circuits that, and accepts either the directory or the file itself."
        </P>

        <CodeBlock
            language="bash"
            code=r#"
                kivro status --env production
                kivro status --project ../launcher
                kivro status --project ../launcher/.kivro.toml
            "#
        />
    }
}

fn exit_codes() -> impl IntoView {
    view! {
        {spec_table(
            &["Code", "Meaning"],
            &[
                &["0", "Success"],
                &["1", "Generic failure"],
                &["2", "Usage error (bad arguments)"],
                &["3", "Required secrets missing"],
                &["4", "Credential store unavailable"],
                &["5", "Manifest missing, invalid, or newer than this CLI"],
                &["6", "Bundle failed to decrypt or verify"],
                &["7", "doctor found problems"],
                &["8", "Cancelled by the user"],
                &["*", "For run: the child's exit code, or 128 + signal"],
            ],
        )}

        <Callout tone=Tone::Accent title="code 3 is the build hook">
            <InlineCode>"kivro status"</InlineCode>
            " exiting 3 is not an error condition to work around — it is the designed interface "
            "for a build system to ask " <em>"can this project run right now"</em> "."
        </Callout>

        <CodeBlock
            language="bash"
            code=r#"
                kivro status --quiet || exit 1
            "#
        />
    }
}

fn init() -> impl IntoView {
    view! {
        <CommandHeading
            name="init"
            usage="kivro init [--name NAME] [--default-env NAME] [--force]"
            summary="Creates .kivro.toml in the current (or --project) directory."
        />

        {spec_table(
            &["Flag", "Default", "Meaning"],
            &[
                &["--name <NAME>", "the directory name, sanitised", "Project identity, and the second level of the storage namespace."],
                &["--default-env <NAME>", "dev", "Value written to [environment] default."],
                &["--force", "off", "Overwrite an existing manifest. Without it, an existing file is an error."],
            ],
        )}

        <P>
            "It never creates a " <InlineCode>".env"</InlineCode>
            ", never generates a value, and refuses to overwrite an existing manifest unless told "
            "to. A directory name that cannot be sanitised into a valid project name is an error "
            "asking for " <InlineCode>"--name"</InlineCode> "."
        </P>
    }
}

fn values() -> impl IntoView {
    view! {
        <CommandHeading
            name="set"
            usage="kivro set NAME [--stdin] [--no-confirm]"
            summary="Stores a value. Prompts twice without echoing by default."
        />
        <P>
            "A value cannot be passed as an argument — that is what puts secrets into shell "
            "history and " <InlineCode>"ps"</InlineCode> " output. "
            <InlineCode>"--stdin"</InlineCode>
            " reads to EOF and strips exactly one trailing newline, so multi-line values such as "
            "private keys survive intact. " <InlineCode>"--no-confirm"</InlineCode>
            " prompts once instead of twice. Setting a name the manifest does not declare warns, "
            "then succeeds."
        </P>
        <CodeBlock
            language="bash"
            code=r#"
                kivro set AUTH0_CLIENT_SECRET                              # interactive
                printf '%s' "$SECRET" | kivro set AUTH0_CLIENT_SECRET --stdin   # CI
            "#
        />

        <CommandHeading
            name="get"
            usage="kivro get NAME [--show]"
            summary="Reports presence and length. --show prints the value."
        />
        <P>
            "Without " <InlineCode>"--show"</InlineCode>
            " the output is presence and byte length only. With it, the value goes to stdout and "
            "a warning is printed when stdout is a terminal — because a value on your screen is a "
            "value in your scrollback."
        </P>

        <CommandHeading
            name="list"
            usage="kivro list [--all]"
            summary="Names and presence. Never values."
        />
        <P>
            <InlineCode>"--all"</InlineCode>
            " additionally includes stored secrets the manifest does not declare, which is how you "
            "find leftovers from a renamed variable."
        </P>

        <CommandHeading
            name="remove"
            usage="kivro remove NAME [--yes]"
            summary="Deletes one value after confirmation."
        />
        <P>
            <InlineCode>"-y"</InlineCode> " / " <InlineCode>"--yes"</InlineCode>
            " skips the prompt for scripts. Removing a name that has no value is not an error."
        </P>
    }
}

fn status() -> impl IntoView {
    view! {
        <CommandHeading
            name="status"
            usage="kivro status"
            summary="Required, optional and undeclared secrets for the resolved environment."
        />

        <CodeBlock
            language="text"
            code=r#"
                launcher / dev

                Required secrets:

                  ✓ AUTH0_CLIENT_ID
                  ✗ AUTH0_CLIENT_SECRET
                  ✓ DATABASE_URL

                1 secret missing.

                Run:
                    kivro set AUTH0_CLIENT_SECRET
            "#
        />

        <P>
            "Exit code 3 when a required secret is missing, 0 otherwise. Optional secrets are "
            "listed separately and never affect the exit code. Values stored without a "
            "declaration appear under their own heading so that they do not become invisible."
        </P>
    }
}

fn doctor() -> impl IntoView {
    view! {
        <CommandHeading
            name="doctor"
            usage="kivro doctor [--fix-gitignore]"
            summary="Diagnoses the manifest, the store, the environment and git hygiene."
        />

        {prose_table(
            &["Area", "Checks"],
            &[
                &["Manifest", "found · parses · project identity · CLI version compatibility · unrecognised keys"],
                &["Store", "which backend · reachable · whether it is OS-protected"],
                &["Environment", "which environment resolved · required secrets present · deprecated secrets still stored"],
                &["Git hygiene", "stray .env · whether it is ignored · recommended .gitignore entries · bundles left in the project root"],
            ],
        )}

        <P>
            "Exit code 7 when a check fails; warnings alone exit 0. "
            <InlineCode>"--fix-gitignore"</InlineCode> " appends "
            <InlineCode>".env"</InlineCode> ", " <InlineCode>".env.*"</InlineCode> " and "
            <InlineCode>"*.kivro"</InlineCode>
            " after asking — never silently."
        </P>

        <Callout tone=Tone::Warning title="a .env containing secrets, unignored">
            "That specific combination is a " <em>"failure"</em>
            ", not a warning, and it is checked by looking at the file's contents rather than its "
            "name. It is the exact state that ends with credentials in a public repository."
        </Callout>
    }
}

fn run_cmd() -> impl IntoView {
    view! {
        <CommandHeading
            name="run"
            usage="kivro run [--no-inherit] -- COMMAND..."
            summary="Loads the environment's secrets, spawns the command, propagates its exit status."
        />

        <CodeBlock
            language="bash"
            code=r#"
                kivro run -- cargo run
                kivro run --env staging -- npm run dev
                kivro run --no-inherit -- ./deploy.sh
                kivro run -- $SHELL             # last resort; see the caveat below
            "#
        />

        {spec_table(
            &["Behaviour", "Detail"],
            &[
                &["Fails first", "A missing required secret is an error before the child is spawned. Exit code 3."],
                &["No file", "Values go into the child's environment block. Nothing is written to disk."],
                &["--no-inherit", "Clears the parent environment, keeping PATH, HOME, USER, SHELL, TMPDIR/TEMP/TMP, LANG, LC_ALL, TERM and the Windows equivalents."],
                &["Ctrl-C", "Ignored by the parent so the child receives it directly; its status is still propagated."],
                &["Exit status", "The child's code, or 128 + signal on Unix."],
            ],
        )}

        <P>
            "Unless " <InlineCode>"--quiet"</InlineCode>
            " is set, one dimmed line is printed before the child starts, naming the project, the "
            "environment and how many secrets were injected — never which, and never their values."
        </P>

        <Callout tone=Tone::Warning title="wrapping a shell">
            <InlineCode>"kivro run -- $SHELL"</InlineCode>
            " starts an interactive shell with the secrets present, and every command you run in "
            "it inherits them — including anything that dumps its environment into a log. Prefer "
            "wrapping the specific command."
        </Callout>
    }
}

fn envfiles() -> impl IntoView {
    view! {
        <CommandHeading
            name="import"
            usage="kivro import [PATH] [--force] [--delete-source]"
            summary="Parses a .env (default .env) and stores every valid entry."
        />
        <P>
            "Accepts comments, blank lines, an optional " <InlineCode>"export "</InlineCode>
            " prefix, and single- or double-quoted values. Keys that are not valid secret names "
            "are reported as skipped, with their line numbers, rather than dropped in silence. "
            "Existing stored values are kept unless " <InlineCode>"--force"</InlineCode>
            " is passed. The source file is never deleted automatically; "
            <InlineCode>"--delete-source"</InlineCode> " still asks."
        </P>

        <CommandHeading
            name="export"
            usage="kivro export [-o PATH] [--force] [--yes]"
            summary="Writes a .env. Explicit, confirmed, and warned about."
        />
        <Callout tone=Tone::Danger title="this re-creates the problem">
            "Writing a " <InlineCode>".env"</InlineCode>
            " puts your credentials back into a plaintext file on disk. The command therefore "
            "requires confirmation (or " <InlineCode>"--yes"</InlineCode>
            "), refuses to overwrite without " <InlineCode>"--force"</InlineCode>
            ", and creates the file " <InlineCode>"0600"</InlineCode>
            " on Unix. Use it only for tools that genuinely cannot read anything else, and delete "
            "it afterwards."
        </Callout>
    }
}

fn bundles() -> impl IntoView {
    view! {
        <CommandHeading
            name="share"
            usage="kivro share [-o PATH] [--recipient KEY]... [--all] [--hint-names] [--force]"
            summary="Creates an encrypted bundle of the environment's stored secrets."
        />

        {spec_table(
            &["Flag", "Meaning"],
            &[
                &["-o, --out <PATH>", "Output path. Defaults to <project>.<environment>.kivro in the project root."],
                &["--recipient <AGE_PUBLIC_KEY>", "Encrypt to one or more age public keys instead of a passphrase. Repeatable."],
                &["--all", "Include stored secrets the manifest does not declare."],
                &["--hint-names", "Record the variable names in the file's unencrypted header."],
                &["--force", "Overwrite an existing file at the output path."],
            ],
        )}

        <P>
            "Passphrase mode is the default and prompts twice, warning when the passphrase is "
            "shorter than twelve characters. In a non-interactive session it requires either "
            <InlineCode>"--recipient"</InlineCode> " or " <InlineCode>"KIVRO_PASSPHRASE"</InlineCode>
            ", and says so rather than hanging on a prompt nobody can answer."
        </P>

        <CommandHeading
            name="accept"
            usage="kivro accept PATH [--identity PATH] [--force]"
            summary="Decrypts a bundle into the credential store."
        />
        <P>
            "Verifies that the bundle's authenticated project matches the local manifest, so a "
            "bundle for another project is refused whatever its filename says. Existing values "
            "are kept unless " <InlineCode>"--force"</InlineCode> " is passed. "
            <InlineCode>"--identity"</InlineCode>
            " supplies an age identity file for bundles created with "
            <InlineCode>"--recipient"</InlineCode> "."
        </P>

        <P>
            "The format, and why the unencrypted hint is treated as advisory, are documented on "
            "the " <DocLink to=nav::doc_path("bundles")>"bundles page"</DocLink> "."
        </P>
    }
}

fn sync() -> impl IntoView {
    view! {
        <CommandHeading
            name="sync"
            usage="kivro sync [--apply]"
            summary="Compares the manifest, the local store, and the configured [sync] source."
        />

        <P>
            "Reports what is present, what is missing, and which of the missing the source can "
            "supply. It writes nothing without " <InlineCode>"--apply"</InlineCode>
            ". With no " <InlineCode>"[sync]"</InlineCode>
            " section in the manifest it still reports present and missing, and says there is no "
            "source configured."
        </P>

        <CodeBlock
            language="toml"
            title=".kivro.toml"
            code=r#"
                [sync]
                kind = "file"
                path = "team-secrets"    # a directory of bundles, relative to the project root
            "#
        />

        <P>
            <InlineCode>"file"</InlineCode>
            " is the only backend in 0.1: a directory of encrypted bundles, which needs no server "
            "and travels over anything a team already has. Other sources implement one trait — "
            "see " <DocLink to=nav::doc_path("architecture")>"architecture"</DocLink> "."
        </P>
    }
}

fn env_vars() -> impl IntoView {
    view! {
        {spec_table(
            &["Variable", "Meaning"],
            &[
                &["KIVRO_ENV", "Environment to use. Below --env, above the manifest default."],
                &["KIVRO_STORE", "keyring (default), file, or memory. An unrecognised value is an error."],
                &["KIVRO_STORE_FILE", "Path for KIVRO_STORE=file. Required when that backend is selected."],
                &["KIVRO_CONFIG_DIR", "Override the configuration directory."],
                &["KIVRO_PASSPHRASE", "Bundle passphrase for non-interactive use."],
                &["NO_COLOR", "Disable colour, like --no-color."],
            ],
        )}

        <Callout tone=Tone::Warning title="KIVRO_PASSPHRASE">
            "A passphrase in the environment is visible to child processes and to anything that "
            "dumps the environment. Prefer " <InlineCode>"--recipient"</InlineCode>
            " with age public keys for automation, and the interactive prompt for humans."
        </Callout>
    }
}

fn config() -> impl IntoView {
    view! {
        <P>
            "Per-machine, non-secret preferences. The path is platform-appropriate — "
            <InlineCode>"~/.config/kivro/config.toml"</InlineCode> ", "
            <InlineCode>"%APPDATA%\\kivro\\config.toml"</InlineCode> ", "
            <InlineCode>"~/Library/Application Support/kivro/config.toml"</InlineCode>
            " — and " <InlineCode>"KIVRO_CONFIG_DIR"</InlineCode> " overrides it. A missing file "
            "is not an error; every key has a default."
        </P>

        <CodeBlock
            language="toml"
            title="config.toml"
            code=r#"
                [defaults]
                environment = "dev"      # used only when the manifest declares no default

                [ui]
                color = true

                [storage]
                namespace = "kivro-secrets"   # first level of the storage namespace
            "#
        />

        <Callout tone=Tone::Danger title="changing the namespace orphans values">
            <InlineCode>"[storage] namespace"</InlineCode>
            " is the first component of every credential's address. Changing it points kivro at a "
            "different set of credentials; it does not migrate the existing ones."
        </Callout>
    }
}

fn json() -> impl IntoView {
    view! {
        <P>
            <InlineCode>"--json"</InlineCode>
            " emits a single JSON document on stdout. No command includes a value except "
            <InlineCode>"get --show"</InlineCode>
            ", which is checked by a test that fails if a value ever appears in JSON output."
        </P>

        {spec_table(
            &["Command", "Shape"],
            &[
                &["init", "{ created, project, environment }"],
                &["set", "{ name, environment, action: \"created\" | \"updated\" }"],
                &["get", "{ name, present, length }"],
                &["list", "{ project, environment, secrets: [{ name, present, required, declared }] }"],
                &["remove", "{ name, removed }"],
                &["status", "{ project, environment, satisfied, missing, secrets: [...] }"],
                &["doctor", "{ status: \"ok\" | \"warning\" | \"error\", checks: [{ level, title, detail }] }"],
                &["import", "{ source, imported, skipped_existing, skipped_invalid }"],
                &["export", "{ written, count, names }"],
                &["share", "{ written, count, cipher }"],
                &["accept", "{ project, environment, stored, skipped_existing }"],
                &["sync", "{ project, environment, source, present, missing, fetchable, unavailable, fetched }"],
            ],
        )}

        <H3>"Errors"</H3>
        <P>
            "A failure under " <InlineCode>"--json"</InlineCode>
            " still prints JSON, and still exits with the documented code. "
            <InlineCode>"kind"</InlineCode>
            " is a stable machine-readable string; the message and hint are for humans."
        </P>
        <CodeBlock
            language="json"
            code=r#"
                {
                  "error": {
                    "kind": "missing_secret",
                    "message": "2 required secret(s) missing for launcher/dev: AUTH0_CLIENT_SECRET, S3_ACCESS_KEY",
                    "hint": "run:\n    kivro set AUTH0_CLIENT_SECRET\n    kivro set S3_ACCESS_KEY"
                  }
                }
            "#
        />
        <P>
            "The full list of " <InlineCode>"kind"</InlineCode> " values is on the "
            <DocLink to=nav::doc_path("troubleshooting")>"troubleshooting page"</DocLink>
            ", each with what causes it and what fixes it."
        </P>
    }
}
