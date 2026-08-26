//! `/docs/bundles` — sharing secrets, and the encrypted file format that carries them.

use crate::content::kit::*;
use crate::nav;
use crate::ui::prelude::*;

pub fn doc() -> Doc {
    Doc::new("bundles", "Encrypted bundles", "Reference", "Design & security")
        .tagline(
            "A bundle moves a project's secrets between developers. One text file, encrypted with \
             age, safe to send over any channel — and specified precisely enough that the format \
             can outlive this implementation.",
        )
        .tags(["age", "format = 1", "tamper-evident"])
        .section(
            DocSection::new("workflow", "The workflow", workflow)
                .numbered("1.0")
                .summary("Two commands on two machines, and the rule about the passphrase."),
        )
        .section(
            DocSection::new("envelope", "File format", envelope)
                .numbered("2.0")
                .summary("The outer JSON envelope, and which of its fields you can trust."),
        )
        .section(
            DocSection::new("payload", "The payload", payload)
                .numbered("2.1")
                .summary("What is inside the ciphertext, and why it is the authoritative copy."),
        )
        .section(
            DocSection::new("duplication", "Why metadata is duplicated", duplication)
                .numbered("2.2")
                .summary("age has no AAD input, so the split is stated instead of hidden."),
        )
        .section(
            DocSection::new("crypto", "Cryptography", crypto)
                .numbered("2.3")
                .summary("What provides each property, and what this project implements (nothing)."),
        )
        .section(
            DocSection::new("agility", "Algorithm agility", agility)
                .numbered("2.4")
                .summary("How a scheme change happens without misreading old files."),
        )
        .section(
            DocSection::new("handling", "Handling a bundle", handling)
                .numbered("3.0")
                .summary("It is still a copy of your credentials in a file."),
        )
        .section(
            DocSection::new("sync-source", "Bundles as a sync source", sync_source)
                .numbered("3.1")
                .summary("A directory of bundles is the only sync backend in 0.1 — deliberately."),
        )
}

fn workflow() -> impl IntoView {
    view! {
        <SideBySide
            left=|| {
                view! {
                    <Card title="Sender" eyebrow="kivro share" icon=icons::COPY>
                        <CodeBlock
                            language="bash"
                            dense=true
                            code=r#"
                                kivro share
                                # -> launcher.dev.kivro
                                # prompts for a passphrase, twice

                                kivro share --recipient age1ql3z...
                                # public-key mode, no passphrase
                            "#
                        />
                    </Card>
                }
            }
            right=|| {
                view! {
                    <Card title="Recipient" eyebrow="kivro accept" icon=icons::DOWNLOAD tone=Tone::Info>
                        <CodeBlock
                            language="bash"
                            dense=true
                            code=r#"
                                kivro accept ./launcher.dev.kivro
                                kivro status
                                rm ./launcher.dev.kivro

                                kivro accept ./launcher.dev.kivro \
                                  --identity ~/.config/age/keys.txt
                            "#
                        />
                    </Card>
                }
            }
        />

        <Callout tone=Tone::Danger title="different channels">
            "The file is encrypted, so send it however you like. The passphrase is the whole "
            "secret — send it another way. A file and its passphrase in the same chat thread is a "
            "plaintext file with extra steps."
        </Callout>

        {definitions(
            &[
                ("Default output", "<project>.<environment>.kivro in the project root; -o overrides it"),
                ("What is included", "every stored value for the environment; --all adds undeclared ones too"),
                ("Passphrase length", "a warning below twelve characters — prefer several random words"),
                ("Existing values", "kept on accept unless --force is passed"),
                ("Project check", "accept refuses a bundle whose authenticated project is not yours"),
            ],
        )}
    }
}

