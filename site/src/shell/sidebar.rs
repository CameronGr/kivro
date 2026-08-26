//! The documentation navigation, and the previous/next pager beneath a page.

use leptos_router::hooks::use_location;

use crate::content;
use crate::nav;
use crate::shell::{go_to, slug_of, use_go};
use crate::ui::prelude::*;
use crate::ui::style::{GLASS, TRANSITION};

/// The grouped page list. Used both in the sidebar and inside the mobile drawer.
#[component]
pub fn DocNav(#[prop(into, optional)] on_pick: Option<Callback<()>>) -> impl IntoView {
    let go = use_go();
    let path = use_location().pathname;
    let active = Signal::derive(move || slug_of(&path.get()).unwrap_or_default());

    view! {
        <nav class="flex flex-col gap-6">
            {content::groups()
                .into_iter()
                .map(|(label, entries)| {
                    let items = Signal::derive(move || {
                        entries
                            .iter()
                            .map(|e| NavItem::new(e.slug, e.title).with_icon(e.icon))
                            .collect::<Vec<_>>()
                    });
                    let select = Callback::new(move |slug: String| {
                        go.run(nav::doc_path(&slug));
                        if let Some(cb) = on_pick {
                            cb.run(());
                        }
                    });
                    view! {
                        <div class="flex flex-col gap-2">
                            <Eyebrow class="px-3">{label}</Eyebrow>
                            <NavList items=items active=active on_select=select />
                        </div>
                    }
                })
                .collect_view()}
        </nav>
    }
}

/// The sticky sidebar column shown from `lg` upwards.
#[component]
pub fn Sidebar() -> impl IntoView {
    view! {
        <aside class="hidden w-[248px] shrink-0 lg:block">
            <div class="sticky top-[5.5rem] max-h-[calc(100vh-7rem)] overflow-y-auto pb-8 pr-2">
                <DocNav />
            </div>
        </aside>
    }
}

/// Links to the pages either side of the current one.
#[component]
pub fn PrevNext(#[prop(into)] slug: String) -> impl IntoView {
    let go = use_go();
    let (previous, next) = content::neighbours(&slug);

    view! {
        <div class="grid gap-3 sm:grid-cols-2">
            {previous
                .map(|entry| {
                    view! { <Pager entry=entry go=go trailing=false /> }
                })}
            {next
                .map(|entry| {
                    view! { <Pager entry=entry go=go trailing=true /> }
                })}
        </div>
    }
}

#[component]
fn Pager(entry: &'static content::Entry, go: Callback<String>, trailing: bool) -> impl IntoView {
    let click = go_to(go, nav::doc_path(entry.slug));
    view! {
        <button
            type="button"
            class=cn!(
                GLASS,
                "group flex w-full flex-col gap-1 rounded-2xl p-4 text-left",
                TRANSITION,
                "hover:border-accent-400/30 hover:bg-accent-500/[0.06]",
                if trailing { "sm:col-start-2 sm:text-right" } else { "" },
            )
            on:click=move |_| click.run(())
        >
            <span class=cn!(
                "flex items-center gap-2 text-[11px] uppercase tracking-[0.16em] text-white/35",
                if trailing { "sm:justify-end" } else { "" },
            )>
                {(!trailing)
                    .then(|| {
                        view! { <Icon icon=icons::ARROW_LEFT class="h-3 w-3" /> }
                    })}
                {if trailing { "Next" } else { "Previous" }}
                {trailing
                    .then(|| {
                        view! { <Icon icon=icons::ARROW_RIGHT class="h-3 w-3" /> }
                    })}
            </span>
            <span class="text-sm font-medium text-white">{entry.title}</span>
            <span class="text-xs leading-5 text-white/45">{entry.blurb}</span>
        </button>
    }
}
