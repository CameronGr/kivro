//! `/docs/install` — getting the binary onto a machine.

use crate::content::kit::*;
use crate::nav;
use crate::ui::prelude::*;

pub fn doc() -> Doc {
    Doc::new("install", "Installation", "Guide", "Getting started")
        .tagline(
            "kivro is a single Rust binary with no runtime dependencies beyond the credential \
             store your operating system already ships. Install it with cargo, then check that \
             the store it will use is actually reachable.",
        )
        .tags(["cargo install", "Rust 1.96+", "Windows · macOS · Linux"])
        .section(
            DocSection::new("requirements", "Requirements", requirements)
                .numbered("1.0")
                .summary("Toolchain version, and what each platform provides as the backend."),
        )
        .section(
            DocSection::new("install-cli", "Install the CLI", install_cli)
                .numbered("1.1")
                .summary("From git, from a checkout, or built by hand."),
        )
        .section(
            DocSection::new("platforms", "Platform notes", platforms)
                .numbered("1.2")
                .summary("Linux needs a running keyring daemon; the others do not."),
        )
        .section(
            DocSection::new("verify", "Verify the installation", verify)
                .numbered("1.3")
                .summary("One command that tells you whether the store works before you rely on it."),
        )
        .section(
            DocSection::new("headless", "Headless machines and CI", headless)
                .numbered("1.4")
                .summary("What to do on a box with no session keyring — and what it costs."),
        )
        .section(
            DocSection::new("library-dep", "Using the library", library_dep)
                .numbered("1.5")
                .summary("Adding the `kivro` crate to a Rust project instead of the binary."),
        )
        .section(
            DocSection::new("upgrading", "Upgrading and removing", upgrading)
                .numbered("1.6")
                .summary("Both are ordinary cargo operations; stored values outlive them."),
        )
}

fn requirements() -> impl IntoView {
    view! {
        <Lead>
            "Rust " {nav::RUST_VERSION}
            " or newer, and a credential store. Everything else is vendored by cargo."
        </Lead>

        {prose_table(
            &["Platform", "Backend", "Extra setup"],
            &[
                &["Windows", "Credential Manager", "None"],
                &["macOS", "Keychain", "None"],
                &[
                    "Linux",
                    "Secret Service (GNOME Keyring, KWallet, KeePassXC…)",
                    "A running keyring daemon, and libdbus-1-dev at build time",
                ],
            ],
        )}

        <Callout tone=Tone::Info title="why a toolchain this recent">
            "The workspace is on the 2024 edition and pins "
            <InlineCode>"rust-version = \"1.96.0\""</InlineCode>
            " in its manifest, so cargo will refuse to build rather than fail somewhere confusing "
            "on an older compiler. " <InlineCode>"rustup update stable"</InlineCode>
            " is usually the whole fix."
        </Callout>
    }
}

fn install_cli() -> impl IntoView {
    view! {
        <H3>"From the repository"</H3>
        <P>"The usual route while the crate is pre-release:"</P>
        <CommandLine command="cargo install --git https://github.com/CameronGr/kivro kivro-cli" />

        <H3>"From a local checkout"</H3>
        <CodeBlock
            language="bash"
            code=r#"
                git clone https://github.com/CameronGr/kivro
                cd kivro
                cargo install --path crates/kivro-cli
            "#
        />

        <H3>"Build without installing"</H3>
        <P>
            "Useful when you want to test a change before it lands. The binary is named "
            <InlineCode>"kivro"</InlineCode> ", not " <InlineCode>"kivro-cli"</InlineCode> "."
        </P>
        <CodeBlock
            language="bash"
            code=r#"
                cargo build --release -p kivro-cli
                ./target/release/kivro --version
            "#
        />

        <Callout tone=Tone::Warning title="PATH">
            <InlineCode>"cargo install"</InlineCode> " writes to "
            <InlineCode>"~/.cargo/bin"</InlineCode> " (or "
            <InlineCode>"%USERPROFILE%\\.cargo\\bin"</InlineCode>
            "). If " <InlineCode>"kivro"</InlineCode>
            " is not found afterwards, that directory is not on your PATH — every rustup install "
            "adds it, but a system package manager install of Rust may not have."
        </Callout>
    }
}

