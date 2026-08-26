use leptos::prelude::*;

use crate::cn;
use crate::ui::icons::{CHEVRON_DOWN, CHEVRON_LEFT, CHEVRON_RIGHT, Icon, IconData};
use crate::ui::style::{FOCUS_RING, GLASS, SUNKEN, TRANSITION};
use crate::ui::theme::Tone;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TabItem {
    pub value: String,
    pub label: String,
    pub icon: Option<IconData>,
    pub badge: Option<String>,
    pub disabled: bool,
}

impl TabItem {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            icon: None,
            badge: None,
            disabled: false,
        }
    }

    pub fn with_icon(mut self, icon: IconData) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn with_badge(mut self, badge: impl Into<String>) -> Self {
        self.badge = Some(badge.into());
        self
    }

    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

#[component]
pub fn Tabs(
    #[prop(into)] value: Signal<String>,
    #[prop(into)] on_change: Callback<String>,
    #[prop(into)] items: Signal<Vec<TabItem>>,
    #[prop(optional)] full_width: bool,
    #[prop(into, optional)] class: String,
) -> impl IntoView {
    view! {
        <div
            role="tablist"
            class=cn!(
                SUNKEN,
                "inline-flex items-center gap-1 rounded-2xl p-1",
                full_width.then_some("flex w-full"),
                class,
            )
        >
            <For each=move || items.get() key=|t| t.value.clone() let:tab>
                {
                    let this = tab.value.clone();
                    let emit = tab.value.clone();
                    let active = Signal::derive(move || value.get() == this);
                    view! {
                        <button
                            type="button"
                            role="tab"
                            aria-selected=move || if active.get() { "true" } else { "false" }
                            disabled=tab.disabled
                            class=move || {
                                cn!(
                                    "inline-flex items-center justify-center gap-2 rounded-xl px-3.5 py-1.5",
                                    "text-sm whitespace-nowrap", TRANSITION, FOCUS_RING, full_width
                                    .then_some("flex-1"), if active.get() {
                                    "bg-accent-500/15 text-accent-200 shadow-[inset_0_0_0_1px_rgb(var(--ac-400)/0.25)]"
                                    } else { "text-white/55 hover:bg-white/[0.05] hover:text-white" },
                                    "disabled:pointer-events-none disabled:opacity-40",
                                )
                            }
                            on:click={
                                let emit = emit.clone();
                                move |_| on_change.run(emit.clone())
                            }
                        >
                            {tab.icon.map(|i| view! { <Icon icon=i class="h-4 w-4" /> })}
                            {tab.label.clone()}
                            {tab
                                .badge
                                .clone()
                                .map(|b| {
                                    view! {
                                        <span class="rounded-full bg-white/10 px-1.5 py-0.5 text-[10px] tabular-nums text-white/60">
                                            {b}
                                        </span>
                                    }
                                })}
                        </button>
                    }
                }
            </For>
        </div>
    }
}

#[component]
pub fn Accordion(
    #[prop(into)] title: String,
    #[prop(into, optional)] summary: Option<String>,
    #[prop(optional)] icon: Option<IconData>,
    #[prop(optional)] open_by_default: bool,
    #[prop(into, optional)] class: String,
    children: ChildrenFn,
) -> impl IntoView {
    let open = RwSignal::new(open_by_default);
    view! {
        <div class=cn!(
            GLASS,
            "overflow-hidden rounded-2xl",
            TRANSITION,
            class,
        )>
            <button
                type="button"
                class=cn!(
                    "flex w-full items-center gap-3 px-5 py-4 text-left",
                    TRANSITION,
                    FOCUS_RING,
                    "hover:bg-white/[0.03]",
                )
                aria-expanded=move || if open.get() { "true" } else { "false" }
                on:click=move |_| open.update(|o| *o = !*o)
            >
                {icon.map(|i| view! { <Icon icon=i class="h-4 w-4 text-accent-400" /> })}
                <span class="min-w-0 flex-1">
                    <span class="block text-sm font-medium text-white">{title}</span>
                    {summary
                        .map(|s| {
                            view! {
                                <span class="mt-0.5 block truncate text-xs text-white/45">{s}</span>
                            }
                        })}
                </span>
                <span class=move || {
                    cn!(
                        "inline-flex shrink-0 text-white/40 transition-transform", open.get()
                        .then_some("rotate-180"),
                    )
                }>
                    <Icon icon=CHEVRON_DOWN class="h-4 w-4" />
                </span>
            </button>
            <Show when=move || open.get()>
                <div class="border-t border-white/[0.06] px-5 py-4 text-sm leading-6 text-white/68">
                    {children()}
                </div>
            </Show>
        </div>
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Crumb {
    pub label: String,
    pub href: Option<String>,
}

impl Crumb {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            href: None,
        }
    }

    pub fn link(label: impl Into<String>, href: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            href: Some(href.into()),
        }
    }
}

