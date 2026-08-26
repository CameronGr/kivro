//! The 404 view, used both by the router fallback and by an unknown doc slug.

use crate::content;
use crate::nav;
use crate::shell::{go_to, use_go};
use crate::ui::prelude::*;

#[component]
pub fn NotFound() -> impl IntoView {
    let go = use_go();

    view! {
        <div class="mx-auto flex max-w-[900px] flex-col gap-8 px-4 py-20 md:px-8">
            <EmptyState
                title="That page does not exist"
                description="The documentation moved around a fair bit while it was being written. \
                             Everything that exists is listed below."
                icon=icons::SEARCH
            >
                <ButtonGroup>
                    <Button variant=Variant::Soft on_click=go_to(go, nav::HOME) icon=icons::ARROW_LEFT>
                        "Home"
                    </Button>
                    <Button
                        variant=Variant::Glass
                        on_click=go_to(go, nav::DOCS)
                        icon=icons::BOOK_OPEN
                    >
                        "Documentation"
                    </Button>
                </ButtonGroup>
            </EmptyState>

            <div class="grid gap-3 sm:grid-cols-2">
                {content::ENTRIES
                    .iter()
                    .map(|entry| {
                        let click = go_to(go, nav::doc_path(entry.slug));
                        view! {
                            <button
                                type="button"
                                class="flex items-start gap-2.5 rounded-2xl border border-white/10 bg-black/20 px-4 py-3 text-left transition duration-150 hover:border-accent-400/25 hover:bg-accent-500/10"
                                on:click=move |_| click.run(())
                            >
                                <Icon
                                    icon=entry.icon
                                    class="mt-0.5 h-4 w-4 shrink-0 text-accent-400/70"
                                />
                                <span class="min-w-0">
                                    <span class="block text-sm font-medium text-white">
                                        {entry.title}
                                    </span>
                                    <span class="block text-xs leading-5 text-white/42">
                                        {entry.blurb}
                                    </span>
                                </span>
                            </button>
                        }
                    })
                    .collect_view()}
            </div>
        </div>
    }
}
