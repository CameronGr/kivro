//! `/docs/:slug` — one documentation page.

use leptos_meta::{Meta, Title};
use leptos_router::hooks::use_params_map;

use crate::content;
use crate::nav;
use crate::pages::NotFound;
use crate::shell::{PrevNext, Sidebar, go_to, use_go};
use crate::ui::prelude::*;

#[component]
pub fn DocRoute() -> impl IntoView {
    let params = use_params_map();
    let slug = Signal::derive(move || params.read().get("slug").unwrap_or_default());

    move || match content::entry(&slug.get()) {
        Some(entry) => view! { <DocView entry=entry /> }.into_any(),
        None => view! { <NotFound /> }.into_any(),
    }
}

#[component]
fn DocView(entry: &'static content::Entry) -> impl IntoView {
    let go = use_go();
    let doc = (entry.build)();
    let base_path = nav::doc_path(entry.slug);
    let description = doc
        .tagline
        .clone()
        .unwrap_or_else(|| entry.blurb.to_string());

    view! {
        <Title text=format!("{} · kivro", entry.title) />
        <Meta name="description" content=description />

        <div class="px-4 pb-10 pt-6 md:px-8">
            <div class="mx-auto flex max-w-[1500px] gap-8">
                <Sidebar />

                <div class="flex min-w-0 flex-1 flex-col gap-6">
                    <Breadcrumbs items=vec![
                        Crumb::link("kivro", nav::HOME),
                        Crumb::link("Docs", nav::DOCS),
                        Crumb::new(entry.title),
                    ] />

                    <DocPage doc=doc base_path=base_path />

                    <div class="flex flex-col gap-4 pt-2">
                        <Divider label="Keep reading" />
                        <PrevNext slug=entry.slug />
                        <div class="flex flex-wrap items-center justify-between gap-3 pt-2">
                            <Button
                                variant=Variant::Ghost
                                size=Size::Sm
                                icon=icons::BOOK_OPEN
                                on_click=go_to(go, nav::DOCS)
                            >
                                "All documentation"
                            </Button>
                            <Button
                                variant=Variant::Ghost
                                size=Size::Sm
                                href=nav::REPO
                                target="_blank"
                                icon=icons::PENCIL
                                trailing_icon=icons::ARROW_UP_RIGHT
                            >
                                "Improve this page"
                            </Button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
