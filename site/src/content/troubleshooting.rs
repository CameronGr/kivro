//! `/docs/troubleshooting` — error kinds, exit codes, and the questions people actually ask.

use crate::content::kit::*;
use crate::nav;
use crate::ui::prelude::*;

pub fn doc() -> Doc {
    Doc::new("troubleshooting", "Troubleshooting", "Guide", "Guides")
        .tagline(
            "Every error carries its own fix — `hint()` returns the command to run. This page is \
             the longer version: what each error kind means, what causes it, and what to do \
             about it.",
        )
        .tags(["error kinds", "exit codes", "FAQ"])
        .section(
            DocSection::new("first", "Start here", first)
                .numbered("1.0")
                .summary("One command that diagnoses most of what goes wrong."),
        )
        .section(
            DocSection::new("kinds", "Error kinds", kinds)
                .numbered("1.1")
                .summary("The stable strings in --json output, and what each one means."),
        )
        .section(
            DocSection::new("store-problems", "The credential store", store_problems)
                .numbered("2.0")
                .summary("Exit code 4, and what it looks like per platform."),
        )
        .section(
            DocSection::new("manifest-problems", "Manifest and environment", manifest_problems)
                .numbered("2.1")
                .summary("Exit code 5, plus the environment mistakes that look like empty secrets."),
        )
        .section(
            DocSection::new("bundle-problems", "Bundles", bundle_problems)
                .numbered("2.2")
                .summary("Exit code 6: decryption, verification, and mismatched projects."),
        )
        .section(
            DocSection::new("faq", "Frequently asked", faq)
                .numbered("3.0")
                .summary("Behaviour that is intentional but surprising the first time."),
        )
}

fn first() -> impl IntoView {
    view! {
        <CommandLine command="kivro doctor" />
        <P>
            "It checks the manifest, the store, the resolved environment, required secrets, "
            "deprecated values, and git hygiene, and reports each as ok, warning or error. Exit "
            "code 7 means at least one check failed."
        </P>
        <P>
            "If " <InlineCode>"doctor"</InlineCode> " is clean and something still misbehaves, run "
            "the failing command with " <InlineCode>"--json"</InlineCode>
            ": the " <InlineCode>"kind"</InlineCode>
            " field is the stable identifier to look up below."
        </P>
    }
}

fn kinds() -> impl IntoView {
    view! {
        {spec_table(
            &["kind", "Exit", "Cause and fix"],
            &[
                &["manifest_not_found", "5", "No .kivro.toml here or in any parent. Run kivro init in the project root, or pass --project."],
                &["manifest_invalid", "5", "The TOML does not parse, or a required key is missing. The message carries the path and the parser's complaint."],
                &["manifest_too_new", "5", "The manifest declares a format newer than this build supports. Upgrade the CLI."],
                &["cli_too_old", "5", "[meta] min_cli_version is above the running version. Upgrade the CLI."],
                &["invalid_name", "1", "A project, environment or secret name breaks its grammar. Secret names are uppercase only."],
                &["unknown_environment", "1", "The environment is not declared and strict is on. The message lists the declared ones."],
                &["no_environment", "1", "Nothing selected one. Pass --env, set KIVRO_ENV, or add [environment] default."],
                &["missing_secret", "3", "A required secret has no stored value. The hint lists a kivro set line per name."],
                &["store_unavailable", "4", "The credential store cannot be reached. Run kivro doctor for the backend-specific detail."],
                &["store_error", "1", "The store was reachable but the operation failed. Mapped by hand so no credential bytes leak into the message."],
                &["crypto_error", "6", "Encryption or decryption failed — most often a wrong passphrase or identity."],
                &["bundle_format", "6", "Not a bundle, or an envelope format newer than this build reads."],
                &["bundle_mismatch", "6", "The unauthenticated hint disagrees with the authenticated payload, or the bundle is for another project."],
                &["env_format", "1", "A .env line could not be parsed. The message carries the file and line number."],
                &["sync_error", "1", "The sync source could not be read, or is read-only for the operation attempted."],
                &["config_invalid", "1", "~/.config/kivro/config.toml does not parse. The message carries the path."],
                &["cancelled", "8", "You declined a confirmation prompt. Nothing was written."],
                &["already_exists", "1", "The target file exists. Pass --force to overwrite."],
                &["io_error", "1", "A filesystem operation failed; the message names the operation and the path."],
            ],
        )}
    }
}

