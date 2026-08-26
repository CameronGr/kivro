//! The sticky top bar, and the drawer it opens on small screens.

use leptos_router::hooks::use_location;

use crate::nav;
use crate::shell::{DocNav, go_to, use_go};
use crate::ui::style::{FOCUS_RING, TRANSITION};
use crate::ui::prelude::*;

#[component]
pub fn Header() -> impl IntoView {
    let go = use_go();
    let path = use_location().pathname;
    let menu = RwSignal::new(false);

    let in_docs = Signal::derive(move || path.get().starts_with(nav::DOCS));

    view! {
        <StickyBar top="top-0">
            <IconButton
                icon=icons::MENU
                label="Open navigation"
                variant=Variant::Glass
                size=Size::Sm
                class="lg:hidden"
                on_click=Callback::new(move |()| menu.set(true))
            />

            <button
                type="button"
                class=cn!(
                    "flex shrink-0 items-center gap-2.5 rounded-2xl px-1.5 py-1 text-left",
                    TRANSITION,
                    "hover:opacity-80",
                )
                on:click=move |_| go.run(nav::HOME.to_string())
            >
                <span class="flex h-8 w-8 items-center justify-center rounded-xl border border-accent-400/30 bg-accent-500/10">
                    <Icon icon=icons::SHIELD_ALERT class="h-4 w-4 text-accent-400" />
                </span>
                <span class="leading-tight">
                    <span class="block text-sm font-semibold tracking-tight text-white">
                        "kivro"
                    </span>
                    <span class="hidden text-[10px] uppercase tracking-[0.18em] text-white/35 sm:block">
                        "secret manager"
                    </span>
                </span>
            </button>

            <nav class="ml-2 hidden items-center gap-1 lg:flex">
                <HeaderLink label="Docs" to=nav::DOCS.to_string() active=in_docs />
                <HeaderLink
                    label="Quick start"
                    to=nav::doc_path("quickstart")
                    active=Signal::derive(move || path.get() == nav::doc_path("quickstart"))
                />
                <HeaderLink
                    label="CLI"
                    to=nav::doc_path("cli")
                    active=Signal::derive(move || path.get() == nav::doc_path("cli"))
                />
                <HeaderLink
                    label="Library"
                    to=nav::doc_path("library")
                    active=Signal::derive(move || path.get() == nav::doc_path("library"))
                />
                <HeaderLink
                    label="Security"
                    to=nav::doc_path("security")
                    active=Signal::derive(move || path.get() == nav::doc_path("security"))
                />
            </nav>

            <div class="ml-auto flex shrink-0 items-center gap-2">
                <span class="hidden md:inline">
                    <StatusDot tone=Tone::Accent pulse=true label=format!("v{}", nav::VERSION) />
                </span>
                <Button
                    variant=Variant::Glass
                    size=Size::Sm
                    href=nav::REPO
                    target="_blank"
                    icon=icons::GITHUB
                    trailing_icon=icons::ARROW_UP_RIGHT
                >
                    "GitHub"
                </Button>
            </div>
        </StickyBar>

        <Drawer
            open=menu
            on_close=Callback::new(move |()| menu.set(false))
            side=Side::Left
            title="Documentation"
        >
            <DocNav on_pick=Callback::new(move |()| menu.set(false)) />
        </Drawer>
    }
}

#[component]
fn HeaderLink(
    #[prop(into)] label: String,
    #[prop(into)] to: String,
    #[prop(into)] active: Signal<bool>,
) -> impl IntoView {
    let go = use_go();
    let click = go_to(go, to);
    view! {
        <button
            type="button"
            class=move || {
                cn!(
                    "rounded-2xl px-3 py-1.5 text-sm", TRANSITION, FOCUS_RING, if active.get() {
                    "bg-accent-500/14 text-accent-100" } else {
                    "text-white/55 hover:bg-white/[0.05] hover:text-white" },
                )
            }
            on:click=move |_| click.run(())
        >
            {label}
        </button>
    }
}
