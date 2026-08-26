//! `/docs/manifest` — the `.kivro.toml` specification.

use crate::content::kit::*;
use crate::nav;
use crate::ui::prelude::*;

pub fn doc() -> Doc {
    Doc::new("manifest", "Manifest format", "Reference", "Reference")
        .tagline(
            "`.kivro.toml` declares what a project needs. It is committed to source control, it \
             is read by every command, and it must never contain a secret value.",
        )
        .tags(["format = 1", "TOML", "forward compatible"])
        .section(
            DocSection::new("example", "A complete example", example)
                .numbered("1.0")
                .summary("Every section, annotated, in one file."),
        )
        .section(
            DocSection::new("meta", "[meta]", meta)
                .numbered("1.1")
                .summary("Format version and minimum CLI version — the compatibility gates."),
        )
        .section(
            DocSection::new("project", "[project]", project)
                .numbered("1.2")
                .summary("The name, which is also part of every credential's address."),
        )
        .section(
            DocSection::new("environment", "[environment]", environment)
                .numbered("1.3")
                .summary("Default, the closed list, and strictness."),
        )
        .section(
            DocSection::new("variables", "[variables]", variables)
                .numbered("1.4")
                .summary("Declarations that apply to every environment."),
        )
        .section(
            DocSection::new("per-environment", "[environments.<name>]", per_environment)
                .numbered("1.5")
                .summary("Layering, and the case rule that keeps settings from shadowing variables."),
        )
        .section(
            DocSection::new("sync-section", "[sync]", sync_section)
                .numbered("1.6")
                .summary("Where `kivro sync` looks for values it is missing."),
        )
        .section(
            DocSection::new("compatibility", "Compatibility", compatibility)
                .numbered("2.0")
                .summary("Three mechanisms for evolving the format, and what is guaranteed."),
        )
        .section(
            DocSection::new("never", "What must never appear", never)
                .numbered("2.1")
                .summary("One rule, and why no tool can enforce it for you."),
        )
}

fn example() -> impl IntoView {
    view! {
        <CodeBlock
            language="toml"
            title=".kivro.toml"
            line_numbers=true
            code=r#"
                [meta]
                format = 1
                min_cli_version = "0.1.0"

                [project]
                name = "infinity-launcher"

                [environment]
                default = "dev"
                list    = ["dev", "staging", "production"]
                strict  = true

                # Applies to every environment.
                [variables]
                DATABASE_URL        = { required = true, description = "Primary Postgres DSN" }
                AUTH0_CLIENT_ID     = { required = true }
                AUTH0_CLIENT_SECRET = { required = true }
                SENTRY_DSN          = { required = false }
                LEGACY_API_TOKEN    = { required = false, deprecated = true }

                # Layered over [variables]. A base declaration stays in effect
                # unless this section overrides that exact name.
                [environments.production]
                SENTRY_DSN    = { required = true }
                S3_ACCESS_KEY = { required = true }
                S3_SECRET_KEY = { required = true }

                [sync]
                kind = "file"
                path = "team-secrets"
            "#
        />

        <Callout tone=Tone::Info title="an annotated copy ships with the repository">
            <InlineCode>"examples/kivro.example.toml"</InlineCode>
            " is the same file with a comment on every key. Copy it into a project as a starting "
            "point when " <InlineCode>"kivro init"</InlineCode> " is too bare."
        </Callout>
    }
}

fn meta() -> impl IntoView {
    view! {
        {spec_table(
            &["Key", "Type", "Default", "Meaning"],
            &[
                &["format", "integer", "1", "Manifest format version. A value above what the CLI supports is a hard error."],
                &["min_cli_version", "string", "—", "Minimum CLI version. Older CLIs refuse to run rather than misread the file."],
            ],
        )}

        <P>
            "These are the two gates that let the format change without breaking anyone. "
            <InlineCode>"format"</InlineCode>
            " is for changes that would make an old CLI misread the file; "
            <InlineCode>"min_cli_version"</InlineCode>
            " is for additions that parse fine on an old CLI but mean something it cannot act on."
        </P>
    }
}

fn project() -> impl IntoView {
    view! {
        {spec_table(
            &["Key", "Type", "Required", "Meaning"],
            &[
                &["name", "string", "yes", "Project identity, and the second level of the storage namespace."],
            ],
        )}

        <P>
            "Must match " <InlineCode>"[A-Za-z0-9][A-Za-z0-9._-]*"</InlineCode>
            " and be at most 64 characters."
        </P>

        <Callout tone=Tone::Danger title="renaming orphans your secrets">
            "The name is part of every credential's address. Changing it makes existing stored "
            "values unreachable through kivro — they are still in the credential store, under the "
            "old service string. If you must rename, "
            <InlineCode>"kivro share"</InlineCode> " first, rename, then "
            <InlineCode>"kivro accept"</InlineCode> "."
        </Callout>
    }
}

fn environment() -> impl IntoView {
    view! {
        {spec_table(
            &["Key", "Type", "Default", "Meaning"],
            &[
                &["default", "string", "—", "Environment used when nothing else selects one."],
                &["list", "array of strings", "—", "Closed set of valid environments. When present, default and every [environments.*] section must be a member."],
                &["strict", "boolean", "true", "Reject environments the manifest does not declare."],
            ],
        )}

        <P>
            "Environment names match " <InlineCode>"[A-Za-z0-9][A-Za-z0-9._-]*"</InlineCode>
            " and are at most 32 characters. Setting " <InlineCode>"strict = false"</InlineCode>
            " allows ad-hoc environments — a scratch scope per developer, say — at the cost of "
            "turning a typo into a silently empty secret set."
        </P>
    }
}

