use leptos::prelude::*;

use crate::cn;
use crate::ui::style::{EYEBROW, SCROLL_MARGIN, SUNKEN};

#[component]
pub fn H1(
    #[prop(into, optional)] id: Option<String>,
    #[prop(into, optional)] class: String,
    children: Children,
) -> impl IntoView {
    view! {
        <h1
            id=id
            class=cn!(
                "text-3xl font-semibold tracking-tight text-white md:text-4xl",
                SCROLL_MARGIN,
                class,
            )
        >
            {children()}
        </h1>
    }
}

#[component]
pub fn H2(
    #[prop(into, optional)] id: Option<String>,
    #[prop(into, optional)] class: String,
    children: Children,
) -> impl IntoView {
    view! {
        <h2
            id=id
            class=cn!(
                "text-2xl font-semibold tracking-tight text-white md:text-3xl",
                SCROLL_MARGIN,
                class,
            )
        >
            {children()}
        </h2>
    }
}

#[component]
pub fn H3(
    #[prop(into, optional)] id: Option<String>,
    #[prop(into, optional)] class: String,
    children: Children,
) -> impl IntoView {
    view! {
        <h3
            id=id
            class=cn!("text-lg font-semibold tracking-tight text-white", SCROLL_MARGIN, class)
        >
            {children()}
        </h3>
    }
}

#[component]
pub fn Lead(#[prop(into, optional)] class: String, children: Children) -> impl IntoView {
    view! { <p class=cn!("text-base leading-7 text-white/78", class)>{children()}</p> }
}

#[component]
pub fn P(#[prop(into, optional)] class: String, children: Children) -> impl IntoView {
    view! { <p class=cn!("text-[15px] leading-7 text-white/70", class)>{children()}</p> }
}

#[component]
pub fn Eyebrow(#[prop(into, optional)] class: String, children: Children) -> impl IntoView {
    view! { <div class=cn!(EYEBROW, class)>{children()}</div> }
}

#[component]
pub fn Muted(#[prop(into, optional)] class: String, children: Children) -> impl IntoView {
    view! { <span class=cn!("text-xs leading-6 text-white/45", class)>{children()}</span> }
}

#[component]
pub fn InlineCode(#[prop(into, optional)] class: String, children: Children) -> impl IntoView {
    view! {
        <code class=cn!(
            "rounded-md border border-white/10 bg-black/30 px-1.5 py-0.5 text-[0.92em] text-accent-200",
            class,
        )>
            {children()}
        </code>
    }
}

#[component]
pub fn Kbd(#[prop(into, optional)] class: String, children: Children) -> impl IntoView {
    view! {
        <kbd class=cn!(
            "inline-flex min-w-[1.75rem] items-center justify-center rounded-lg border border-white/15",
            "bg-white/[0.06] px-1.5 py-0.5 font-mono text-[11px] font-medium text-white/75",
            "shadow-[inset_0_-1px_0_rgba(255,255,255,0.12)]",
            class,
        )>
            {children()}
        </kbd>
    }
}

#[component]
pub fn List(
    #[prop(optional)] ordered: bool,
    #[prop(into, optional)] class: String,
    children: Children,
) -> impl IntoView {
    let inner = view! { <>{children()}</> };
    let cls = cn!("grid gap-2", class);
    if ordered {
        view! { <ol class=cls>{inner}</ol> }.into_any()
    } else {
        view! { <ul class=cls>{inner}</ul> }.into_any()
    }
}

#[component]
pub fn ListItem(#[prop(into, optional)] class: String, children: Children) -> impl IntoView {
    view! {
        <li class=cn!(SUNKEN, "rounded-2xl px-4 py-3 text-sm leading-6 text-white/72", class)>
            {children()}
        </li>
    }
}

#[component]
pub fn Divider(
    #[prop(into, optional)] label: Option<String>,
    #[prop(into, optional)] class: String,
) -> impl IntoView {
    let rule_l = "h-px flex-1 bg-gradient-to-r from-transparent via-accent-500/15 to-transparent";
    let rule_r = "h-px flex-1 bg-gradient-to-l from-transparent via-accent-500/15 to-transparent";
    view! {
        <div class=cn!("flex items-center gap-4 py-1", class) role="separator">
            <div class=rule_l></div>
            {label
                .map(|l| {
                    view! {
                        <span class="shrink-0 rounded-full border border-accent-500/30 bg-accent-500/10 px-3 py-0.5 text-[10px] uppercase tracking-[0.18em] text-accent-400/70">
                            {l}
                        </span>
                    }
                })}
            <div class=rule_r></div>
        </div>
    }
}
