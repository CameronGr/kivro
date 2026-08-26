use leptos::html::Div;
use leptos::prelude::*;

use crate::cn;
use crate::ui::hooks::{use_click_outside, use_escape, use_id};
use crate::ui::icons::{CHECK, CHEVRON_DOWN, Icon};
use crate::ui::style::{FOCUS_RING, GLASS_RAISED, SUNKEN, TRANSITION};
use crate::ui::theme::Size;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
    pub disabled: bool,
}

impl SelectOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectGroup {
    pub label: String,
    pub options: Vec<SelectOption>,
}

impl SelectGroup {
    pub fn new(label: impl Into<String>, options: Vec<SelectOption>) -> Self {
        Self {
            label: label.into(),
            options,
        }
    }
}

#[component]
pub fn Select(
    #[prop(into)] value: Signal<String>,
    #[prop(into)] on_change: Callback<String>,
    #[prop(into)] groups: Signal<Vec<SelectGroup>>,
    #[prop(into, default = "Select…".to_string())] placeholder: String,
    #[prop(optional)] size: Size,
    #[prop(into, default = Signal::from(false))] disabled: Signal<bool>,
    #[prop(into, optional)] class: String,
) -> impl IntoView {
    let open = RwSignal::new(false);
    let root: NodeRef<Div> = NodeRef::new();
    let listbox_id = use_id("listbox");

    use_click_outside(root, move || open.set(false));
    use_escape(move || {
        if open.get_untracked() {
            open.set(false);
        }
    });

    let navigable = Signal::derive(move || {
        groups
            .get()
            .into_iter()
            .flat_map(|g| g.options)
            .filter(|o| !o.disabled)
            .map(|o| o.value)
            .collect::<Vec<_>>()
    });

    let selected_label = Signal::derive({
        let placeholder = placeholder.clone();
        move || {
            let current = value.get();
            groups
                .get()
                .iter()
                .flat_map(|g| &g.options)
                .find(|o| o.value == current)
                .map(|o| o.label.clone())
                .unwrap_or_else(|| placeholder.clone())
        }
    });
    let has_selection = Signal::derive(move || navigable.get().iter().any(|v| *v == value.get()));

    let commit = move |v: String| {
        on_change.run(v);
        open.set(false);
    };

    let step = move |delta: isize| {
        let options = navigable.get_untracked();
        if options.is_empty() {
            return;
        }
        let current = value.get_untracked();
        let idx = options.iter().position(|v| *v == current);
        let next = match idx {
            None if delta > 0 => 0,
            None => options.len() - 1,
            Some(i) => (i as isize + delta).clamp(0, options.len() as isize - 1) as usize,
        };
        on_change.run(options[next].clone());
    };

    let indexed_groups = move || groups.get().into_iter().enumerate().collect::<Vec<_>>();

    view! {
        <div node_ref=root class=cn!("relative", class)>
            <button
                type="button"
                class=move || {
                    cn!(
                        "inline-flex w-full items-center justify-between gap-3 rounded-2xl text-white",
                        SUNKEN, size.field(), TRANSITION, FOCUS_RING,
                        "backdrop-blur-xl hover:border-white/20 hover:bg-white/[0.06]", open.get()
                        .then_some("border-accent-400/30 bg-accent-500/[0.06]"),
                        "disabled:cursor-not-allowed disabled:opacity-50",
                    )
                }
                aria-haspopup="listbox"
                aria-expanded=move || if open.get() { "true" } else { "false" }
                aria-controls=listbox_id.clone()
                disabled=move || disabled.get()
                on:click=move |_| open.update(|o| *o = !*o)
                on:keydown=move |ev| {
                    match ev.key().as_str() {
                        "ArrowDown" | "ArrowUp" if !open.get_untracked() => {
                            ev.prevent_default();
                            open.set(true);
                        }
                        "ArrowDown" => {
                            ev.prevent_default();
                            step(1);
                        }
                        "ArrowUp" => {
                            ev.prevent_default();
                            step(-1);
                        }
                        "Enter" | " " => {
                            ev.prevent_default();
                            open.update(|o| *o = !*o);
                        }
                        "Home" => {
                            ev.prevent_default();
                            if let Some(first) = navigable.get_untracked().first() {
                                on_change.run(first.clone());
                            }
                        }
                        "End" => {
                            ev.prevent_default();
                            if let Some(last) = navigable.get_untracked().last() {
                                on_change.run(last.clone());
                            }
                        }
                        _ => {}
                    }
                }
            >
                <span class=move || {
                    cn!(
                        "min-w-0 truncate", if has_selection.get() { "text-white" } else {
                        "text-white/40" },
                    )
                }>{move || selected_label.get()}</span>
                <span class=move || {
                    cn!(
                        "inline-flex shrink-0 text-accent-400 transition-transform", open.get()
                        .then_some("rotate-180"),
                    )
                }>
                    <Icon icon=CHEVRON_DOWN class="h-4 w-4" />
                </span>
            </button>

            <Show when=move || open.get()>
                <div class=cn!(
                    GLASS_RAISED,
                    "iui-pop absolute z-50 mt-2 w-full overflow-hidden rounded-xl",
                )>
                    <div
                        id=listbox_id.clone()
                        role="listbox"
                        class="max-h-[60vh] overflow-y-auto p-2 outline-none"
                    >
                        <For
                            each=indexed_groups
                            key=|(idx, g)| format!("{idx}-{}", g.label)
                            let:entry
                        >
                            {
                            let group = entry.1;
                            view! {
                            <div class="mb-2 last:mb-0">
                                <Show when={
                                    let has = !group.label.is_empty();
                                    move || has
                                }>
                                    <div class="px-3 pb-1 pt-2 text-[10px] font-semibold uppercase tracking-[0.2em] text-white/35">
                                        {group.label.clone()}
                                    </div>
                                </Show>
                                <div class="space-y-0.5">
                                    {group
                                        .options
                                        .iter()
                                        .cloned()
                                        .map(|opt| {
                                            let opt_value = opt.value.clone();
                                            let is_selected = Signal::derive({
                                                let v = opt.value.clone();
                                                move || value.get() == v
                                            });
                                            view! {
                                                <button
                                                    type="button"
                                                    role="option"
                                                    aria-selected=move || {
                                                        if is_selected.get() { "true" } else { "false" }
                                                    }
                                                    disabled=opt.disabled
                                                    class=move || {
                                                        cn!(
                                                            "flex w-full items-center justify-between gap-3 rounded-xl border px-3 py-2",
                                                            "text-left text-sm", TRANSITION, if is_selected.get() {
                                                            "border-accent-400/20 bg-accent-500/15 text-accent-200" } else {
                                                            "border-transparent text-white/70 hover:bg-white/[0.05] hover:text-white" },
                                                            "disabled:pointer-events-none disabled:opacity-40",
                                                        )
                                                    }
                                                    on:click=move |_| commit(opt_value.clone())
                                                >
                                                    <span class="min-w-0 truncate">{opt.label.clone()}</span>
                                                    <Show when=move || is_selected.get()>
                                                        <Icon
                                                            icon=CHECK
                                                            class="h-4 w-4 shrink-0 text-accent-400"
                                                        />
                                                    </Show>
                                                </button>
                                            }
                                        })
                                        .collect_view()}
                                </div>
                            </div>
                            }}
                        </For>
                    </div>
                </div>
            </Show>
        </div>
    }
}