#[component]
pub fn Breadcrumbs(
    #[prop(into)] items: Vec<Crumb>,
    #[prop(into, optional)] class: String,
) -> impl IntoView {
    let last = items.len().saturating_sub(1);
    view! {
        <nav aria-label="Breadcrumb" class=cn!("flex items-center gap-1.5 text-xs", class)>
            {items
                .into_iter()
                .enumerate()
                .map(|(i, crumb)| {
                    let is_last = i == last;
                    view! {
                        {(i > 0)
                            .then(|| {
                                view! { <Icon icon=CHEVRON_RIGHT class="h-3 w-3 text-white/25" /> }
                            })}
                        {match crumb.href.filter(|_| !is_last) {
                            Some(href) => {
                                view! {
                                    <a
                                        href=href
                                        class=cn!(
                                            "uppercase tracking-[0.14em] text-white/45 hover:text-accent-300",
                                            TRANSITION,
                                        )
                                    >
                                        {crumb.label}
                                    </a>
                                }
                                    .into_any()
                            }
                            None => {
                                view! {
                                    <span
                                        aria-current=is_last.then_some("page")
                                        class="uppercase tracking-[0.14em] text-white/70"
                                    >
                                        {crumb.label}
                                    </span>
                                }
                                    .into_any()
                            }
                        }}
                    }
                })
                .collect_view()}
        </nav>
    }
}

#[component]
pub fn Pagination(
    #[prop(into)] page: Signal<usize>,
    #[prop(into)] total: Signal<usize>,
    #[prop(into)] on_change: Callback<usize>,
    #[prop(into, optional)] class: String,
) -> impl IntoView {
    let pages = Signal::derive(move || page_window(page.get(), total.get()));

    let step_button = move |delta: isize, icon: IconData, label: &'static str| {
        view! {
            <button
                type="button"
                aria-label=label
                class=cn!(
                    SUNKEN,
                    "inline-flex h-8 w-8 items-center justify-center rounded-xl text-white/60",
                    TRANSITION,
                    FOCUS_RING,
                    "hover:border-accent-400/25 hover:text-white",
                    "disabled:pointer-events-none disabled:opacity-30",
                )
                disabled=move || {
                    let next = page.get() as isize + delta;
                    next < 1 || next > total.get() as isize
                }
                on:click=move |_| {
                    let next = page.get_untracked() as isize + delta;
                    if next >= 1 && next <= total.get_untracked() as isize {
                        on_change.run(next as usize);
                    }
                }
            >
                <Icon icon=icon class="h-4 w-4" />
            </button>
        }
    };

    view! {
        <nav aria-label="Pagination" class=cn!("flex items-center gap-1.5", class)>
            {step_button(-1, CHEVRON_LEFT, "Previous page")}
            <For
                each=move || pages.get()
                key=|slot| match slot {
                    PageSlot::Number(n) => format!("n{n}"),
                    PageSlot::Gap(side) => format!("gap{side}"),
                }
                let:slot
            >
                {match slot {
                    PageSlot::Gap(_) => {
                        view! { <span class="px-1 text-sm text-white/25">"…"</span> }.into_any()
                    }
                    PageSlot::Number(n) => {
                        let active = Signal::derive(move || page.get() == n);
                        view! {
                            <button
                                type="button"
                                aria-current=move || active.get().then_some("page")
                                class=move || {
                                    cn!(
                                        "inline-flex h-8 min-w-8 items-center justify-center rounded-xl border px-2",
                                        "text-sm tabular-nums", TRANSITION, FOCUS_RING, if active.get() {
                                        "border-accent-400/30 bg-accent-500/15 text-accent-200" } else {
                                        "border-white/10 bg-black/20 text-white/60 hover:border-accent-400/25 hover:text-white"
                                        },
                                    )
                                }
                                on:click=move |_| on_change.run(n)
                            >
                                {n}
                            </button>
                        }
                            .into_any()
                    }
                }}
            </For>
            {step_button(1, CHEVRON_RIGHT, "Next page")}
        </nav>
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PageSlot {
    Number(usize),
    Gap(u8),
}

fn page_window(current: usize, total: usize) -> Vec<PageSlot> {
    if total == 0 {
        return Vec::new();
    }
    let current = current.clamp(1, total);
    if total <= 7 {
        return (1..=total).map(PageSlot::Number).collect();
    }

    let mut out = vec![PageSlot::Number(1)];
    let start = current.saturating_sub(1).max(2);
    let end = (current + 1).min(total - 1);

    if start > 2 {
        out.push(PageSlot::Gap(0));
    }
    for n in start..=end {
        out.push(PageSlot::Number(n));
    }
    if end < total - 1 {
        out.push(PageSlot::Gap(1));
    }
    out.push(PageSlot::Number(total));
    out
}

#[component]
pub fn StickyBar(
    #[prop(into, default = "top-0".to_string())] top: String,
    #[prop(into, optional)] class: String,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=cn!(
            "sticky z-40 border-b border-white/10 bg-black/40 px-4 py-3 backdrop-blur-sm md:px-6",
            top,
            class,
        )>
            <div class="flex items-center gap-2 md:gap-3">{children()}</div>
        </div>
    }
}

