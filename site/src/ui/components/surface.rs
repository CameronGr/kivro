use std::cell::Cell;

use leptos::html::Div;
use leptos::prelude::*;

use crate::cn;
use crate::ui::icons::{ARROW_RIGHT, Icon, IconData};
use crate::ui::style::{EYEBROW, GLASS, SUNKEN, TRANSITION};
use crate::ui::theme::Tone;

#[component]
pub fn Panel(
    #[prop(into, default = Signal::from(false))] highlighted: Signal<bool>,
    #[prop(into, optional)] id: Option<String>,
    #[prop(into, optional)] class: String,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            id=id
            class=move || {
                cn!(
                    GLASS, "rounded-2xl", TRANSITION, highlighted.get()
                    .then_some("border-accent-400/30 bg-accent-500/[0.06]"), & class,
                )
            }
        >
            {children()}
        </div>
    }
}

#[component]
pub fn Card(
    #[prop(into)] title: String,
    #[prop(into, optional)] eyebrow: Option<String>,
    #[prop(optional)] icon: Option<IconData>,
    #[prop(optional)] tone: Tone,
    #[prop(optional)] actions: Option<ViewFn>,
    #[prop(into, optional)] class: String,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=cn!(GLASS, "rounded-2xl p-5 md:p-6", class)>
            <div class="mb-3 flex items-start justify-between gap-4">
                <div class="flex min-w-0 items-center gap-3">
                    {icon
                        .map(|i| {
                            view! {
                                <div class=cn!(
                                    "flex h-11 w-11 shrink-0 items-center justify-center rounded-2xl border",
                                    tone.border(),
                                    tone.bg(),
                                    tone.icon(),
                                )>
                                    <Icon icon=i class="h-5 w-5" />
                                </div>
                            }
                        })}
                    <div class="min-w-0">
                        {eyebrow
                            .map(|e| {
                                view! {
                                    <div class=cn!(
                                        "mb-0.5 text-[11px] uppercase tracking-[0.16em]",
                                        tone.text(),
                                        "opacity-80",
                                    )>{e}</div>
                                }
                            })}
                        <div class="truncate text-base font-semibold text-white">{title}</div>
                    </div>
                </div>
                {actions.map(|a| a.run())}
            </div>
            <div class="text-sm leading-6 text-white/65">{children()}</div>
        </div>
    }
}

#[component]
pub fn Tile(
    #[prop(into)] title: String,
    #[prop(into)] eyebrow: String,
    #[prop(into)] tagline: String,
    #[prop(optional)] icon: Option<IconData>,
    #[prop(optional, into)] bullets: Vec<String>,
    #[prop(into)] cta: String,
    #[prop(into, optional)] href: Option<String>,
    #[prop(optional)] disabled: bool,
    #[prop(optional)] flag: Option<(String, Tone)>,
    #[prop(into, optional)] on_click: Option<Callback<()>>,
    #[prop(into, optional)] class: String,
) -> impl IntoView {
    let tone = if disabled {
        Tone::Neutral
    } else {
        Tone::Accent
    };
    let cls = cn!(
        GLASS,
        "group relative flex flex-col gap-5 rounded-2xl p-6 md:p-8 text-left",
        TRANSITION,
        if disabled {
            "opacity-80 cursor-not-allowed"
        } else {
            "cursor-pointer hover:border-accent-400/30 hover:bg-accent-500/[0.06]"
        },
        class,
    );

    let body = view! {
        <div class="flex items-start justify-between gap-4">
            <div class="flex items-center gap-3">
                {icon
                    .map(|i| {
                        view! {
                            <div class=cn!(
                                "flex h-12 w-12 shrink-0 items-center justify-center rounded-2xl border",
                                tone.border(),
                                tone.bg(),
                                tone.icon(),
                            )>
                                <Icon icon=i class="h-5 w-5" />
                            </div>
                        }
                    })}
                <div class="min-w-0">
                    <div class="text-[11px] font-semibold uppercase tracking-[0.16em] text-white/45">
                        {eyebrow}
                    </div>
                    <h2 class="text-2xl font-semibold tracking-tight text-white">{title}</h2>
                </div>
            </div>
            {match flag {
                Some((label, flag_tone)) => {
                    view! {
                        <span class=cn!(
                            "shrink-0 rounded-full border px-3 py-1 text-[10px] font-semibold uppercase tracking-[0.18em]",
                            flag_tone.border(),
                            flag_tone.bg(),
                            flag_tone.text(),
                        )>{label}</span>
                    }
                        .into_any()
                }
                None if !disabled => {
                    view! {
                        <Icon
                            icon=ARROW_RIGHT
                            class="mt-1 h-5 w-5 text-accent-400 transition-transform group-hover:translate-x-1"
                        />
                    }
                        .into_any()
                }
                None => ().into_any(),
            }}
        </div>

        <p class="text-sm leading-6 text-white/65">{tagline}</p>

        <ul class="grid gap-2">
            {bullets
                .into_iter()
                .map(|b| {
                    view! {
                        <li class=cn!(
                            SUNKEN,
                            "rounded-2xl px-4 py-2.5 text-[13px] leading-6 text-white/68",
                        )>{b}</li>
                    }
                })
                .collect_view()}
        </ul>

        <div class=cn!(
            "mt-1 inline-flex items-center gap-2 self-start rounded-2xl border px-4 py-2 text-sm",
            TRANSITION,
            if disabled {
                "border-white/10 bg-black/20 text-white/45"
            } else {
                "border-accent-400/25 bg-accent-500/10 text-accent-200 group-hover:bg-accent-500/15"
            },
        )>
            {cta}
            {(!disabled).then(|| view! { <Icon icon=ARROW_RIGHT class="h-4 w-4" /> })}
        </div>
    };

    match href.filter(|_| !disabled) {
        Some(href) => view! {
            <a href=href class=cls>
                {body}
            </a>
        }
        .into_any(),
        None => view! {
            <button
                type="button"
                class=cls
                disabled=disabled
                on:click=move |_| {
                    if let Some(cb) = on_click {
                        cb.run(());
                    }
                }
            >
                {body}
            </button>
        }
        .into_any(),
    }
}