fn envelope() -> impl IntoView {
    view! {
        <CodeBlock
            language="json"
            title="launcher.dev.kivro"
            code=r#"
                {
                  "magic": "kivro-bundle",
                  "format": 1,
                  "cipher": "age-v1-scrypt",
                  "hint": {
                    "project": "launcher",
                    "environment": "dev",
                    "created_at": "2026-01-14T09:31:00Z",
                    "created_by": "cameron"
                  },
                  "payload": "-----BEGIN AGE ENCRYPTED FILE-----\n…\n-----END AGE ENCRYPTED FILE-----\n"
                }
            "#
        />

        {prose_table(
            &["Field", "Authenticated", "Meaning"],
            &[
                &["magic", "no", "Fixed string identifying the file type: kivro-bundle."],
                &["format", "no", "Envelope version. Above the supported version, the file is refused."],
                &["cipher", "no", "age-v1-scrypt (passphrase) or age-v1-x25519 (recipients)."],
                &["hint", "no", "Advisory routing metadata. See below."],
                &["payload", "yes", "ASCII-armored age file."],
            ],
        )}

        <P>
            "The envelope is JSON so that tooling can read the routing fields without a key, and "
            "so a human can tell what a stray file is. Everything outside "
            <InlineCode>"payload"</InlineCode> " is unauthenticated, and the format says so rather "
            "than presenting it as trustworthy."
        </P>
    }
}

fn payload() -> impl IntoView {
    view! {
        <CodeBlock
            language="json"
            title="inside the age ciphertext"
            code=r#"
                {
                  "format": 1,
                  "project": "launcher",
                  "environment": "dev",
                  "created_at": "2026-01-14T09:31:00Z",
                  "created_by": "cameron",
                  "secrets": { "DATABASE_URL": "…", "AUTH0_CLIENT_SECRET": "…" }
                }
            "#
        />

        <P>
            "This is the " <strong class="text-white/85">"authoritative"</strong>
            " copy of every field. age authenticates the whole ciphertext — ChaCha20-Poly1305 per "
            "chunk, plus an HMAC-SHA-256 over the header — so the payload is tamper-evident. On "
            "decryption, every hint field is compared against it, and any disagreement is a hard "
            <InlineCode>"bundle_mismatch"</InlineCode> " error."
        </P>

        <Callout tone=Tone::Info title="names are withheld by default">
            "A list of variable names discloses which vendors and services a project depends on, "
            "so the hint omits them unless " <InlineCode>"--hint-names"</InlineCode>
            " is passed. That flag exists for tooling that has to plan before it can decrypt."
        </Callout>
    }
}

fn duplication() -> impl IntoView {
    view! {
        <P>
            "age has no additional-authenticated-data input, so there is no way to bind an outer "
            "plaintext header to the ciphertext. Two designs were available: pretend the header is "
            "trustworthy, or state the split. The format states the split."
        </P>

        {definitions(
            &[
                (
                    "The hint exists to route",
                    "so tooling can say \"this looks like a bundle for launcher/dev\" before a \
                     passphrase is available",
                ),
                (
                    "The payload exists to decide",
                    "every hint field is cross-checked against it on decryption, and a mismatch is \
                     an error, not a warning",
                ),
                (
                    "accept goes one step further",
                    "it compares the payload's project against the local manifest, so a bundle for \
                     another project is refused even when its filename and hint both agree with \
                     each other",
                ),
            ],
        )}
    }
}

fn crypto() -> impl IntoView {
    view! {
        {prose_table(
            &["Concern", "Provided by"],
            &[
                &["Confidentiality", "age: ChaCha20-Poly1305 over the STREAM construction"],
                &["Integrity and authentication", "age: per-chunk AEAD tags plus a header HMAC-SHA-256"],
                &["Password-based KDF", "age scrypt recipient, with the work factor carried in the header"],
                &["Public-key mode", "age X25519 + HKDF-SHA-256"],
                &["Versioning", "format (envelope) and cipher (scheme)"],
            ],
        )}

        <Callout tone=Tone::Accent title="no cryptography is implemented in this project">
            "age was chosen over a bespoke format because it is specified, widely reviewed, and "
            "interoperable — a bundle's payload can be decrypted with the standard "
            <InlineCode>"age"</InlineCode>
            " CLI, which matters for a format that will hold production credentials for years."
        </Callout>

        <P>
            "Passphrases are never used as keys directly: age's scrypt recipient handles "
            "stretching, with the work factor stored in the file. Decryption bounds the accepted "
            "work factor at 2^20, so a hostile bundle cannot turn "
            <InlineCode>"kivro accept"</InlineCode> " into hours of KDF work."
        </P>
    }
}

