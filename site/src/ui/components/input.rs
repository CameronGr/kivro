use leptos::prelude::*;
use leptos::web_sys;
use wasm_bindgen::JsCast;

use crate::cn;
use crate::ui::hooks::use_id;
use crate::ui::icons::{Icon, IconData, SEARCH, X};
use crate::ui::style::{FOCUS_RING, SUNKEN, TRANSITION};
use crate::ui::theme::{Size, Tone};

#[component]
pub fn Field(
    #[prop(into)] label: String,
    #[prop(into, optional)] hint: Option<String>,
    #[prop(into, optional)] error: Option<Signal<Option<String>>>,
    #[prop(optional)] required: bool,
    #[prop(into, optional)] control_id: Option<String>,
    #[prop(into, optional)] class: String,
    children: Children,
) -> impl IntoView {
    let error = error.unwrap_or_else(|| Signal::from(None));
    view! {
        <div class=cn!("flex flex-col gap-1.5", class)>
            <label
                for=control_id
                class="flex items-center gap-1 text-xs font-medium uppercase tracking-[0.14em] text-white/50"
            >
                {label}
                {required.then(|| view! { <span class="text-accent-400">"*"</span> })}
            </label>
            {children()}
            {move || match error.get() {
                Some(msg) => {
                    view! { <div class="text-xs leading-5 text-red-300">{msg}</div> }.into_any()
                }
                None => {
                    hint.clone()
                        .map(|h| view! { <div class="text-xs leading-5 text-white/40">{h}</div> })
                        .into_any()
                }
            }}
        </div>
    }
}

fn field_classes(size: Size, invalid: bool, extra: &str) -> String {
    cn!(
        "w-full rounded-2xl text-white placeholder:text-white/30",
        SUNKEN,
        size.field(),
        TRANSITION,
        FOCUS_RING,
        if invalid {
            "border-red-400/40 bg-red-500/[0.06] focus-visible:ring-red-400/40"
        } else {
            "hover:border-white/20 focus-visible:border-accent-400/40 focus-visible:ring-accent-400/40"
        },
        "disabled:cursor-not-allowed disabled:opacity-50",
        extra,
    )
}

fn value_of(ev: &web_sys::Event) -> Option<String> {
    let target = ev.target()?;
    if let Some(input) = target.dyn_ref::<web_sys::HtmlInputElement>() {
        return Some(input.value());
    }
    target
        .dyn_ref::<web_sys::HtmlTextAreaElement>()
        .map(|t| t.value())
}

#[component]
pub fn TextInput(
    #[prop(into)] value: Signal<String>,
    #[prop(into)] on_input: Callback<String>,
    #[prop(into, optional)] placeholder: Option<String>,
    #[prop(into, default = "text".to_string())] input_type: String,
    #[prop(optional)] icon: Option<IconData>,
    #[prop(optional)] size: Size,
    #[prop(into, default = Signal::from(false))] disabled: Signal<bool>,
    #[prop(into, default = Signal::from(false))] invalid: Signal<bool>,
    #[prop(into, optional)] id: Option<String>,
    #[prop(into, optional)] class: String,
) -> impl IntoView {
    let id = id.unwrap_or_else(|| use_id("input"));
    let has_icon = icon.is_some();

    let input = view! {
        <input
            id=id
            type=input_type
            class=move || {
                field_classes(size, invalid.get(), &cn!(has_icon.then_some("pl-10"), & class))
            }
            prop:value=move || value.get()
            placeholder=placeholder
            disabled=move || disabled.get()
            aria-invalid=move || invalid.get().then_some("true")
            on:input=move |ev| {
                if let Some(v) = value_of(&ev) {
                    on_input.run(v);
                }
            }
        />
    };

    match icon {
        Some(i) => view! {
            <div class="relative">
                <Icon
                    icon=i
                    class="pointer-events-none absolute left-3.5 top-1/2 h-4 w-4 -translate-y-1/2 text-accent-400"
                />
                {input}
            </div>
        }
        .into_any(),
        None => input.into_any(),
    }
}