#[component]
pub fn Stat(
    #[prop(into)] label: String,
    #[prop(into)] value: String,
    #[prop(into, optional)] detail: Option<String>,
    #[prop(optional)] icon: Option<IconData>,
    #[prop(optional)] tone: Tone,
    #[prop(into, optional)] class: String,
) -> impl IntoView {
    view! {
        <div class=cn!(GLASS, "rounded-2xl p-5", class)>
            <div class=cn!("mb-2 flex items-center gap-2", EYEBROW)>
                {icon.map(|i| view! { <Icon icon=i class=cn!("h-3.5 w-3.5", tone.icon()) /> })}
                <span>{label}</span>
            </div>
            <div class="text-2xl font-semibold tracking-tight text-white tabular-nums">{value}</div>
            {detail
                .map(|d| {
                    view! { <div class=cn!("mt-1 text-xs", tone.text(), "opacity-80")>{d}</div> }
                })}
        </div>
    }
}

#[component]
pub fn EmptyState(
    #[prop(into)] title: String,
    #[prop(into, optional)] description: Option<String>,
    #[prop(optional)] icon: Option<IconData>,
    #[prop(optional)] children: Option<Children>,
    #[prop(into, optional)] class: String,
) -> impl IntoView {
    view! {
        <div class=cn!(
            SUNKEN,
            "flex flex-col items-center gap-3 rounded-2xl px-6 py-12 text-center",
            class,
        )>
            {icon
                .map(|i| {
                    view! {
                        <div class="flex h-12 w-12 items-center justify-center rounded-2xl border border-white/10 bg-white/[0.04] text-white/35">
                            <Icon icon=i class="h-5 w-5" />
                        </div>
                    }
                })}
            <div class="text-base font-semibold text-white/85">{title}</div>
            {description
                .map(|d| {
                    view! { <p class="max-w-sm text-sm leading-6 text-white/50">{d}</p> }
                })}
            {children.map(|c| view! { <div class="mt-2">{c()}</div> })}
        </div>
    }
}

#[component]
pub fn AnimatedPage(#[prop(into, optional)] class: String, children: Children) -> impl IntoView {
    const DURATION: std::time::Duration = std::time::Duration::from_millis(450);
    const ANIMATION: &str = "iui-animate-page";

    let animating = RwSignal::new(true);
    // Each arming supersedes the previous one, so a navigation part-way through
    // an animation does not have the earlier timer drop the overflow lock.
    let generation = StoredValue::new_local(Cell::new(0u32));
    let arm = move || {
        let current = generation.with_value(|g| {
            let next = g.get().wrapping_add(1);
            g.set(next);
            next
        });
        animating.set(true);
        set_timeout(
            move || {
                if generation.with_value(|g| g.get()) == current {
                    animating.set(false);
                }
            },
            DURATION,
        );
    };
    arm();

    // A client-side navigation usually patches this element in place rather
    // than recreating it, and a CSS animation only runs when the element is
    // inserted — so without this the new page would snap into its final
    // position. Dropping the class, forcing a reflow and re-adding it restarts
    // the animation on every mount, in-place rebuilds included.
    let node: NodeRef<Div> = NodeRef::new();
    Effect::new(move |_| {
        let Some(element) = node.get() else {
            return;
        };
        let classes = element.class_list();
        let _ = classes.remove_1(ANIMATION);
        let _ = element.offset_width();
        let _ = classes.add_1(ANIMATION);
        arm();
    });

    view! {
        <div
            node_ref=node
            class=move || {
                cn!(ANIMATION, animating.get().then_some("overflow-hidden"), & class)
            }
        >
            {children()}
        </div>
    }
}
