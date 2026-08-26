use leptos::prelude::*;

use crate::cn;
use crate::ui::style::{GLASS, TRANSITION};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Align {
    #[default]
    Start,
    Center,
    End,
}

impl Align {
    pub const fn class(self) -> &'static str {
        match self {
            Align::Start => "text-left",
            Align::Center => "text-center",
            Align::End => "text-right",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Column {
    pub label: String,
    pub align: Align,
    pub width: Option<String>,
}

impl Column {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            align: Align::Start,
            width: None,
        }
    }

    pub fn aligned(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn width(mut self, width: impl Into<String>) -> Self {
        self.width = Some(width.into());
        self
    }
}

#[component]
pub fn Table(
    #[prop(into)] columns: Vec<Column>,
    #[prop(into, optional)] caption: Option<String>,
    #[prop(optional)] show_caption: bool,
    #[prop(into, optional)] class: String,
    children: Children,
) -> impl IntoView {
    let aligns: Vec<&'static str> = columns.iter().map(|c| c.align.class()).collect();
    provide_context(ColumnAligns(aligns));

    view! {
        <div class=cn!(GLASS, "overflow-hidden rounded-2xl", class)>
            <div class="overflow-x-auto">
                <table class="w-full border-collapse text-sm">
                    {caption
                        .map(|c| {
                            view! {
                                <caption class=cn!(
                                    "px-5 py-3 text-left text-xs uppercase tracking-[0.16em] text-white/40",
                                    (!show_caption).then_some("sr-only"),
                                )>{c}</caption>
                            }
                        })}
                    <thead>
                        <tr class="border-b border-white/10 bg-white/[0.03]">
                            {columns
                                .into_iter()
                                .map(|col| {
                                    view! {
                                        <th
                                            scope="col"
                                            class=cn!(
                                                "px-5 py-3 text-xs font-semibold uppercase tracking-[0.16em] text-white/45",
                                                col.align.class(),
                                                col.width,
                                            )
                                        >
                                            {col.label}
                                        </th>
                                    }
                                })
                                .collect_view()}
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-white/[0.06]">{children()}</tbody>
                </table>
            </div>
        </div>
    }
}

#[derive(Clone)]
struct ColumnAligns(Vec<&'static str>);

#[component]
pub fn Row(
    #[prop(into, default = Signal::from(false))] highlighted: Signal<bool>,
    #[prop(into, optional)] class: String,
    children: Children,
) -> impl IntoView {
    view! {
        <tr class=move || {
            cn!(
                TRANSITION, "hover:bg-white/[0.03]", highlighted.get()
                .then_some("bg-accent-500/[0.07]"), & class,
            )
        }>{children()}</tr>
    }
}

#[component]
pub fn Cell(
    #[prop(optional)] index: usize,
    #[prop(optional)] align: Option<Align>,
    #[prop(optional)] header: bool,
    #[prop(into, optional)] class: String,
    children: Children,
) -> impl IntoView {
    let align = align.map(Align::class).unwrap_or_else(|| {
        use_context::<ColumnAligns>()
            .and_then(|a| a.0.get(index).copied())
            .unwrap_or("text-left")
    });
    let cls = cn!("px-5 py-3 align-top leading-6", align, class);

    if header {
        view! {
            <th scope="row" class=cn!(cls, "font-medium text-white/85")>
                {children()}
            </th>
        }
        .into_any()
    } else {
        view! { <td class=cn!(cls, "text-white/68")>{children()}</td> }.into_any()
    }
}
