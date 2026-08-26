use leptos::prelude::*;

use crate::cn;
use crate::ui::icons::{
    CIRCLE_CHECK, INFO, Icon, IconData, LOADER, SHIELD_ALERT, SPARKLES, TRIANGLE_ALERT, X,
};
use crate::ui::style::{GLASS, SUNKEN, TRANSITION};
use crate::ui::theme::{Size, Tone};

pub const fn tone_icon(tone: Tone) -> IconData {
    match tone {
        Tone::Accent => CIRCLE_CHECK,
        Tone::Neutral | Tone::Info => INFO,
        Tone::Warning => TRIANGLE_ALERT,
        Tone::Danger => SHIELD_ALERT,
        Tone::Dev => SPARKLES,
    }
}

#[component]
pub fn Callout(
    #[prop(optional)] tone: Tone,
    #[prop(into, optional)] title: Option<String>,
    #[prop(optional)] icon: Option<IconData>,
    #[prop(into, optional)] class: String,
    children: Children,
) -> impl IntoView {
    let heading = title.unwrap_or_else(|| {
        match tone {
            Tone::Accent => "note",
            Tone::Neutral => "aside",
            Tone::Info => "info",
            Tone::Warning => "caution",
            Tone::Danger => "warning",
            Tone::Dev => "preview",
        }
        .to_string()
    });
    let glyph = icon.unwrap_or(tone_icon(tone));

    view! {
        <div class=cn!("rounded-xl border p-5 backdrop-blur-xl", tone.soft(), class)>
            <div class="mb-2 flex items-center gap-2">
                <Icon icon=glyph class="h-4 w-4" />
                <div class="text-sm font-semibold uppercase tracking-[0.16em]">{heading}</div>
            </div>
            <div class="text-sm leading-7 opacity-95">{children()}</div>
        </div>
    }
}

#[component]
pub fn Badge(
    #[prop(optional)] tone: Tone,
    #[prop(optional)] size: Size,
    #[prop(optional)] icon: Option<IconData>,
    #[prop(optional)] squared: bool,
    #[prop(into, optional)] class: String,
    children: Children,
) -> impl IntoView {
    let pad = match size {
        Size::Xs => "px-2 py-0.5 text-[10px] gap-1",
        Size::Sm => "px-2.5 py-1 text-[11px] gap-1.5",
        Size::Md => "px-3 py-1 text-xs gap-1.5",
        Size::Lg => "px-3.5 py-1.5 text-sm gap-2",
    };
    view! {
        <span class=cn!(
            "inline-flex items-center border font-medium uppercase tracking-[0.14em] whitespace-nowrap",
            if squared { "rounded-lg" } else { "rounded-full" },
            pad,
            tone.soft(),
            class,
        )>
            {icon.map(|i| view! { <Icon icon=i class=size.icon() /> })}
            {children()}
        </span>
    }
}

#[component]
pub fn Alert(
    #[prop(optional)] tone: Tone,
    #[prop(into)] title: String,
    #[prop(into, optional)] description: Option<String>,
    #[prop(optional)] icon: Option<IconData>,
    #[prop(into, optional)] on_dismiss: Option<Callback<()>>,
    #[prop(into, optional)] class: String,
) -> impl IntoView {
    let glyph = icon.unwrap_or(tone_icon(tone));
    view! {
        <div
            role="status"
            class=cn!("flex items-start gap-3 rounded-2xl border p-4", tone.soft(), class)
        >
            <Icon icon=glyph class="mt-0.5 h-4 w-4 shrink-0" />
            <div class="min-w-0 flex-1">
                <div class="text-sm font-semibold">{title}</div>
                {description
                    .map(|d| {
                        view! { <div class="mt-1 text-sm leading-6 opacity-80">{d}</div> }
                    })}
            </div>
            {on_dismiss
                .map(|cb| {
                    view! {
                        <button
                            type="button"
                            aria-label="Dismiss"
                            class=cn!(
                                "-m-1 shrink-0 rounded-lg p-1 opacity-60 hover:opacity-100",
                                TRANSITION,
                            )
                            on:click=move |_| cb.run(())
                        >
                            <Icon icon=X class="h-4 w-4" />
                        </button>
                    }
                })}
        </div>
    }
}

#[component]
pub fn Spinner(
    #[prop(optional)] size: Size,
    #[prop(optional)] tone: Tone,
    #[prop(into, optional)] label: Option<String>,
    #[prop(into, optional)] class: String,
) -> impl IntoView {
    let name = label.unwrap_or_else(|| "Loading".to_string());
    view! {
        <Icon
            icon=LOADER
            class=cn!("animate-spin", size.icon(), tone.icon(), class)
            label=name
        />
    }
}