fn platforms() -> impl IntoView {
    view! {
        <H3>"Windows"</H3>
        <P>
            "Nothing to configure. Secrets appear in Credential Manager under "
            <em>"Windows Credentials"</em> ", with a service name of "
            <InlineCode>"kivro-secrets:<project>:<environment>"</InlineCode> "."
        </P>

        <H3>"macOS"</H3>
        <P>
            "Nothing to configure. The login keychain is used. The first access from a newly "
            "built binary prompts for permission, which is the OS doing its job — allow it once "
            "and it stops asking for that binary."
        </P>

        <H3>"Linux"</H3>
        <P>
            "The Secret Service API is a D-Bus interface, so it needs a daemon that implements it "
            "and a session bus to talk over. GNOME Keyring, KWallet and KeePassXC (with Secret "
            "Service integration enabled) all qualify. On a desktop this is already running."
        </P>
        <CodeBlock
            language="bash"
            title="build dependency, Debian/Ubuntu"
            code=r#"
                sudo apt install libdbus-1-dev pkg-config
            "#
        />
        <Callout tone=Tone::Danger title="no daemon, no fallback">
            "An unavailable keyring is an error, never a silent downgrade to something weaker. If "
            "there is no Secret Service, " <InlineCode>"kivro"</InlineCode>
            " exits with code 4 and tells you so, rather than quietly writing your credentials "
            "somewhere less protected."
        </Callout>
    }
}

fn verify() -> impl IntoView {
    view! {
        <P>
            <InlineCode>"kivro doctor"</InlineCode>
            " is the installation test. Run it inside a project once you have a manifest; outside "
            "one it still reports on the store and the CLI."
        </P>
        <CodeBlock
            language="bash"
            code=r#"
                kivro --version
                kivro doctor
            "#
        />
        <P>"It checks, in order:"</P>
        {definitions(
            &[
                ("manifest", "found, parses, project identity, CLI version compatibility"),
                ("unknown keys", "manifest keys this build does not recognise, as advisory warnings"),
                ("credential store", "which backend, and whether it is reachable and OS-protected"),
                ("environment", "which environment resolved, and whether required secrets are present"),
                ("deprecated secrets", "values still stored for variables the manifest marks deprecated"),
                ("git hygiene", "a stray .env, missing .gitignore entries, bundles left in the project root"),
            ],
        )}
        <P>
            "Exit code 7 means at least one check failed. Warnings alone exit 0. The full "
            "breakdown is in the "
            <DocLink to=nav::doc_path("cli")>"CLI reference"</DocLink> "."
        </P>
    }
}

fn headless() -> impl IntoView {
    view! {
        <P>
            "CI runners and containers usually have no D-Bus session, no Keychain and no "
            "Credential Manager. For those, " <InlineCode>"KIVRO_STORE=file"</InlineCode>
            " selects a plaintext JSON store at a path you choose."
        </P>
        <CodeBlock
            language="bash"
            code=r#"
                export KIVRO_STORE=file
                export KIVRO_STORE_FILE="$RUNNER_TEMP/kivro-store.json"

                printf '%s' "$DATABASE_URL" | kivro set DATABASE_URL --stdin
                kivro status
                kivro run -- cargo test
            "#
        />
        <Callout tone=Tone::Danger title="plaintext on disk">
            "The file store provides no confidentiality against anything except other users on "
            "the same machine. It is never a default and never a fallback, the CLI warns on every "
            "command that uses it, and "
            <InlineCode>"doctor"</InlineCode>
            " reports it as a finding. Use it for ephemeral CI storage whose real source of truth "
            "is your CI provider's own secret storage — never for a developer machine."
        </Callout>
        <P>
            "There is also " <InlineCode>"KIVRO_STORE=memory"</InlineCode>
            ", which keeps values for the lifetime of one process. It exists for tests."
        </P>
    }
}

fn library_dep() -> impl IntoView {
    view! {
        <P>
            "The CLI is one consumer of the "<InlineCode>"kivro"</InlineCode>
            " crate, not the only interface. To load secrets from a Rust program without spawning "
            "a subprocess:"
        </P>
        <CodeBlock
            language="toml"
            title="Cargo.toml"
            code=r#"
                [dependencies]
                kivro = { git = "https://github.com/CameronGr/kivro" }
            "#
        />
        <P>
            "The API is covered on the "
            <DocLink to=nav::doc_path("library")>"library page"</DocLink>
            ". You do not need the CLI installed to use it, though "
            <InlineCode>"kivro set"</InlineCode> " remains the way values get into the store."
        </P>
    }
}

fn upgrading() -> impl IntoView {
    view! {
        <CodeBlock
            language="bash"
            code=r#"
                # upgrade in place
                cargo install --git https://github.com/CameronGr/kivro kivro-cli --force

                # remove the binary
                cargo uninstall kivro-cli
            "#
        />
        <Callout tone=Tone::Info title="stored values are not touched">
            "Uninstalling the binary leaves every credential where it is — they belong to the OS "
            "store, not to kivro. To remove values, use " <InlineCode>"kivro remove NAME"</InlineCode>
            " while the binary is still installed, or delete them through the platform's own "
            "credential UI afterwards."
        </Callout>
        <P>
            "Manifests written by 0.1 declare " <InlineCode>"format = 1"</InlineCode>
            ", which future releases will keep reading. A manifest that declares a "
            <em>"newer"</em>
            " format than your CLI understands is refused outright rather than half-parsed; that "
            "is the upgrade prompt working as intended."
        </P>
    }
}