fn variables() -> impl IntoView {
    view! {
        <CodeBlock
            language="toml"
            code=r#"
                [variables]
                DATABASE_URL = { required = true, description = "Primary Postgres DSN" }
                FEATURE_FLAG = true          # shorthand for { required = true }
                LEGACY_TOKEN = false         # shorthand for { required = false }
            "#
        />

        {spec_table(
            &["Key", "Type", "Default", "Meaning"],
            &[
                &["required", "boolean", "true", "Whether status and run fail without a stored value."],
                &["description", "string", "—", "Shown in diagnostics. Write it for whoever joins next."],
                &["example", "string", "—", "A NON-secret example value, for documentation only."],
                &["deprecated", "boolean", "false", "doctor warns while a value is still stored."],
            ],
        )}

        <P>
            "Variable names must match " <InlineCode>"[A-Z_][A-Z0-9_]*"</InlineCode>
            " and be at most 128 characters. Lowercase names are not accepted, which is what makes "
            "the case rule in the next section safe."
        </P>
    }
}

fn per_environment() -> impl IntoView {
    view! {
        <P>
            "Per-environment declarations layer " <em>"over"</em> " "
            <InlineCode>"[variables]"</InlineCode>
            ": a base declaration stays in effect unless the environment overrides that exact name."
        </P>

        <CodeBlock
            language="toml"
            title="two equivalent forms"
            code=r#"
                # Variables directly in the section:
                [environments.production]
                SENTRY_DSN = { required = true }

                # Or, when you also want environment settings:
                [environments.production]
                description = "customer-facing"
                [environments.production.variables]
                SENTRY_DSN = { required = true }
            "#
        />

        <Callout tone=Tone::Accent title="uppercase is a variable, lowercase is a setting">
            "Keys inside " <InlineCode>"[environments.<name>]"</InlineCode>
            " are disambiguated by case. Since variable names are validated as uppercase, the two "
            "sets can never overlap, and new settings keys can be added in future versions without "
            "shadowing anybody's variable. Reserved settings keys today: "
            <InlineCode>"description"</InlineCode> ", " <InlineCode>"inherit"</InlineCode> ", "
            <InlineCode>"variables"</InlineCode> "."
        </Callout>

        <H3>"Worked example"</H3>
        <CodeBlock
            language="toml"
            code=r#"
                [variables]
                DATABASE_URL = { required = true }
                SENTRY_DSN   = { required = false }

                [environments.production]
                SENTRY_DSN    = { required = true }   # overrides the base declaration
                S3_ACCESS_KEY = { required = true }   # new in this environment only
            "#
        />
        {prose_table(
            &["Environment", "DATABASE_URL", "SENTRY_DSN", "S3_ACCESS_KEY"],
            &[
                &["dev", "required", "optional", "not declared"],
                &["production", "required", "required", "required"],
            ],
        )}
    }
}

fn sync_section() -> impl IntoView {
    view! {
        {spec_table(
            &["Key", "Type", "Meaning"],
            &[
                &["kind", "string", "Backend discriminator. file is the only one in 0.1."],
                &["path", "string", "For kind = \"file\": a directory of bundles, relative to the project root."],
            ],
        )}

        <P>
            "Non-secret settings only — this section says " <em>"where to look"</em>
            ", never " <em>"what the values are"</em>
            ". Unknown backends are an error naming what this build supports."
        </P>
    }
}

fn compatibility() -> impl IntoView {
    view! {
        <P>"The format is designed to evolve in three ways, in increasing order of severity:"</P>

        <List ordered=true>
            <ListItem>
                <strong class="text-white/85">"Additive keys."</strong>
                " Unknown keys and unknown sections are ignored, and collected for "
                <InlineCode>"kivro doctor"</InlineCode>
                " to report as advisory warnings. An older CLI keeps working with a newer manifest "
                "whenever the addition is genuinely optional."
            </ListItem>
            <ListItem>
                <strong class="text-white/85">"min_cli_version."</strong>
                " For additions that are syntactically ignorable but semantically required. Older "
                "CLIs stop with a clear upgrade message rather than running with a partial "
                "understanding of the file."
            </ListItem>
            <ListItem>
                <strong class="text-white/85">"format."</strong>
                " For changes that would make an old CLI misread the file. Anything above the "
                "supported version is refused outright."
            </ListItem>
        </List>

        <H3>"Guarantees for future versions"</H3>
        {definitions(
            &[
                ("format = 1 manifests will keep parsing", "the compatibility promise of 0.1"),
                (
                    "Variable names stay uppercase-only",
                    "so the case-based disambiguation in [environments.<name>] remains stable",
                ),
                (
                    "The namespace derivation will not change without a format bump",
                    "changing app:project:environment would orphan every stored secret",
                ),
            ],
        )}
    }
}

fn never() -> impl IntoView {
    view! {
        <Callout tone=Tone::Danger title="no values, in any form">
            "Including temporary ones, examples that happen to be real, and base64 of the above. "
            <InlineCode>"kivro doctor"</InlineCode>
            " cannot detect this for you — a string in a TOML file looks like every other string. "
            "Review the manifest in code review like any other committed file."
        </Callout>

        <P>
            "The " <InlineCode>"example"</InlineCode>
            " key is the one that invites trouble: it exists so a description can show the "
            <em>"shape"</em> " of a value (" <InlineCode>"postgres://localhost/launcher"</InlineCode>
            "), not a working one. If you would not paste it into a pull request, it does not "
            "belong there."
        </P>

        <P>
            "A hostile edit to the manifest cannot leak a value, but it can add a declaration, "
            "which makes " <InlineCode>"run"</InlineCode>
            " inject something the project did not previously ask for. That is covered in the "
            <DocLink to=nav::doc_path("security")>"threat model"</DocLink> "."
        </P>
    }
}