#[component]
pub fn Progress(
    #[prop(into)] value: Signal<f64>,
    #[prop(optional)] tone: Tone,
    #[prop(optional)] show_value: bool,
    #[prop(into, optional)] label: Option<String>,
    #[prop(into, optional)] class: String,
) -> impl IntoView {
    let pct = Signal::derive(move || (value.get().clamp(0.0, 1.0) * 100.0).round());
    let fill = match tone {
        Tone::Accent => "bg-accent-400",
        Tone::Neutral => "bg-white/60",
        Tone::Info => "bg-sky-400",
        Tone::Warning => "bg-amber-400",
        Tone::Danger => "bg-red-400",
        Tone::Dev => "bg-violet-400",
    };

    view! {
        <div class=cn!("flex items-center gap-3", class)>
            <div
                class=cn!(SUNKEN, "h-1.5 flex-1 overflow-hidden rounded-full")
                role="progressbar"
                aria-valuemin="0"
                aria-valuemax="100"
                aria-valuenow=move || pct.get()
                aria-label=label
            >
                <div
                    class=cn!("h-full rounded-full transition-[width] duration-300", fill)
                    style:width=move || format!("{}%", pct.get())
                ></div>
            </div>
            {show_value
                .then(|| {
                    view! {
                        <span class="w-10 shrink-0 text-right text-xs tabular-nums text-white/55">
                            {move || format!("{}%", pct.get())}
                        </span>
                    }
                })}
        </div>
    }
}

#[component]
pub fn Skeleton(
    #[prop(into, optional)] class: String,
    #[prop(optional)] circle: bool,
) -> impl IntoView {
    view! {
        <div
            aria-hidden="true"
            class=cn!(
                "iui-skeleton",
                if circle { "rounded-full" } else { "rounded-xl" },
                class.is_empty().then_some("h-4 w-full"),
                class,
            )
        ></div>
    }
}

#[component]
pub fn Avatar(
    #[prop(into)] name: String,
    #[prop(into, optional)] src: Option<String>,
    #[prop(optional)] size: Size,
    #[prop(into, optional)] class: String,
) -> impl IntoView {
    let initials: String = name
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase();
    let text = match size {
        Size::Xs => "text-[10px]",
        Size::Sm => "text-xs",
        Size::Md => "text-sm",
        Size::Lg => "text-base",
    };
    let cls = cn!(
        "inline-flex shrink-0 items-center justify-center overflow-hidden rounded-full border",
        "border-accent-400/25 bg-accent-500/10 font-semibold text-accent-200",
        size.square(),
        text,
        class,
    );

    match src {
        Some(src) => view! {
            <span class=cls>
                <img src=src alt=name class="h-full w-full object-cover" />
            </span>
        }
        .into_any(),
        None => view! {
            <span class=cls role="img" aria-label=name.clone() title=name>
                <span aria-hidden="true">{initials}</span>
            </span>
        }
        .into_any(),
    }
}

#[component]
pub fn StatusDot(
    #[prop(optional)] tone: Tone,
    #[prop(optional)] pulse: bool,
    #[prop(into, optional)] label: Option<String>,
    #[prop(into, optional)] class: String,
) -> impl IntoView {
    let core = match tone {
        Tone::Accent => "bg-accent-400",
        Tone::Neutral => "bg-white/50",
        Tone::Info => "bg-sky-400",
        Tone::Warning => "bg-amber-400",
        Tone::Danger => "bg-red-400",
        Tone::Dev => "bg-violet-400",
    };
    view! {
        <span class=cn!("inline-flex items-center gap-2", class)>
            <span class="relative flex h-2 w-2">
                {pulse
                    .then(|| {
                        view! {
                            <span class=cn!(
                                "absolute inline-flex h-full w-full animate-ping rounded-full opacity-60",
                                core,
                            )></span>
                        }
                    })}
                <span class=cn!("relative inline-flex h-2 w-2 rounded-full", core)></span>
            </span>
            {label
                .map(|l| {
                    view! { <span class="text-xs text-white/60">{l}</span> }
                })}
        </span>
    }
}

#[component]
pub fn Well(#[prop(into, optional)] class: String, children: Children) -> impl IntoView {
    view! {
        <div class=cn!(GLASS, "divide-y divide-white/[0.06] overflow-hidden rounded-2xl", class)>
            {children()}
        </div>
    }
}
