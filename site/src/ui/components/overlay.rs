use leptos::html::Div;
use leptos::prelude::*;

use crate::cn;
use crate::ui::hooks::{use_click_outside, use_escape, use_id, use_scroll_lock};
use crate::ui::icons::{Icon, IconData, X};
use crate::ui::style::{GLASS_RAISED, TRANSITION};
use crate::ui::theme::{Size, Tone};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Side {
    Left,
    #[default]
    Right,
    Bottom,
}

#[component]
pub fn Modal(
    #[prop(into)] open: Signal<bool>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into, optional)] title: Option<String>,
    #[prop(into, optional)] description: Option<String>,
    #[prop(optional)] icon: Option<IconData>,
    #[prop(optional)] tone: Tone,
    #[prop(into, default = "max-w-lg".to_string())] width: String,
    #[prop(optional)] footer: Option<ViewFn>,
    #[prop(into, optional)] class: String,
    children: ChildrenFn,
) -> impl IntoView {
    let panel: NodeRef<Div> = NodeRef::new();
    let title_id = use_id("modal-title");

    use_scroll_lock(open);
    use_escape(move || {
        if open.get_untracked() {
            on_close.run(());
        }
    });
    use_click_outside(panel, move || {
        if open.get_untracked() {
            on_close.run(());
        }
    });

    view! {
        <Show when=move || open.get()>
            <div class="fixed inset-0 z-[100] flex items-center justify-center p-4">
                <div class="iui-fade absolute inset-0 bg-black/70 backdrop-blur-sm"></div>
                <div
                    node_ref=panel
                    role="dialog"
                    aria-modal="true"
                    aria-labelledby=title_id.clone()
                    class=cn!(
                        GLASS_RAISED,
                        "iui-pop relative z-10 flex max-h-[85vh] w-full flex-col rounded-2xl",
                        width.clone(),
                        class.clone(),
                    )
                >
                    <div class="flex items-start justify-between gap-4 border-b border-white/[0.08] p-5 md:p-6">
                        <div class="flex min-w-0 items-center gap-3">
                            {icon
                                .map(|i| {
                                    view! {
                                        <div class=cn!(
                                            "flex h-10 w-10 shrink-0 items-center justify-center rounded-2xl border",
                                            tone.border(),
                                            tone.bg(),
                                            tone.icon(),
                                        )>
                                            <Icon icon=i class="h-4 w-4" />
                                        </div>
                                    }
                                })}
                            <div class="min-w-0">
                                <h2
                                    id=title_id.clone()
                                    class="truncate text-lg font-semibold tracking-tight text-white"
                                >
                                    {title.clone().unwrap_or_default()}
                                </h2>
                                {description
                                    .clone()
                                    .map(|d| {
                                        view! {
                                            <p class="mt-1 text-sm leading-6 text-white/55">{d}</p>
                                        }
                                    })}
                            </div>
                        </div>
                        <button
                            type="button"
                            aria-label="Close"
                            class=cn!(
                                "-m-1 shrink-0 rounded-xl p-1.5 text-white/45 hover:bg-white/[0.06] hover:text-white",
                                TRANSITION,
                            )
                            on:click=move |_| on_close.run(())
                        >
                            <Icon icon=X class="h-4 w-4" />
                        </button>
                    </div>

                    <div class="min-h-0 flex-1 overflow-y-auto p-5 text-sm leading-6 text-white/70 md:p-6">
                        {children()}
                    </div>

                    {footer
                        .clone()
                        .map(|f| {
                            view! {
                                <div class="flex items-center justify-end gap-2 border-t border-white/[0.08] p-4 md:px-6">
                                    {f.run()}
                                </div>
                            }
                        })}
                </div>
            </div>
        </Show>
    }
}

