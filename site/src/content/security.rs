//! `/docs/security` — what is protected, what is not, and where the limits are.

use crate::content::kit::*;
use crate::nav;
use crate::ui::prelude::*;

pub fn doc() -> Doc {
    Doc::new("security", "Security model", "Reference", "Design & security")
        .tagline(
            "What the tool actually protects against, and what it does not. Where a guarantee is \
             weaker than it might appear, it is said plainly here rather than glossed.",
        )
        .tags(["threat model", "honest limits", "no bespoke crypto"])
        .section(
            DocSection::new("does", "What the tool does", does)
                .numbered("1.0")
                .summary("Five properties, each of which closes a specific leak."),
        )
        .section(
            DocSection::new("in-scope", "Threats in scope", in_scope)
                .numbered("1.1")
                .summary("What is mitigated, and by which mechanism."),
        )
        .section(
            DocSection::new("out-of-scope", "Threats out of scope", out_of_scope)
                .numbered("1.2")
                .summary("What no local secret manager can offer, stated up front."),
        )
        .section(
            DocSection::new("memory", "Memory handling", memory)
                .numbered("2.0")
                .summary("Zeroization narrows a window. It is not a boundary."),
        )
        .section(
            DocSection::new("crypto-position", "Cryptography", crypto_position)
                .numbered("2.1")
                .summary("None is implemented here, and that is the design decision."),
        )
        .section(
            DocSection::new("file-store", "The insecure file store", file_store)
                .numbered("2.2")
                .summary("Why a plaintext backend exists, and the four rails around it."),
        )
        .section(
            DocSection::new("passphrase-env", "KIVRO_PASSPHRASE", passphrase_env)
                .numbered("2.3")
                .summary("A convenience with a cost worth stating."),
        )
        .section(
            DocSection::new("export-warning", "kivro export", export_warning)
                .numbered("2.4")
                .summary("The command that undoes the tool, and why it still exists."),
        )
        .section(
            DocSection::new("leak-tests", "Tests that exist to catch leaks", leak_tests)
                .numbered("3.0")
                .summary("Not logic errors — disclosure. A different kind of test."),
        )
        .section(
            DocSection::new("reporting", "Reporting a vulnerability", reporting)
                .numbered("3.1")
                .summary("Privately, to the maintainers."),
        )
}

fn does() -> impl IntoView {
    view! {
        <List>
            <ListItem>
                "Stores values in the OS credential store — Credential Manager, Keychain, Secret "
                "Service — instead of plaintext files."
            </ListItem>
            <ListItem>
                "Keeps values out of the repository: the manifest holds declarations only."
            </ListItem>
            <ListItem>
                "Injects values directly into a child process environment. No intermediate file, "
                "no shell argument."
            </ListItem>
            <ListItem>
                "Encrypts values with age when they have to move between machines."
            </ListItem>
            <ListItem>
                "Fails loudly rather than degrading: an unavailable keyring is an error, never a "
                "silent fallback to something weaker."
            </ListItem>
        </List>
    }
}

fn in_scope() -> impl IntoView {
    view! {
        {prose_table(
            &["Threat", "Mitigation"],
            &[
                &[
                    "Secrets committed to git",
                    "Values are never in a file the repository tracks; doctor flags a stray .env and missing .gitignore entries",
                ],
                &[
                    "Secrets in shell history",
                    "set prompts; a value cannot be passed as an argument. --stdin exists for CI",
                ],
                &[
                    "Secrets in ps output or argv",
                    "Values are only ever passed through the environment block, never argv",
                ],
                &[
                    "Secrets in logs, errors and crash dumps",
                    "SecretString has no Display and no Serialize; Debug is redacted; no error variant carries a value; keyring errors that could carry credential bytes are mapped by hand",
                ],
                &[
                    "Secrets in terminal scrollback",
                    "Nothing prints a value except get --show, which warns when stdout is a terminal",
                ],
                &[
                    "Secrets left on disk after sharing",
                    "Bundles are encrypted at rest; doctor warns about bundles left in the project root",
                ],
                &[
                    "A tampered bundle",
                    "age authenticates the ciphertext; the unauthenticated header is cross-checked against the authenticated payload and mismatches are refused",
                ],
                &[
                    "A bundle from the wrong project",
                    "accept compares the authenticated payload's project against the local manifest",
                ],
                &[
                    "Cross-project collision",
                    "Names are validated so the app:project:environment namespace is injective",
                ],
                &[
                    "A hostile bundle burning CPU",
                    "The accepted scrypt work factor is bounded at 2^20",
                ],
            ],
        )}
    }
}

