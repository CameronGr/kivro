//! `/docs/concepts` — the vocabulary the rest of the documentation assumes.

use crate::content::kit::*;
use crate::nav;
use crate::ui::prelude::*;

pub fn doc() -> Doc {
    Doc::new("concepts", "Core concepts", "Guide", "Guides")
        .tagline(
            "Six ideas carry the whole tool: the manifest, the store, the scope, the environment, \
             the declaration, and the deliberate refusal to fall back to anything weaker.",
        )
        .tags(["scopes", "resolution order", "no silent fallback"])
        .section(
            DocSection::new("two-halves", "The two halves", two_halves)
                .numbered("1.0")
                .summary("Declarations are committed; values are not, and never meet."),
        )
        .section(
            DocSection::new("names", "Names and validation", names)
                .numbered("1.1")
                .summary("Three identifier types, each validated at construction."),
        )
        .section(
            DocSection::new("scopes", "Scopes and the storage namespace", scopes)
                .numbered("1.2")
                .summary("How a project, an environment and a name become one credential."),
        )
        .section(
            DocSection::new("enumeration", "Enumeration and the index", enumeration)
                .numbered("1.3")
                .summary("Why listing secrets needs a cache, and why the cache cannot lie to you."),
        )
        .section(
            DocSection::new("resolution", "Environment resolution", resolution)
                .numbered("1.4")
                .summary("Five steps, in a fixed order, with the manifest above local preference."),
        )
        .section(
            DocSection::new("declarations", "Declarations and presence", declarations)
                .numbered("1.5")
                .summary("Required, optional, undeclared, deprecated — and what each one changes."),
        )
        .section(
            DocSection::new("failure", "Failing loudly", failure)
                .numbered("1.6")
                .summary("Every degraded path is an error, not a quieter success."),
        )
}

fn two_halves() -> impl IntoView {
    view! {
        <Lead>
            "The manifest answers " <em>"what does this project need"</em>
            ". The credential store answers " <em>"what are the values on this machine"</em>
            ". Nothing in the design lets an answer to the first question contain an answer to the "
            "second."
        </Lead>

        {prose_table(
            &["", "Manifest", "Credential store"],
            &[
                &["Lives in", ".kivro.toml in the repository", "Windows / macOS / Linux keyring"],
                &["Committed", "Yes — that is the point", "Never; it is not a file you own"],
                &["Holds", "Names, requiredness, descriptions, environments", "Values, one per secret"],
                &["Shared by", "git, like any other source file", "kivro share, as an encrypted bundle"],
                &["Per developer", "Identical for everyone", "Different on every machine"],
            ],
        )}

        <Callout tone=Tone::Info title="a third, smaller half">
            "Global configuration — " <InlineCode>"~/.config/kivro/config.toml"</InlineCode>
            " — holds per-machine preferences: colour, a fallback environment, and the storage "
            "namespace. It is non-secret by construction, and the manifest outranks it wherever "
            "they overlap."
        </Callout>
    }
}

fn names() -> impl IntoView {
    view! {
        <P>
            "Three identifier types are validated when they are constructed, not when they are "
            "used. An invalid name cannot reach the storage layer at all."
        </P>

        {spec_table(
            &["Type", "Grammar", "Max length", "Example"],
            &[
                &["ProjectName", "[A-Za-z0-9][A-Za-z0-9._-]*", "64", "infinity-launcher"],
                &["EnvironmentName", "[A-Za-z0-9][A-Za-z0-9._-]*", "32", "production"],
                &["SecretName", "[A-Z_][A-Z0-9_]*", "128", "AUTH0_CLIENT_SECRET"],
            ],
        )}

        <P>
            "Two consequences fall out of this, and both are load-bearing. None of the three can "
            "contain a colon, which is what makes the storage namespace unambiguous. And secret "
            "names are uppercase-only, which is what lets "
            <InlineCode>"[environments.production]"</InlineCode>
            " hold variable declarations and settings in the same table without either shadowing "
            "the other."
        </P>

        <CodeBlock
            language="rust"
            title="validation happens once, at the edge"
            code=r#"
                let name = SecretName::new("AUTH0_CLIENT_SECRET")?;   // ok
                let bad  = SecretName::new("auth0-client-secret");    // Err(InvalidName)
            "#
        />
    }
}