#[component]
pub fn Drawer(
    #[prop(into)] open: Signal<bool>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(optional)] side: Side,
    #[prop(into, optional)] title: Option<String>,
    #[prop(into, optional)] size: Option<String>,
    #[prop(into, optional)] class: String,
    children: ChildrenFn,
) -> impl IntoView {
    let panel: NodeRef<Div> = NodeRef::new();
    use_scroll_lock(open);
    use_escape(move || {
        if open.get_untracked() {
            on_close.run(());
        }
    });
    use_click_outside(panel, move || {
        if open.get_untracked() {
            on_close.run(());
        }
    });

    let (position, motion, extent) = match side {
        Side::Left => (
            "inset-y-0 left-0",
            "iui-slide-left",
            "h-full w-full max-w-sm",
        ),
        Side::Right => (
            "inset-y-0 right-0",
            "iui-slide-right",
            "h-full w-full max-w-sm",
        ),
        Side::Bottom => (
            "inset-x-0 bottom-0",
            "iui-slide-up",
            "max-h-[85vh] w-full rounded-t-3xl",
        ),
    };

    view! {
        <Show when=move || open.get()>
            <div class="fixed inset-0 z-[100]">
                <div class="iui-fade absolute inset-0 bg-black/70 backdrop-blur-sm"></div>
                <div
                    node_ref=panel
                    role="dialog"
                    aria-modal="true"
                    aria-label=title.clone()
                    class=cn!(
                        GLASS_RAISED,
                        "absolute flex flex-col",
                        position,
                        motion,
                        size.clone().unwrap_or_else(|| extent.to_string()),
                        class.clone(),
                    )
                >
                    {title
                        .clone()
                        .map(|t| {
                            view! {
                                <div class="flex items-center justify-between gap-4 border-b border-white/[0.08] p-5">
                                    <h2 class="truncate text-base font-semibold text-white">{t}</h2>
                                    <button
                                        type="button"
                                        aria-label="Close"
                                        class=cn!(
                                            "-m-1 rounded-xl p-1.5 text-white/45 hover:bg-white/[0.06] hover:text-white",
                                            TRANSITION,
                                        )
                                        on:click=move |_| on_close.run(())
                                    >
                                        <Icon icon=X class="h-4 w-4" />
                                    </button>
                                </div>
                            }
                        })}
                    <div class="min-h-0 flex-1 overflow-y-auto p-5 text-sm leading-6 text-white/70">
                        {children()}
                    </div>
                </div>
            </div>
        </Show>
    }
}