fn store_problems() -> impl IntoView {
    view! {
        <Accordion
            title="Linux: store_unavailable, or a D-Bus error"
            summary="No Secret Service is running, or no session bus is available"
            icon=icons::TRIANGLE_ALERT
        >
            <P>
                "The Secret Service API is a D-Bus interface. On a desktop, GNOME Keyring, KWallet "
                "or KeePassXC provides it and it is already running. Over SSH, in a container, or "
                "on a bare server it usually is not."
            </P>
            <CodeBlock
                language="bash"
                dense=true
                code=r#"
                    # is anything providing it?
                    busctl --user list | grep -i secrets

                    # build dependency, if the install itself failed
                    sudo apt install libdbus-1-dev pkg-config
                "#
            />
            <P>
                "For CI and containers, use " <InlineCode>"KIVRO_STORE=file"</InlineCode>
                " with a path under the job's temporary directory — after reading the "
                <DocLink to=nav::doc_path("security")>"security page"</DocLink>
                " on what that gives up."
            </P>
        </Accordion>

        <Accordion
            title="macOS: a prompt on every run"
            summary="The keychain does not recognise the binary yet"
            icon=icons::INFO
        >
            <P>
                "The first access from a newly built or newly installed binary prompts for "
                "permission. Choosing " <em>"Always Allow"</em>
                " stops it for that binary. Rebuilding the CLI produces a different binary, so a "
                "prompt after " <InlineCode>"cargo install --force"</InlineCode> " is expected."
            </P>
        </Accordion>

        <Accordion
            title="Windows: values are there, but kivro cannot see them"
            summary="Almost always the namespace or the project name"
            icon=icons::WRENCH
        >
            <P>
                "Credentials are addressed by "
                <InlineCode>"namespace:project:environment"</InlineCode>
                ". Changing " <InlineCode>"[project] name"</InlineCode> " or "
                <InlineCode>"[storage] namespace"</InlineCode>
                " points at a different address; the old values are still in Credential Manager "
                "under the old service string. Open Credential Manager and look at the service "
                "names to confirm which is which."
            </P>
        </Accordion>

        <Accordion
            title="KIVRO_STORE is set and everything looks empty"
            summary="A different backend is a different set of values"
            icon=icons::CIRCLE_ALERT
        >
            <P>
                "Each backend is a separate world. " <InlineCode>"KIVRO_STORE=file"</InlineCode>
                " in a shell profile is a common cause of " <em>"my secrets disappeared"</em>
                ". " <InlineCode>"kivro doctor"</InlineCode>
                " names the active backend on every run, and warns whenever the active one is not "
                "OS-protected."
            </P>
        </Accordion>
    }
}

fn manifest_problems() -> impl IntoView {
    view! {
        <Accordion
            title="manifest_not_found in a subdirectory"
            summary="Discovery walks up, not down"
            icon=icons::FILE_TEXT
            open_by_default=true
        >
            <P>
                "Discovery starts at the working directory and walks up looking for "
                <InlineCode>".kivro.toml"</InlineCode>
                ". Running from a sibling directory, or from a monorepo root above the manifest, "
                "finds nothing. Pass " <InlineCode>"--project"</InlineCode>
                " with the directory or the manifest path."
            </P>
        </Accordion>

        <Accordion
            title="unknown_environment: prod"
            summary="strict is on, and the manifest spells it differently"
            icon=icons::CIRCLE_ALERT
        >
            <P>
                "The error lists the declared environments. This is the intended behaviour: "
                "resolving " <InlineCode>"prod"</InlineCode> " to an empty secret set would let "
                <InlineCode>"kivro run"</InlineCode>
                " start your application against nothing at all. Set "
                <InlineCode>"[environment] strict = false"</InlineCode>
                " only if you genuinely want ad-hoc scopes."
            </P>
        </Accordion>

        <Accordion
            title="run does not inject a value I definitely set"
            summary="It is stored, but not declared"
            icon=icons::TRIANGLE_ALERT
        >
            <P>
                <InlineCode>"kivro run"</InlineCode>
                " loads exactly the names the manifest declares for the resolved environment. A "
                "value set without a declaration is stored and reported by "
                <InlineCode>"status"</InlineCode> " and " <InlineCode>"list --all"</InlineCode>
                ", but never injected. Add it to " <InlineCode>"[variables]"</InlineCode>
                " and it appears."
            </P>
        </Accordion>

        <Accordion
            title="A variable is set in dev but missing in production"
            summary="Environments are separate scopes, on purpose"
            icon=icons::INFO
        >
            <P>
                "There is no inheritance of " <em>"values"</em>
                " between environments — only of " <em>"declarations"</em>
                ". Set it again with " <InlineCode>"--env production"</InlineCode>
                ", or move it with " <InlineCode>"share"</InlineCode> " / "
                <InlineCode>"accept"</InlineCode> "."
            </P>
        </Accordion>
    }
}

