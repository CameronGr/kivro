//! Authoring helpers shared by every documentation page.
//!
//! The pages are hand-written Leptos views rather than markdown, so these keep
//! the repetitive shapes — specification tables, command headings, file trees —
//! from being spelled out a hundred times.

use crate::ui::prelude::*;

/// A specification table whose first column is an identifier (a key, a flag, a
/// variable name) and is therefore rendered as inline code.
pub fn spec_table(head: &[&'static str], rows: &[&[&'static str]]) -> impl IntoView {
    table(head, rows, true)
}

/// A table of prose, with no monospaced first column.
pub fn prose_table(head: &[&'static str], rows: &[&[&'static str]]) -> impl IntoView {
    table(head, rows, false)
}

fn table(head: &[&'static str], rows: &[&[&'static str]], code_first: bool) -> impl IntoView {
    let columns: Vec<Column> = head.iter().map(|h| Column::new(*h)).collect();
    let rows: Vec<Vec<&'static str>> = rows.iter().map(|r| r.to_vec()).collect();

    view! {
        <Table columns=columns>
            {rows
                .into_iter()
                .map(|cells| {
                    view! {
                        <Row>
                            {cells
                                .into_iter()
                                .enumerate()
                                .map(|(index, cell)| {
                                    if index == 0 && code_first {
                                        view! {
                                            <Cell index=index header=true>
                                                <InlineCode>{cell}</InlineCode>
                                            </Cell>
                                        }
                                            .into_any()
                                    } else {
                                        view! { <Cell index=index>{cell}</Cell> }.into_any()
                                    }
                                })
                                .collect_view()}
                        </Row>
                    }
                })
                .collect_view()}
        </Table>
    }
}

/// The heading a single CLI command is documented under: the invocation in
/// monospace, followed by a one-line summary.
#[component]
pub fn CommandHeading(
    #[prop(into)] name: String,
    #[prop(into)] usage: String,
    #[prop(into)] summary: String,
) -> impl IntoView {
    let id = format!("cmd-{}", name.replace(' ', "-"));
    view! {
        <div class="flex flex-col gap-2 pt-2">
            <H3 id=id class="font-mono text-accent-200">{usage}</H3>
            <P class="!text-white/62">{summary}</P>
        </div>
    }
}

/// A pair of columns, used for do/don't and before/after comparisons.
#[component]
pub fn SideBySide(#[prop(into)] left: ViewFn, #[prop(into)] right: ViewFn) -> impl IntoView {
    view! {
        <div class="grid gap-4 md:grid-cols-2">
            {left.run()}
            {right.run()}
        </div>
    }
}

/// Small definition row: a term and its meaning, in an inset surface.
pub fn definitions(items: &[(&'static str, &'static str)]) -> impl IntoView {
    let items: Vec<(&'static str, &'static str)> = items.to_vec();
    view! {
        <List>
            {items
                .into_iter()
                .map(|(term, meaning)| {
                    view! {
                        <ListItem>
                            <span class="font-medium text-white/90">{term}</span>
                            <span class="text-white/40">" — "</span>
                            {meaning}
                        </ListItem>
                    }
                })
                .collect_view()}
        </List>
    }
}

/// An in-app link. Renders a real `<a>` so middle-click, "copy link" and
/// crawlers all behave, but routes through the client-side router on a plain
/// left click instead of reloading the whole wasm bundle.
#[component]
pub fn DocLink(#[prop(into)] to: String, children: Children) -> impl IntoView {
    let navigate = leptos_router::hooks::use_navigate();
    let target = to.clone();
    view! {
        <a
            href=to
            class="text-accent-200 underline-offset-4 transition duration-150 hover:underline"
            on:click=move |ev| {
                if ev.ctrl_key() || ev.meta_key() || ev.shift_key() || ev.alt_key()
                    || ev.button() != 0
                {
                    return;
                }
                ev.prevent_default();
                navigate(&target, Default::default());
            }
        >
            {children()}
        </a>
    }
}
