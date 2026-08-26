use leptos::prelude::*;

use crate::cn;
use crate::ui::icons::{Icon, IconData, LOADER};
use crate::ui::style::{FOCUS_RING, SUNKEN, TRANSITION};
use crate::ui::theme::{Size, Tone, Variant};

fn variant_classes(variant: Variant, tone: Tone) -> String {
    match variant {
        Variant::Soft => cn!(
            tone.border(),
            tone.bg(),
            tone.text(),
            "hover:brightness-125"
        ),
        Variant::Solid => cn!("border border-transparent font-medium", tone.solid()),
        Variant::Glass => cn!(
            SUNKEN,
            "text-white/68",
            "hover:border-accent-400/25 hover:bg-accent-500/10 hover:text-white",
        ),
        Variant::Ghost => cn!(
            "border border-transparent text-white/60",
            "hover:bg-white/[0.06] hover:text-white",
        ),
        Variant::Link => cn!(
            "border-0 p-0 underline-offset-4 hover:underline",
            tone.text(),
        ),
    }
}

#[component]
pub fn Button(
    #[prop(optional)] variant: Variant,
    #[prop(optional)] tone: Tone,
    #[prop(optional)] size: Size,
    #[prop(optional)] icon: Option<IconData>,
    #[prop(optional)] trailing_icon: Option<IconData>,
    #[prop(into, default = Signal::from(false))] loading: Signal<bool>,
    #[prop(into, default = Signal::from(false))] disabled: Signal<bool>,
    #[prop(optional)] full_width: bool,
    #[prop(into, optional)] href: Option<String>,
    #[prop(into, optional)] target: Option<String>,
    #[prop(into, optional)] on_click: Option<Callback<()>>,
    #[prop(into, optional)] class: String,
    children: Children,
) -> impl IntoView {
    let is_link = variant == Variant::Link;
    let cls = cn!(
        "inline-flex items-center justify-center whitespace-nowrap",
        TRANSITION,
        FOCUS_RING,
        tone.ring(),
        (!is_link).then_some(size.radius()),
        (!is_link).then_some(size.control()),
        is_link.then_some("gap-1.5 text-sm"),
        variant_classes(variant, tone),
        full_width.then_some("w-full"),
        "disabled:pointer-events-none disabled:opacity-45 aria-disabled:pointer-events-none aria-disabled:opacity-45",
        class,
    );

    let icon_size = size.icon();
    let blocked = Signal::derive(move || disabled.get() || loading.get());

    let leading = {
        let icon_size = icon_size.to_string();
        move || {
            if loading.get() {
                Some(
                    view! {<Icon icon=LOADER class=cn!(icon_size.clone(), "animate-spin") /> }
                        .into_any(),
                )
            } else {
                icon.map(|i| view! {<Icon icon=i class=icon_size.clone() />}.into_any())
            }
        }
    };

    let trailing = trailing_icon.map(|i| view! {<Icon icon=i class=icon_size />});

    let fire = move |_| {
        if blocked.get_untracked() {
            return;
        }
        if let Some(cb) = on_click {
            cb.run(());
        }
    };

    match href {
        Some(href) => {
            let rel = target
                .as_deref()
                .filter(|t| *t == "_blank")
                .map(|_| "noreferrer noopener");
            view! {
                <a
                    href=href
                    target=target
                    rel=rel
                    class=cls
                    aria-disabled=move || blocked.get().then_some("true")
                    tabindex=move || blocked.get().then_some("-1")
                    on:click=fire
                >
                    {leading()}
                    {children()}
                    {trailing}
                </a>
            }
            .into_any()
        }
        None => view! {
            <button
                type="button"
                class=cls
                disabled=move || blocked.get()
                aria-busy=move || loading.get().then_some("true")
                on:click=fire
            >
                {leading()}
                {children()}
                {trailing}
            </button>
        }
        .into_any(),
    }
}

#[component]
pub fn IconButton(
    icon: IconData,
    #[prop(into)] label: String,
    #[prop(optional)] variant: Variant,
    #[prop(optional)] tone: Tone,
    #[prop(optional)] size: Size,
    #[prop(into, default = Signal::from(false))] disabled: Signal<bool>,
    #[prop(into, optional)] on_click: Option<Callback<()>>,
    #[prop(into, optional)] class: String,
) -> impl IntoView {
    let cls = cn!(
        "inline-flex items-center justify-center",
        size.square(),
        size.radius(),
        TRANSITION,
        FOCUS_RING,
        tone.ring(),
        variant_classes(variant, tone),
        "disabled:pointer-events-none disabled:opacity-45",
        class,
    );
    view! {
        <button
            type="button"
            class=cls
            title=label.clone()
            aria-label=label
            disabled=move || disabled.get()
            on:click=move |_| {
                if let Some(cb) = on_click {
                    cb.run(());
                }
            }
        >
            <Icon icon=icon class=size.icon() />
        </button>
    }
}

#[component]
pub fn ButtonGroup(#[prop(into, optional)] class: String, children: Children) -> impl IntoView {
    view! {
        <div class=cn!("inline-flex items-center gap-2", class) role="group">
            {children()}
        </div>
    }
}