#[component]
pub fn TextArea(
    #[prop(into)] value: Signal<String>,
    #[prop(into)] on_input: Callback<String>,
    #[prop(into, optional)] placeholder: Option<String>,
    #[prop(default = 4)] rows: u32,
    #[prop(optional)] size: Size,
    #[prop(into, default = Signal::from(false))] disabled: Signal<bool>,
    #[prop(into, default = Signal::from(false))] invalid: Signal<bool>,
    #[prop(into, optional)] id: Option<String>,
    #[prop(into, optional)] class: String,
) -> impl IntoView {
    let id = id.unwrap_or_else(|| use_id("textarea"));
    view! {
        <textarea
            id=id
            rows=rows
            class=move || field_classes(size, invalid.get(), &cn!("resize-y leading-6", & class))
            prop:value=move || value.get()
            placeholder=placeholder
            disabled=move || disabled.get()
            aria-invalid=move || invalid.get().then_some("true")
            on:input=move |ev| {
                if let Some(v) = value_of(&ev) {
                    on_input.run(v);
                }
            }
        ></textarea>
    }
}

#[component]
pub fn SearchInput(
    #[prop(into)] value: Signal<String>,
    #[prop(into)] on_input: Callback<String>,
    #[prop(into, default = "Filter…".to_string())] placeholder: String,
    #[prop(optional)] size: Size,
    #[prop(into, optional)] class: String,
) -> impl IntoView {
    view! {
        <label class=cn!(
            SUNKEN,
            "flex items-center gap-2 rounded-2xl text-white/70",
            size.field(),
            TRANSITION,
            "focus-within:border-accent-400/40",
            class,
        )>
            <Icon icon=SEARCH class="h-3.5 w-3.5 shrink-0 text-accent-400" />
            <input
                type="search"
                class="w-full min-w-0 bg-transparent text-inherit outline-none placeholder:text-white/30 [&::-webkit-search-cancel-button]:hidden"
                prop:value=move || value.get()
                placeholder=placeholder
                on:input=move |ev| {
                    if let Some(v) = value_of(&ev) {
                        on_input.run(v);
                    }
                }
            />
            <Show when=move || !value.get().is_empty()>
                <button
                    type="button"
                    aria-label="Clear search"
                    class="shrink-0 rounded-md p-0.5 text-white/35 hover:text-white"
                    on:click=move |_| on_input.run(String::new())
                >
                    <Icon icon=X class="h-3.5 w-3.5" />
                </button>
            </Show>
        </label>
    }
}

#[component]
pub fn Switch(
    #[prop(into)] checked: Signal<bool>,
    #[prop(into)] on_change: Callback<bool>,
    #[prop(into, optional)] label: Option<String>,
    #[prop(into, optional)] description: Option<String>,
    #[prop(optional)] tone: Tone,
    #[prop(into, default = Signal::from(false))] disabled: Signal<bool>,
    #[prop(into, optional)] class: String,
) -> impl IntoView {
    let track_on = match tone {
        Tone::Accent => "bg-accent-500/70 border-accent-400/40",
        Tone::Neutral => "bg-white/30 border-white/25",
        Tone::Info => "bg-sky-500/70 border-sky-400/40",
        Tone::Warning => "bg-amber-500/70 border-amber-400/40",
        Tone::Danger => "bg-red-500/70 border-red-400/40",
        Tone::Dev => "bg-violet-500/70 border-violet-400/40",
    };

    view! {
        <label class=cn!(
            "flex items-start gap-3",
            TRANSITION,
            "cursor-pointer has-[:disabled]:cursor-not-allowed has-[:disabled]:opacity-50",
            class,
        )>
            <button
                type="button"
                role="switch"
                class=move || {
                    cn!(
                        "relative mt-0.5 inline-flex h-6 w-11 shrink-0 items-center rounded-full border",
                        TRANSITION, FOCUS_RING, tone.ring(), if checked.get() { track_on } else {
                        "border-white/10 bg-black/40" },
                    )
                }
                aria-checked=move || if checked.get() { "true" } else { "false" }
                disabled=move || disabled.get()
                on:click=move |_| on_change.run(!checked.get_untracked())
            >
                <span
                    class="inline-block h-4 w-4 rounded-full bg-white shadow transition-transform duration-150"
                    style:transform=move || {
                        if checked.get() {
                            "translateX(1.5rem)"
                        } else {
                            "translateX(0.25rem)"
                        }
                    }
                ></span>
            </button>
            {label
                .map(|l| {
                    view! {
                        <span class="min-w-0">
                            <span class="block text-sm text-white/85">{l}</span>
                            {description
                                .map(|d| {
                                    view! {
                                        <span class="mt-0.5 block text-xs leading-5 text-white/45">
                                            {d}
                                        </span>
                                    }
                                })}
                        </span>
                    }
                })}
        </label>
    }
}