fn scopes() -> impl IntoView {
    view! {
        <P>
            "A " <strong class="text-white/85">"scope"</strong> " is a project plus an "
            "environment. A " <strong class="text-white/85">"store key"</strong>
            " is a scope plus a secret name, and that is the full address of one value."
        </P>

        <CodeBlock
            language="text"
            title="the address"
            code=r#"
                kivro-secrets              application namespace (configurable)
                  └── launcher               project, from [project] name
                        └── dev                environment
                              ├── DATABASE_URL
                              ├── AUTH0_CLIENT_ID
                              └── AUTH0_CLIENT_SECRET
            "#
        />

        <P>"Rendered into the OS credential store as one credential per secret:"</P>

        <CodeBlock
            language="text"
            code=r#"
                service = "kivro-secrets:launcher:dev"
                user    = "DATABASE_URL"
            "#
        />

        <Callout tone=Tone::Accent title="the rendering is injective">
            "Because no name component can contain a colon, two different scopes can never produce "
            "the same service string. " <InlineCode>"DATABASE_URL"</InlineCode>
            " in two projects, or in two environments of one project, are unrelated entries — "
            "there is no shared namespace for them to collide in."
        </Callout>

        <P>
            "The first level, the application namespace, defaults to "
            <InlineCode>"kivro-secrets"</InlineCode> " and can be changed under "
            <InlineCode>"[storage] namespace"</InlineCode>
            " in the global configuration. Changing it points at a different set of credentials; "
            "it does not move the existing ones."
        </P>
    }
}

fn enumeration() -> impl IntoView {
    view! {
        <P>
            "No credential store offers a portable " <em>"list everything under this service"</em>
            " call. Windows, macOS and the Secret Service each disagree about what enumeration "
            "even means. So enumeration is served by a per-scope index credential, stored under "
            "the reserved user name " <InlineCode>"__index"</InlineCode>
            ", holding a JSON array of names only."
        </P>

        {definitions(
            &[
                (
                    "It cannot collide",
                    "secret names are uppercase by validation, so __index is not a name any \
                     variable can have",
                ),
                (
                    "It holds no values",
                    "only names, so a leak of the index discloses no credentials",
                ),
                (
                    "It is a cache, not the truth",
                    "get never consults it, and status probes manifest-declared names directly",
                ),
            ],
        )}

        <Callout tone=Tone::Info title="what a stale index costs you">
            "Exactly one thing: " <InlineCode>"list"</InlineCode>
            " may miss a name that was written by something other than kivro. Every declared "
            "variable is still probed directly, so " <InlineCode>"status"</InlineCode> " and "
            <InlineCode>"run"</InlineCode>
            " stay correct even when enumeration is not. Degrading enumeration is survivable; "
            "degrading correctness is not."
        </Callout>
    }
}

