use std::sync::Arc;
use std::time::Duration;

use leptos::prelude::*;

use crate::cn;
use crate::ui::components::{AnimatedPage, Badge, SearchInput};
use crate::ui::hooks::{scroll_to_id, use_active_section, use_clipboard};
use crate::ui::icons::{BOOK_OPEN, CHECK, HASH, Icon, LINK_2};
use crate::ui::style::{GLASS, SCROLL_MARGIN, TRANSITION};
use crate::ui::theme::{Size, Tone};

#[derive(Clone)]
pub struct DocSection {
    pub slug: String,
    pub title: String,
    pub number: Option<String>,
    pub summary: Option<String>,
    pub render: Arc<dyn Fn() -> AnyView + Send + Sync>,
}

impl std::fmt::Debug for DocSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DocSection")
            .field("slug", &self.slug)
            .field("title", &self.title)
            .field("number", &self.number)
            .finish_non_exhaustive()
    }
}

impl DocSection {
    pub fn new<F, V>(slug: impl Into<String>, title: impl Into<String>, render: F) -> Self
    where
        F: Fn() -> V + Send + Sync + 'static,
        V: IntoView + 'static,
    {
        Self {
            slug: slug.into(),
            title: title.into(),
            number: None,
            summary: None,
            render: Arc::new(move || render().into_any()),
        }
    }

    pub fn numbered(mut self, number: impl Into<String>) -> Self {
        self.number = Some(number.into());
        self
    }

    pub fn summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = Some(summary.into());
        self
    }

    fn haystack(&self) -> String {
        [
            Some(self.title.as_str()),
            self.summary.as_deref(),
            Some(self.slug.as_str()),
            self.number.as_deref(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
    }
}

#[derive(Clone, Debug)]
pub struct Doc {
    pub slug: String,
    pub title: String,
    pub eyebrow: String,
    pub category: String,
    pub tagline: Option<String>,
    pub tags: Vec<String>,
    pub sections: Vec<DocSection>,
}

impl Doc {
    pub fn new(
        slug: impl Into<String>,
        title: impl Into<String>,
        eyebrow: impl Into<String>,
        category: impl Into<String>,
    ) -> Self {
        Self {
            slug: slug.into(),
            title: title.into(),
            eyebrow: eyebrow.into(),
            category: category.into(),
            tagline: None,
            tags: Vec::new(),
            sections: Vec::new(),
        }
    }

    pub fn tagline(mut self, tagline: impl Into<String>) -> Self {
        self.tagline = Some(tagline.into());
        self
    }

    pub fn tags<I, S>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.tags = tags.into_iter().map(Into::into).collect();
        self
    }

    pub fn section(mut self, section: DocSection) -> Self {
        self.sections.push(section);
        self
    }
}