#[component]
pub fn NavList(
    #[prop(into)] items: Signal<Vec<NavItem>>,
    #[prop(into)] active: Signal<String>,
    #[prop(into, optional)] on_select: Option<Callback<String>>,
    #[prop(into, optional)] class: String,
) -> impl IntoView {
    view! {
        <nav class=cn!("flex flex-col gap-0.5", class)>
            <For each=move || items.get() key=|i| i.value.clone() let:item>
                {
                    let this = item.value.clone();
                    let emit = item.value.clone();
                    let is_active = Signal::derive(move || active.get() == this);
                    let cls = move || {
                        cn!(
                            "flex items-center gap-2.5 rounded-2xl border px-3 py-2 text-left text-sm",
                            TRANSITION, if is_active.get() {
                            "border-accent-400/30 bg-accent-500/14 text-white" } else {
                            "border-transparent text-white/60 hover:border-white/10 hover:bg-white/[0.04] hover:text-white"
                            },
                        )
                    };
                    match item.href.clone() {
                        Some(href) => {
                            view! {
                                <a href=href class=cls>
                                    {item.icon.map(|i| view! { <Icon icon=i class="h-4 w-4" /> })}
                                    <span class="min-w-0 truncate">{item.label.clone()}</span>
                                </a>
                            }
                                .into_any()
                        }
                        None => {
                            view! {
                                <button
                                    type="button"
                                    class=cls
                                    on:click=move |_| {
                                        if let Some(cb) = on_select {
                                            cb.run(emit.clone());
                                        }
                                    }
                                >
                                    {item.icon.map(|i| view! { <Icon icon=i class="h-4 w-4" /> })}
                                    <span class="min-w-0 truncate">{item.label.clone()}</span>
                                </button>
                            }
                                .into_any()
                        }
                    }
                }
            </For>
        </nav>
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavItem {
    pub value: String,
    pub label: String,
    pub icon: Option<IconData>,
    pub href: Option<String>,
}

impl NavItem {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            icon: None,
            href: None,
        }
    }

    pub fn with_icon(mut self, icon: IconData) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn with_href(mut self, href: impl Into<String>) -> Self {
        self.href = Some(href.into());
        self
    }
}

#[component]
pub fn Link(
    #[prop(into)] href: String,
    #[prop(optional)] external: bool,
    #[prop(optional)] tone: Tone,
    #[prop(into, optional)] class: String,
    children: Children,
) -> impl IntoView {
    view! {
        <a
            href=href
            target=external.then_some("_blank")
            rel=external.then_some("noreferrer noopener")
            class=cn!(tone.text(), "underline-offset-4 hover:underline", TRANSITION, class)
        >
            {children()}
            {external.then(|| view! { <span aria-hidden="true">" ↗"</span> })}
        </a>
    }
}

#[cfg(test)]
mod tests {
    use super::{PageSlot::*, page_window};

    #[test]
    fn short_ranges_are_not_collapsed() {
        assert_eq!(
            page_window(3, 5),
            vec![Number(1), Number(2), Number(3), Number(4), Number(5)]
        );
    }

    #[test]
    fn middle_of_a_long_range_gets_both_gaps() {
        assert_eq!(
            page_window(10, 20),
            vec![
                Number(1),
                Gap(0),
                Number(9),
                Number(10),
                Number(11),
                Gap(1),
                Number(20)
            ]
        );
    }

    #[test]
    fn near_the_start_only_the_trailing_gap_appears() {
        assert_eq!(
            page_window(2, 20),
            vec![Number(1), Number(2), Number(3), Gap(1), Number(20)]
        );
    }

    #[test]
    fn near_the_end_only_the_leading_gap_appears() {
        assert_eq!(
            page_window(19, 20),
            vec![Number(1), Gap(0), Number(18), Number(19), Number(20)]
        );
    }

    #[test]
    fn out_of_range_and_empty_inputs_are_safe() {
        assert!(page_window(1, 0).is_empty());
        assert_eq!(page_window(99, 3), vec![Number(1), Number(2), Number(3)]);
    }
}