fn bundle_problems() -> impl IntoView {
    view! {
        <Accordion
            title="crypto_error on accept"
            summary="Wrong passphrase, or the wrong identity file"
            icon=icons::SHIELD_ALERT
        >
            <P>
                "age reports a decryption failure without saying why, because saying why would be "
                "an oracle. Check that the bundle's "
                <InlineCode>"cipher"</InlineCode> " field matches what you are supplying: "
                <InlineCode>"age-v1-scrypt"</InlineCode> " needs a passphrase, "
                <InlineCode>"age-v1-x25519"</InlineCode> " needs " <InlineCode>"--identity"</InlineCode>
                ". The envelope is plain JSON, so you can read that field without decrypting "
                "anything."
            </P>
        </Accordion>

        <Accordion
            title="bundle_mismatch"
            summary="The hint and the payload disagree, or the project is not yours"
            icon=icons::TRIANGLE_ALERT
        >
            <P>
                "Two different checks produce this. Either the unauthenticated header claims "
                "something the authenticated payload contradicts — which means the file has been "
                "edited — or the payload's project does not match your local manifest. Neither is "
                "recoverable by retrying; ask the sender for a fresh bundle."
            </P>
        </Accordion>

        <Accordion
            title="share refuses in a script"
            summary="Passphrase mode needs a terminal"
            icon=icons::TERMINAL
        >
            <P>
                "Without a TTY there is nowhere to prompt, so "
                <InlineCode>"share"</InlineCode> " requires either "
                <InlineCode>"--recipient"</InlineCode> " with an age public key, or "
                <InlineCode>"KIVRO_PASSPHRASE"</InlineCode>
                " in the environment. Prefer the first: an environment variable is visible to "
                "every child process."
            </P>
        </Accordion>
    }
}

fn faq() -> impl IntoView {
    view! {
        <Accordion
            title="Can I put the manifest in git?"
            summary="Yes — that is what it is for"
            icon=icons::CHECK
            open_by_default=true
        >
            <P>
                "It declares names, not values, and it cannot hold a value. Committing it is how "
                "everyone on the team gets the same list of what a project needs."
            </P>
        </Accordion>

        <Accordion
            title="What happens to my secrets if I uninstall kivro?"
            summary="Nothing — they belong to the OS store"
            icon=icons::INFO
        >
            <P>
                "They stay exactly where they are, under the same service names, and are visible "
                "in Credential Manager, Keychain Access, or your Secret Service front-end. "
                "Uninstalling removes the tool, not the data."
            </P>
        </Accordion>

        <Accordion
            title="Can two projects use the same variable name?"
            summary="Yes; they are unrelated entries"
            icon=icons::CHECK
        >
            <P>
                "The full address includes the project and environment, and no name component can "
                "contain a colon, so the rendering is injective. "
                <InlineCode>"DATABASE_URL"</InlineCode>
                " in two projects can never collide."
            </P>
        </Accordion>

        <Accordion
            title="Why can't I pass a value on the command line?"
            summary="Because that is the leak this tool exists to close"
            icon=icons::SHIELD_ALERT
        >
            <P>
                "Arguments end up in shell history, in " <InlineCode>"ps"</InlineCode>
                " output, and in any shell integration that logs commands. "
                <InlineCode>"--stdin"</InlineCode>
                " covers every legitimate scripted case without those consequences."
            </P>
        </Accordion>

        <Accordion
            title="Does run write a .env anywhere?"
            summary="No. Never, under any flag"
            icon=icons::CHECK
        >
            <P>
                "Values go into the child's environment block directly. The only command that "
                "writes a " <InlineCode>".env"</InlineCode> " is "
                <InlineCode>"kivro export"</InlineCode>
                ", which asks first and warns while it does it."
            </P>
        </Accordion>

        <Accordion
            title="Can I use it with a monorepo?"
            summary="Yes — one manifest per project, or one at the root"
            icon=icons::WRENCH
        >
            <P>
                "Discovery walks up from the working directory, so a manifest per package works "
                "naturally, each with its own project name and its own storage scope. A single "
                "root manifest shared by every package works too; pick based on whether the "
                "packages genuinely share a secret set."
            </P>
        </Accordion>

        <Accordion
            title="Is there a hosted or team server?"
            summary="Not in 0.1 — bundles and a shared directory are the team story"
            icon=icons::INFO
        >
            <P>
                "The " <InlineCode>"SyncSource"</InlineCode>
                " trait exists so an HTTP or object-storage backend can be added without changing "
                "the format or anything above that layer, but 0.1 ships the bundle-directory "
                "source only. See the " <DocLink to=nav::doc_path("roadmap")>"status page"</DocLink>
                "."
            </P>
        </Accordion>
    }
}