fn out_of_scope() -> impl IntoView {
    view! {
        {prose_table(
            &["Threat", "Why"],
            &[
                &[
                    "A compromised developer machine",
                    "Anything that can run code as you can ask the credential store for the same secrets this tool can. That is inherent to any local secret manager",
                ],
                &[
                    "Malicious child processes",
                    "kivro run hands the secrets to the command you named. It cannot police what that command does with them; supply-chain risk in your dependencies is unchanged",
                ],
                &[
                    "Another process reading /proc/<pid>/environ",
                    "On Linux this is restricted to the same user and root. Environment variables are the standard interface; that is the trade-off they carry",
                ],
                &[
                    "Memory forensics, swap, core dumps",
                    "Best-effort only — see the next section",
                ],
                &[
                    "A malicious .kivro.toml",
                    "The manifest is code-reviewed content in your repository. It cannot carry values, but a hostile edit could add declarations. Review it like any other file",
                ],
                &[
                    "Insider access",
                    "There is no per-secret access control in 0.1. Anyone you hand a bundle to has the values",
                ],
            ],
        )}

        <Callout tone=Tone::Info title="the point of writing these down">
            "A security tool that lists only its strengths teaches you to trust it in situations it "
            "was never built for. The out-of-scope table is the more useful half of this page."
        </Callout>
    }
}

fn memory() -> impl IntoView {
    view! {
        <P>
            <InlineCode>"SecretString"</InlineCode> " wraps a "
            <InlineCode>"zeroize::Zeroizing<String>"</InlineCode>
            ", so the heap buffer is overwritten when the value is dropped, and the serialised "
            "bundle payload is zeroized after encryption."
        </P>

        <Callout tone=Tone::Warning title="this is not a guarantee that a secret exists in exactly one place">
            "It cannot be, in a language where the allocator is free to copy:"
        </Callout>

        <List>
            <ListItem>
                <InlineCode>"String"</InlineCode>
                " growth reallocates and leaves the old buffer's contents behind."
            </ListItem>
            <ListItem>"Moves and clones copy bytes nothing tracks."</ListItem>
            <ListItem>"The OS may page memory to swap or write it to a core dump."</ListItem>
            <ListItem>
                "Between " <InlineCode>"expose_secret()"</InlineCode>
                " and the kernel receiving the child's environment block, the value exists as an "
                "ordinary allocation."
            </ListItem>
            <ListItem>
                "The " <InlineCode>"keyring"</InlineCode> " and " <InlineCode>"age"</InlineCode>
                " crates hold their own copies while working."
            </ListItem>
        </List>

        <P>
            "Zeroization here reduces the window and the number of stale copies. Treat it as "
            "defence in depth, not as a boundary you can rely on."
        </P>
    }
}

fn crypto_position() -> impl IntoView {
    view! {
        <P>
            "No cryptographic primitive or protocol is implemented in this project. Bundles use "
            "the " <InlineCode>"age"</InlineCode>
            " crate: ChaCha20-Poly1305 over the STREAM construction, HMAC-SHA-256 over the header, "
            "scrypt for passphrases, X25519 + HKDF for public keys."
        </P>

        <P>
            "Passphrases are never used as keys directly — age's scrypt recipient handles "
            "stretching, with the work factor carried in the file and bounded on read. The format "
            "and its versioning strategy are on the "
            <DocLink to=nav::doc_path("bundles")>"bundles page"</DocLink> "."
        </P>

        <Callout tone=Tone::Accent title="the boring choice on purpose">
            "A bespoke format would need review this project cannot buy. age is specified, widely "
            "reviewed, and interoperable, which also means a bundle stays readable with the "
            "standard " <InlineCode>"age"</InlineCode> " CLI if this tool disappears tomorrow."
        </Callout>
    }
}