#[component]
pub fn DocToc(
    #[prop(into)] sections: Signal<Vec<DocSection>>,
    #[prop(into)] active: Signal<Option<String>>,
    #[prop(into)] on_pick: Callback<String>,
    #[prop(into, default = "On this page".to_string())] title: String,
    #[prop(into, optional)] class: String,
) -> impl IntoView {
    let query = RwSignal::new(String::new());
    let filtered = Signal::derive(move || {
        let needle = query.get().trim().to_lowercase();
        sections
            .get()
            .into_iter()
            .filter(|s| needle.is_empty() || s.haystack().contains(&needle))
            .collect::<Vec<_>>()
    });

    view! {
        <aside class=cn!(
            "flex w-full flex-col gap-4",
            "xl:sticky xl:top-[8rem] xl:max-h-[calc(100vh-9rem)] xl:w-[300px] xl:self-start",
            class,
        )>
            <div class=cn!(GLASS, "rounded-3xl p-4")>
                <div class="mb-3 flex items-center gap-2 text-white/80">
                    <Icon icon=BOOK_OPEN class="h-4 w-4 text-accent-400" />
                    <h2 class="text-sm font-semibold tracking-wide">{title}</h2>
                </div>

                <SearchInput
                    value=query
                    on_input=Callback::new(move |v| query.set(v))
                    size=Size::Sm
                    class="mb-3"
                />

                <div class="max-h-[60vh] space-y-1 overflow-auto pr-1">
                    <For each=move || filtered.get() key=|s| s.slug.clone() let:section>
                        {
                            let this = section.slug.clone();
                            let emit = section.slug.clone();
                            let is_active = Signal::derive(move || {
                                active.get().as_deref() == Some(this.as_str())
                            });
                            view! {
                                <button
                                    type="button"
                                    class=move || {
                                        cn!(
                                            "flex w-full items-start gap-2 rounded-2xl border px-3 py-2 text-left",
                                            TRANSITION, if is_active.get() {
                                            "border-accent-400/30 bg-accent-500/14 text-white" } else {
                                            "border-transparent bg-white/[0.02] text-white/60 hover:border-white/10 hover:bg-white/[0.04] hover:text-white"
                                            },
                                        )
                                    }
                                    on:click={
                                        let emit = emit.clone();
                                        move |_| on_pick.run(emit.clone())
                                    }
                                >
                                    <Icon icon=HASH class="mt-1 h-3.5 w-3.5 shrink-0 text-accent-400/70" />
                                    <span class="min-w-0 flex-1">
                                        <span class="block truncate text-sm font-medium">
                                            {section
                                                .number
                                                .clone()
                                                .map(|n| format!("{n} "))
                                                .unwrap_or_default()}
                                            {section.title.clone()}
                                        </span>
                                        <span class="block truncate text-[11px] uppercase tracking-[0.12em] text-white/35">
                                            {section.slug.clone()}
                                        </span>
                                    </span>
                                </button>
                            }
                        }
                    </For>
                    <Show when=move || filtered.get().is_empty()>
                        <div class="px-3 py-6 text-center text-xs text-white/30">
                            "No sections match."
                        </div>
                    </Show>
                </div>
            </div>
        </aside>
    }
}

#[component]
pub fn DocSectionCard(
    section: DocSection,
    #[prop(into)] base_path: String,
    #[prop(into, default = Signal::from(false))] active: Signal<bool>,
) -> impl IntoView {
    let (copied, copy) = use_clipboard(Duration::from_millis(1800));
    let href = format!("{base_path}#{}", section.slug);
    let render = section.render.clone();

    let copy_link = {
        let href = href.clone();
        move |_| {
            let origin = web_sys::window()
                .and_then(|w| w.location().origin().ok())
                .unwrap_or_default();
            copy(format!("{origin}{href}"));
            if let Some(history) = web_sys::window().and_then(|w| w.history().ok()) {
                let _ =
                    history.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&href));
            }
        }
    };

    view! {
        <section
            id=section.slug.clone()
            class=move || {
                cn!(
                    GLASS, "rounded-2xl p-6 md:p-8", SCROLL_MARGIN, TRANSITION, active.get()
                    .then_some("border-accent-400/30 bg-accent-500/[0.06]"),
                )
            }
        >
            <header class="mb-6 flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
                <div class="min-w-0">
                    <div class="mb-2 flex flex-wrap items-center gap-2 text-xs uppercase tracking-[0.16em] text-white/42">
                        <span class="rounded-full border border-white/10 bg-white/[0.04] px-2.5 py-1 text-white/56">
                            "Section"
                        </span>
                        {section.number.clone()}
                        <span class="text-accent-300/80">{format!("#{}", section.slug)}</span>
                    </div>
                    <h2 class="text-2xl font-semibold tracking-tight text-white md:text-3xl">
                        {section.title.clone()}
                    </h2>
                    {section
                        .summary
                        .clone()
                        .map(|s| {
                            view! {
                                <p class="mt-3 max-w-3xl text-sm leading-7 text-white/62">{s}</p>
                            }
                        })}
                </div>

                <button
                    type="button"
                    class=cn!(
                        "inline-flex shrink-0 items-center gap-2 rounded-2xl border border-white/10 bg-black/20",
                        "px-3 py-2 text-sm text-white/68",
                        TRANSITION,
                        "hover:border-accent-400/25 hover:bg-accent-500/10 hover:text-white",
                    )
                    on:click=copy_link
                >
                    {move || {
                        if copied.get() {
                            view! { <Icon icon=CHECK class="h-4 w-4 text-accent-400" /> }.into_any()
                        } else {
                            view! { <Icon icon=LINK_2 class="h-4 w-4 text-accent-400" /> }.into_any()
                        }
                    }}
                    {move || if copied.get() { "Copied!" } else { "Copy link" }}
                </button>
            </header>

            <div class="space-y-5 text-[15px] leading-7 text-white/72">{move || render()}</div>
        </section>
    }
}