fn resolution() -> impl IntoView {
    view! {
        <P>"Which environment a command operates on is decided in this order:"</P>

        <List ordered=true>
            <ListItem>
                <InlineCode>"--env NAME"</InlineCode> " on the command line"
            </ListItem>
            <ListItem>
                "the " <InlineCode>"KIVRO_ENV"</InlineCode> " environment variable"
            </ListItem>
            <ListItem>
                <InlineCode>"[environment] default"</InlineCode> " in the manifest"
            </ListItem>
            <ListItem>
                <InlineCode>"[defaults] environment"</InlineCode> " in the global configuration"
            </ListItem>
            <ListItem>"otherwise, an error — nothing is guessed"</ListItem>
        </List>

        <Callout tone=Tone::Accent title="why the manifest outranks your config">
            "A per-machine preference must never silently change which environment a shared "
            "project runs against. If the manifest says " <InlineCode>"dev"</InlineCode>
            ", your personal " <InlineCode>"config.toml"</InlineCode>
            " saying " <InlineCode>"production"</InlineCode>
            " does not quietly point your test run at customer data."
        </Callout>

        <P>
            "Under " <InlineCode>"[environment] strict"</InlineCode>
            " — which is on by default — an environment the manifest does not declare is "
            "rejected. " <InlineCode>"--env prod"</InlineCode>
            " against a project that spells it " <InlineCode>"production"</InlineCode>
            " is an error naming the declared environments, not an empty and confusing secret set."
        </P>
    }
}

fn declarations() -> impl IntoView {
    view! {
        {prose_table(
            &["State", "Meaning", "Effect on status", "Effect on run"],
            &[
                &[
                    "Required, present",
                    "Declared with required = true and a value is stored",
                    "✓",
                    "Injected",
                ],
                &[
                    "Required, missing",
                    "Declared but nothing stored",
                    "✗, exit code 3",
                    "Refuses to start the child",
                ],
                &[
                    "Optional",
                    "required = false",
                    "Listed separately, never fails",
                    "Injected when present, skipped when not",
                ],
                &[
                    "Undeclared",
                    "Stored, but the manifest does not mention it",
                    "Reported under \"stored but not declared\"",
                    "Not injected — run only loads declared names",
                ],
                &[
                    "Deprecated",
                    "deprecated = true and a value is still stored",
                    "Normal, plus a doctor warning",
                    "Injected while it remains declared",
                ],
            ],
        )}

        <Callout tone=Tone::Warning title="undeclared values are not injected">
            "This catches people once. " <InlineCode>"kivro run"</InlineCode>
            " loads exactly the names the manifest declares for the resolved environment. A value "
            "you set without declaring it is stored, is reported by "
            <InlineCode>"status"</InlineCode> " and " <InlineCode>"list --all"</InlineCode>
            ", and is included by " <InlineCode>"share --all"</InlineCode>
            " — but it will not appear in your process environment until it is declared."
        </Callout>

        <P>
            "Per-environment declarations layer over the base " <InlineCode>"[variables]"</InlineCode>
            " table: a base declaration stays in effect unless an environment overrides that exact "
            "name. The precise merge rules are on the "
            <DocLink to=nav::doc_path("manifest")>"manifest page"</DocLink> "."
        </P>
    }
}

fn failure() -> impl IntoView {
    view! {
        <Lead>
            "Every place the tool could quietly do something weaker, it stops instead. This is the "
            "single most important design rule in the codebase, and it is worth stating as a list."
        </Lead>

        {definitions(
            &[
                (
                    "An unavailable keyring",
                    "is an error (exit 4), never a fallback to a file",
                ),
                (
                    "An unrecognised KIVRO_STORE value",
                    "is rejected, never guessed at",
                ),
                (
                    "A missing required secret",
                    "fails before the child process is spawned, not during its first request",
                ),
                (
                    "An undeclared environment under strict",
                    "is an error naming the declared ones, not an empty secret set",
                ),
                (
                    "A manifest declaring a newer format",
                    "is refused outright rather than half-understood",
                ),
                (
                    "A bundle whose hint disagrees with its payload",
                    "is a hard error, not a warning",
                ),
                (
                    "A bundle from another project",
                    "is refused by accept, whatever its filename claims",
                ),
                (
                    "Writing a .env",
                    "requires an explicit command, a confirmation, and warns while it does it",
                ),
            ],
        )}

        <P>
            "The cost is that kivro is occasionally annoying. The benefit is that it is never "
            "quietly wrong, which is the only property that matters for something holding your "
            "credentials."
        </P>
    }
}