fn file_store() -> impl IntoView {
    view! {
        <P>
            <InlineCode>"KIVRO_STORE=file"</InlineCode>
            " selects a plaintext JSON store. It exists because CI containers have no D-Bus "
            "session, Keychain or Credential Manager, and testing the real binary end to end is "
            "worth more than testing a mock."
        </P>

        <H3>"Safeguards"</H3>
        <List>
            <ListItem>
                "It is never a default and never a fallback — an unavailable keyring is an error, "
                "and an unrecognised " <InlineCode>"KIVRO_STORE"</InlineCode>
                " value is rejected rather than guessed."
            </ListItem>
            <ListItem>
                "The CLI prints a warning on " <strong class="text-white/85">"every"</strong>
                " command that uses it."
            </ListItem>
            <ListItem>
                <InlineCode>"doctor"</InlineCode> " reports it as a finding."
            </ListItem>
            <ListItem>
                "Files are created " <InlineCode>"0600"</InlineCode> " on Unix."
            </ListItem>
        </List>

        <Callout tone=Tone::Danger title="it provides no confidentiality">
            "Against anything but other users on the same machine. Do not use it for real "
            "credentials on a machine you care about. In CI, scope it to the job's temporary "
            "directory and let your CI provider's own secret storage remain the source of truth."
        </Callout>
    }
}

fn passphrase_env() -> impl IntoView {
    view! {
        <P>
            "Supplies a bundle passphrase without a terminal, for CI. A passphrase in the "
            "environment is visible to child processes and to anything that dumps the environment "
            "— including, on Linux, anything that can read "
            <InlineCode>"/proc/<pid>/environ"</InlineCode> " as the same user."
        </P>
        <P>
            "Prefer " <InlineCode>"--recipient"</InlineCode>
            " with age public keys for automation, and the interactive prompt for humans."
        </P>
    }
}

fn export_warning() -> impl IntoView {
    view! {
        <Callout tone=Tone::Danger title="export re-creates exactly the problem this tool exists to solve">
            "It is therefore explicit, never implicit: it requires confirmation (or "
            <InlineCode>"--yes"</InlineCode>
            "), warns about plaintext on disk, refuses to overwrite without "
            <InlineCode>"--force"</InlineCode> ", and creates the file "
            <InlineCode>"0600"</InlineCode> "."
        </Callout>

        <P>
            "Use it only for tools that genuinely cannot read anything else, and delete the file "
            "afterwards. If a tool needs a " <InlineCode>".env"</InlineCode>
            " at startup and nothing else, generating it inside a wrapper script that removes it "
            "on exit is better than leaving one in the working tree — but wrapping the tool with "
            <InlineCode>"kivro run"</InlineCode> " is better still."
        </P>
    }
}

fn leak_tests() -> impl IntoView {
    view! {
        <P>
            "Several tests in the workspace exist to catch disclosure rather than incorrect "
            "behaviour. They assert on what is " <em>"absent"</em> " from output:"
        </P>

        {definitions(
            &[
                ("Debug redaction", "formatting a SecretString, or a struct containing one, never contains the value"),
                ("JSON output", "no --json document contains a value"),
                ("list output", "prints names and presence, never values"),
                ("Bundle text", "the serialised bundle does not contain plaintext"),
                ("Namespace injectivity", "two different scopes cannot render to the same service string"),
            ],
        )}

        <P>
            "This is a different category of test from the rest of the suite: a leak does not make "
            "anything fail, which is exactly why it needs a test that fails on its behalf."
        </P>
    }
}

fn reporting() -> impl IntoView {
    view! {
        <P>
            "Report suspected vulnerabilities privately to the maintainers rather than in a public "
            "issue. A public issue for a disclosure bug is itself a disclosure."
        </P>
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
        </ButtonGroup>
    }
}