fn agility() -> impl IntoView {
    view! {
        <P>
            <InlineCode>"cipher"</InlineCode>
            " names the scheme, and an unrecognised value is refused with a message naming what "
            "the build supports. Adding a scheme — a future age version, a post-quantum recipient "
            "type — means adding a " <InlineCode>"cipher"</InlineCode>
            " value; old builds refuse cleanly rather than misreading. "
            <InlineCode>"format"</InlineCode>
            " versions the envelope itself, for changes to the JSON structure."
        </P>

        <Callout tone=Tone::Info title="a worked migration">
            "For a hypothetical " <InlineCode>"age-v2"</InlineCode>
            ": new CLIs keep writing " <InlineCode>"age-v1-*"</InlineCode>
            " until a release date, then switch; " <InlineCode>"kivro accept"</InlineCode>
            " reads both indefinitely. Bundles are short-lived transfer artefacts rather than "
            "storage, which is what keeps this cheap."
        </Callout>
    }
}

fn handling() -> impl IntoView {
    view! {
        <P>
            "Bundles are encrypted, but they are still a copy of your credentials in a file. Four "
            "habits are worth having:"
        </P>

        <List>
            <ListItem>
                "Send the file and the passphrase over " <strong class="text-white/85">"different"</strong>
                " channels."
            </ListItem>
            <ListItem>
                "Prefer several random words to a short passphrase. scrypt buys time, not "
                "miracles, and a bundle in a chat log is available to an attacker forever."
            </ListItem>
            <ListItem>
                "Prefer " <InlineCode>"--recipient"</InlineCode>
                " (age public keys) for anything automated."
            </ListItem>
            <ListItem>
                "Delete the file after accepting it. " <InlineCode>"kivro doctor"</InlineCode>
                " warns about bundles left in a project root, and "
                <InlineCode>"doctor --fix-gitignore"</InlineCode> " adds "
                <InlineCode>"*.kivro"</InlineCode> " to " <InlineCode>".gitignore"</InlineCode> "."
            </ListItem>
        </List>

        <Callout tone=Tone::Warning title="there is no revocation">
            "Once a bundle is out, every value in it is out. A bundle you regret is a rotation "
            "task, not a deletion task — see the "
            <DocLink to=nav::doc_path("security")>"security model"</DocLink>
            " on what is and is not in scope for 0.1."
        </Callout>
    }
}

fn sync_source() -> impl IntoView {
    view! {
        <P>
            "A directory of bundles is the only " <InlineCode>"[sync]"</InlineCode>
            " backend in 0.1. It needs no server and works over any transport a team already has: "
            "a shared drive, a private repository, an object bucket someone mounts."
        </P>

        <CodeBlock
            language="toml"
            code=r#"
                [sync]
                kind = "file"
                path = "team-secrets"
            "#
        />

        <CodeBlock
            language="bash"
            code=r#"
                kivro sync            # report only: present, missing, fetchable, unavailable
                kivro sync --apply    # fetch what the source can supply
            "#
        />

        <P>
            "Future sources — an internal HTTP service, S3-compatible storage, git-backed "
            "storage, another secret manager — implement one trait without changing the bundle "
            "format or anything above that layer. The format is the compatibility surface; the "
            "transport is not."
        </P>
    }
}