#[component]
pub fn DocPage(doc: Doc, #[prop(into)] base_path: String) -> impl IntoView {
    let sections = StoredValue::new(doc.sections.clone());
    let slugs = Signal::derive(move || {
        sections.with_value(|s| s.iter().map(|x| x.slug.clone()).collect::<Vec<_>>())
    });
    let active = use_active_section(slugs);

    Effect::new(move |_| {
        let hash = web_sys::window()
            .and_then(|w| w.location().hash().ok())
            .unwrap_or_default();
        let target = hash.trim_start_matches('#').to_string();
        if target.is_empty() {
            return;
        }
        set_timeout(
            move || {
                scroll_to_id(&target, false);
                active.set(Some(target.clone()));
            },
            Duration::from_millis(80),
        );
    });

    let pick = {
        let base_path = base_path.clone();
        Callback::new(move |slug: String| {
            active.set(Some(slug.clone()));
            scroll_to_id(&slug, false);
            if let Some(history) = web_sys::window().and_then(|w| w.history().ok()) {
                let url = format!("{base_path}#{slug}");
                let _ = history.push_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&url));
            }
        })
    };

    let toc_sections = Signal::derive(move || sections.get_value());
    let header = doc.clone();

    view! {
        // No `min-h-full` here: as a flex child its `100%` resolves against the
        // parent's own content-derived height, which inflates this box and
        // pushes anything rendered after the document out of the container.
        <div class="text-white">
            <div class="relative z-10 flex flex-col gap-6 xl:flex-row">
                <DocToc
                    sections=toc_sections
                    active=Signal::derive(move || active.get())
                    on_pick=pick
                />

                // Only the article replays the page-in animation. The doc
                // sidebar and the table of contents beside it are navigation:
                // they stay where they are across a navigation.
                <AnimatedPage class="min-w-0 flex-1 space-y-6">
                    <section class=cn!(GLASS, "rounded-2xl p-6 md:p-8")>
                        <div class="mb-3 flex flex-wrap items-center gap-2 text-xs uppercase tracking-[0.18em] text-white/42">
                            <Badge tone=Tone::Accent>{header.eyebrow.clone()}</Badge>
                            <span>{header.category.clone()}</span>
                        </div>
                        <h1 class="text-3xl font-semibold tracking-tight text-white md:text-4xl">
                            {header.title.clone()}
                        </h1>
                        {header
                            .tagline
                            .clone()
                            .map(|t| {
                                view! {
                                    <p class="mt-4 max-w-3xl text-sm leading-7 text-white/62 md:text-base">
                                        {t}
                                    </p>
                                }
                            })}
                        <Show when={
                            let has_tags = !header.tags.is_empty();
                            move || has_tags
                        }>
                            <div class="mt-5 flex flex-wrap gap-2">
                                {header
                                    .tags
                                    .iter()
                                    .cloned()
                                    .map(|t| {
                                        view! {
                                            <Badge tone=Tone::Neutral size=Size::Sm>
                                                {t}
                                            </Badge>
                                        }
                                    })
                                    .collect_view()}
                            </div>
                        </Show>
                    </section>

                    <div class="flex flex-col gap-6">
                        {doc
                            .sections
                            .iter()
                            .cloned()
                            .enumerate()
                            .map(|(idx, section)| {
                                let slug = section.slug.clone();
                                let is_active = Signal::derive(move || {
                                    active.get().as_deref() == Some(slug.as_str())
                                });
                                let rule = (idx > 0)
                                    .then(|| {
                                        view! {
                                            <crate::ui::components::Divider label=section
                                                .number
                                                .clone()
                                                .unwrap_or_default() />
                                        }
                                    });
                                view! {
                                    {rule}
                                    <DocSectionCard
                                        section=section
                                        base_path=base_path.clone()
                                        active=is_active
                                    />
                                }
                            })
                            .collect_view()}
                    </div>
                </AnimatedPage>
            </div>
        </div>
    }
}