#[component]
pub fn Tooltip(
    #[prop(into)] text: String,
    #[prop(optional)] placement: Placement,
    #[prop(into, optional)] class: String,
    children: Children,
) -> impl IntoView {
    let pos = match placement {
        Placement::Top => "bottom-full left-1/2 mb-2 -translate-x-1/2",
        Placement::Bottom => "top-full left-1/2 mt-2 -translate-x-1/2",
        Placement::Left => "right-full top-1/2 mr-2 -translate-y-1/2",
        Placement::Right => "left-full top-1/2 ml-2 -translate-y-1/2",
    };
    view! {
        <span class=cn!("group/tt relative inline-flex", class)>
            {children()}
            <span
                role="tooltip"
                class=cn!(
                    "pointer-events-none absolute z-50 hidden whitespace-nowrap rounded-xl border",
                    "border-white/10 bg-black/90 px-2.5 py-1.5 text-xs text-white/80 shadow-lg shadow-black/50",
                    "backdrop-blur-xl group-hover/tt:block group-focus-within/tt:block",
                    pos,
                )
            >
                {text}
            </span>
        </span>
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Placement {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
}

#[component]
pub fn Popover(
    trigger: ViewFn,
    #[prop(optional)] align_end: bool,
    #[prop(into, default = "w-64".to_string())] width: String,
    #[prop(into, optional)] class: String,
    children: ChildrenFn,
) -> impl IntoView {
    let open = RwSignal::new(false);
    let root: NodeRef<Div> = NodeRef::new();

    use_click_outside(root, move || open.set(false));
    use_escape(move || {
        if open.get_untracked() {
            open.set(false);
        }
    });

    view! {
        <div node_ref=root class=cn!("relative inline-flex", class)>
            <div
                class="contents"
                on:click=move |_| open.update(|o| *o = !*o)
            >
                {trigger.run()}
            </div>
            <Show when=move || open.get()>
                <div class=cn!(
                    GLASS_RAISED,
                    "iui-pop absolute top-full z-50 mt-2 overflow-hidden rounded-2xl p-2",
                    if align_end { "right-0" } else { "left-0" },
                    width.clone(),
                )>{children()}</div>
            </Show>
        </div>
    }
}

#[component]
pub fn MenuItem(
    #[prop(optional)] icon: Option<IconData>,
    #[prop(optional)] tone: Tone,
    #[prop(into, optional)] on_click: Option<Callback<()>>,
    #[prop(into, default = Signal::from(false))] disabled: Signal<bool>,
    #[prop(into, optional)] class: String,
    children: Children,
) -> impl IntoView {
    view! {
        <button
            type="button"
            role="menuitem"
            class=cn!(
                "flex w-full items-center gap-2.5 rounded-xl px-3 py-2 text-left text-sm",
                TRANSITION,
                tone.text(),
                "hover:bg-white/[0.06] disabled:pointer-events-none disabled:opacity-40",
                class,
            )
            disabled=move || disabled.get()
            on:click=move |_| {
                if let Some(cb) = on_click {
                    cb.run(());
                }
            }
        >
            {icon.map(|i| view! { <Icon icon=i class=cn!("h-4 w-4", tone.icon()) /> })}
            {children()}
        </button>
    }
}

#[component]
pub fn MenuSeparator() -> impl IntoView {
    view! { <div class="my-1 h-px bg-white/[0.08]" role="separator"></div> }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Toast {
    pub id: u64,
    pub title: String,
    pub description: Option<String>,
    pub tone: Tone,
}

impl Toast {
    pub fn new(id: u64, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            description: None,
            tone: Tone::Accent,
        }
    }

    pub fn with_tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

#[component]
pub fn ToastHost(
    #[prop(into)] toasts: Signal<Vec<Toast>>,
    #[prop(into)] on_dismiss: Callback<u64>,
    #[prop(optional)] bottom: bool,
    #[prop(into, optional)] class: String,
) -> impl IntoView {
    view! {
        <div
            aria-live="polite"
            class=cn!(
                "pointer-events-none fixed right-4 z-[200] flex w-[min(24rem,calc(100vw-2rem))] flex-col gap-2",
                if bottom { "bottom-4 flex-col-reverse" } else { "top-20" },
                class,
            )
        >
            <For each=move || toasts.get() key=|t| t.id let:toast>
                <div class=cn!(
                    GLASS_RAISED,
                    "iui-pop pointer-events-auto flex items-start gap-3 rounded-2xl border p-4",
                    toast.tone.border(),
                )>
                    <Icon
                        icon=crate::ui::components::tone_icon(toast.tone)
                        class=cn!("mt-0.5 h-4 w-4 shrink-0", toast.tone.icon())
                    />
                    <div class="min-w-0 flex-1">
                        <div class="text-sm font-semibold text-white">{toast.title.clone()}</div>
                        {toast
                            .description
                            .clone()
                            .map(|d| {
                                view! {
                                    <div class="mt-1 text-sm leading-6 text-white/60">{d}</div>
                                }
                            })}
                    </div>
                    <button
                        type="button"
                        aria-label="Dismiss notification"
                        class="-m-1 shrink-0 rounded-lg p-1 text-white/35 hover:text-white"
                        on:click={
                            let id = toast.id;
                            move |_| on_dismiss.run(id)
                        }
                    >
                        <Icon icon=X class="h-3.5 w-3.5" />
                    </button>
                </div>
            </For>
        </div>
    }
}

#[component]
pub fn ConfirmDialog(
    #[prop(into)] open: Signal<bool>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into)] on_confirm: Callback<()>,
    #[prop(into)] title: String,
    #[prop(into)] message: String,
    #[prop(into, default = "Confirm".to_string())] confirm_label: String,
    #[prop(into, default = "Cancel".to_string())] cancel_label: String,
    #[prop(default = Tone::Danger)] tone: Tone,
) -> impl IntoView {
    use crate::ui::components::Button;
    use crate::ui::theme::Variant;

    let footer = ViewFn::from(move || {
        let cancel = cancel_label.clone();
        let confirm = confirm_label.clone();
        view! {
            <Button variant=Variant::Ghost size=Size::Md on_click=on_close>
                {cancel}
            </Button>
            <Button variant=Variant::Soft tone=tone size=Size::Md on_click=on_confirm>
                {confirm}
            </Button>
        }
    });

    view! {
        <Modal
            open=open
            on_close=on_close
            title=title
            tone=tone
            icon=crate::ui::components::tone_icon(tone)
            width="max-w-md"
            footer=footer
        >
            {message.clone()}
        </Modal>
    }
}
