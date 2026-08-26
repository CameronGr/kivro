//! `/docs` — the documentation index.

use crate::content;
use crate::nav;
use crate::shell::{go_to, use_go};
use crate::ui::prelude::*;
use crate::ui::style::GLASS;

#[component]
pub fn DocsIndex() -> impl IntoView {
    let go = use_go();
    let query = RwSignal::new(String::new());

    let matches = Signal::derive(move || {
        let needle = query.get().trim().to_lowercase();
        content::ENTRIES
            .iter()
            .filter(|e| {
                needle.is_empty()
                    || format!("{} {} {} {}", e.title, e.slug, e.group, e.blurb)
                        .to_lowercase()
                        .contains(&needle)
            })
            .collect::<Vec<_>>()
    });

    view! {
        <div class="mx-auto flex max-w-[1200px] flex-col gap-8 px-4 pb-8 pt-8 md:px-8 md:pt-12">
            <section class=cn!(GLASS, "flex flex-col gap-5 rounded-2xl p-6 md:p-8")>
                <div class="flex flex-wrap items-center gap-2">
                    <Badge tone=Tone::Accent>"Documentation"</Badge>
                    <span class="text-xs uppercase tracking-[0.18em] text-white/42">
                        {format!("kivro {}", nav::VERSION)}
                    </span>
                </div>
                <h1 class="text-3xl font-semibold tracking-tight text-white md:text-4xl">
                    "Everything about kivro"
                </h1>
                <p class="max-w-3xl text-sm leading-7 text-white/62 md:text-base">
                    "Guides for getting a project running, references for the CLI, the manifest "
                    "and the library, and the design documents that say how the storage model and "
                    "the bundle format actually work — including what they do not protect."
                </p>
                <SearchInput
                    value=query
                    on_input=Callback::new(move |v| query.set(v))
                    placeholder="Search the documentation…"
                    class="max-w-md"
                />
            </section>

            <Show
                when=move || !matches.get().is_empty()
                fallback=move || {
                    view! {
                        <EmptyState
                            title="Nothing matches"
                            description="Try a command name, a manifest key, or an error kind."
                            icon=icons::SEARCH
                        />
                    }
                }
            >
                <div class="flex flex-col gap-8">
                    {move || {
                        let visible = matches.get();
                        content::groups()
                            .into_iter()
                            .filter_map(|(label, entries)| {
                                let entries: Vec<_> = entries
                                    .into_iter()
                                    .filter(|e| visible.iter().any(|v| v.slug == e.slug))
                                    .collect();
                                if entries.is_empty() {
                                    return None;
                                }
                                Some(
                                    view! {
                                        <section class="flex flex-col gap-4">
                                            <Divider label=label.to_string() />
                                            <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
                                                {entries
                                                    .into_iter()
                                                    .map(|entry| {
                                                        let doc = (entry.build)();
                                                        let count = doc.sections.len();
                                                        let bullets: Vec<String> = doc
                                                            .tags
                                                            .iter()
                                                            .take(2)
                                                            .cloned()
                                                            .collect();
                                                        view! {
                                                            <Tile
                                                                title=entry.title
                                                                eyebrow=entry.group
                                                                tagline=entry.blurb
                                                                icon=entry.icon
                                                                bullets=bullets
                                                                cta=format!("{count} sections")
                                                                on_click=go_to(go, nav::doc_path(entry.slug))
                                                            />
                                                        }
                                                    })
                                                    .collect_view()}
                                            </div>
                                        </section>
                                    },
                                )
                            })
                            .collect_view()
                    }}
                </div>
            </Show>
        </div>
    }
}