#[component]
pub fn Checkbox(
    #[prop(into)] checked: Signal<bool>,
    #[prop(into)] on_change: Callback<bool>,
    #[prop(into, optional)] label: Option<String>,
    #[prop(into, default = Signal::from(false))] disabled: Signal<bool>,
    #[prop(into, optional)] class: String,
) -> impl IntoView {
    view! {
        <label class=cn!(
            "inline-flex cursor-pointer items-center gap-2.5 text-sm text-white/80",
            "has-[:disabled]:cursor-not-allowed has-[:disabled]:opacity-50",
            class,
        )>
            <span class="relative inline-flex">
                <input
                    type="checkbox"
                    class=cn!(
                        "peer h-4 w-4 appearance-none rounded-md border border-white/15 bg-black/30",
                        TRANSITION,
                        FOCUS_RING,
                        "checked:border-accent-400/60 checked:bg-accent-500/80",
                    )
                    prop:checked=move || checked.get()
                    disabled=move || disabled.get()
                    on:change=move |_| on_change.run(!checked.get_untracked())
                />
                <svg
                    viewBox="0 0 16 16"
                    fill="none"
                    class="pointer-events-none absolute left-0 top-0 h-4 w-4 scale-75 opacity-0 transition peer-checked:scale-100 peer-checked:opacity-100"
                >
                    <path
                        d="M3.5 8.5l3 3 6-6"
                        stroke="#04140d"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    />
                </svg>
            </span>
            {label}
        </label>
    }
}

#[component]
pub fn Radio(
    #[prop(into)] name: String,
    #[prop(into)] value: String,
    #[prop(into)] selected: Signal<String>,
    #[prop(into)] on_change: Callback<String>,
    #[prop(into, optional)] label: Option<String>,
    #[prop(into, default = Signal::from(false))] disabled: Signal<bool>,
    #[prop(into, optional)] class: String,
) -> impl IntoView {
    let this = value.clone();
    let emit = value.clone();
    view! {
        <label class=cn!(
            "inline-flex cursor-pointer items-center gap-2.5 text-sm text-white/80",
            "has-[:disabled]:cursor-not-allowed has-[:disabled]:opacity-50",
            class,
        )>
            <input
                type="radio"
                name=name
                value=value
                class=cn!(
                    "h-4 w-4 appearance-none rounded-full border border-white/15 bg-black/30",
                    TRANSITION,
                    FOCUS_RING,
                    "checked:border-[5px] checked:border-accent-400",
                )
                prop:checked=move || selected.get() == this
                disabled=move || disabled.get()
                on:change=move |_| on_change.run(emit.clone())
            />
            {label}
        </label>
    }
}

#[component]
pub fn Slider(
    #[prop(into)] value: Signal<f64>,
    #[prop(into)] on_input: Callback<f64>,
    #[prop(default = 0.0)] min: f64,
    #[prop(default = 100.0)] max: f64,
    #[prop(default = 1.0)] step: f64,
    #[prop(optional)] show_value: bool,
    #[prop(into, default = Signal::from(false))] disabled: Signal<bool>,
    #[prop(into, optional)] class: String,
) -> impl IntoView {
    view! {
        <div class=cn!("flex items-center gap-3", class)>
            <input
                type="range"
                class="iui-slider h-1.5 w-full min-w-0 cursor-pointer appearance-none rounded-full bg-white/10 disabled:cursor-not-allowed disabled:opacity-50"
                min=min
                max=max
                step=step
                prop:value=move || value.get()
                disabled=move || disabled.get()
                on:input=move |ev| {
                    if let Some(v) = event_target_value(&ev).parse::<f64>().ok() {
                        on_input.run(v);
                    }
                }
            />
            {show_value
                .then(|| {
                    view! {
                        <span class="w-12 shrink-0 text-right text-xs tabular-nums text-white/55">
                            {move || value.get()}
                        </span>
                    }
                })}
        </div>
    }
}
