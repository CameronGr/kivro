use crate::ui::prelude::*;

/// Assemble the whole gallery as a single document.
pub fn gallery_doc() -> Doc {
    Doc::new(
        "gallery",
        "Component gallery",
        "Library · v0.1.0",
        "infinity-ui",
    )
    .tagline(
        "Every component in infinity-ui, rendered live. The page you are reading is itself \
             built from DocPage, DocToc, and DocSectionCard — the documentation shell is not a \
             demo, it is the same code.",
    )
    .tags([
        "Leptos 0.8",
        "Tailwind v3",
        "CSR / SSR / hydrate",
        "No JS deps",
    ])
    .section(
        DocSection::new("foundations", "Foundations", foundations)
            .numbered("1.0")
            .summary("Tones, sizes, and the surfaces everything else is built on."),
    )
    .section(
        DocSection::new("typography", "Typography", typography)
            .numbered("1.1")
            .summary("Headings, prose, lists, and inline chrome."),
    )
    .section(
        DocSection::new("buttons", "Buttons", buttons)
            .numbered("2.0")
            .summary("Five variants across six tones, plus icon buttons and loading state."),
    )
    .section(
        DocSection::new("feedback", "Feedback", feedback)
            .numbered("2.1")
            .summary("Callouts, badges, alerts, progress, and skeletons."),
    )
    .section(
        DocSection::new("surfaces", "Surfaces", surfaces)
            .numbered("2.2")
            .summary("Cards, tiles, stats, and empty states."),
    )
    .section(
        DocSection::new("forms", "Form controls", forms)
            .numbered("3.0")
            .summary("Text entry, selection, and toggles — all controlled."),
    )
    .section(
        DocSection::new("overlays", "Overlays", overlays)
            .numbered("3.1")
            .summary("Modals, drawers, popovers, tooltips, and toasts."),
    )
    .section(
        DocSection::new("navigation", "Navigation", navigation)
            .numbered("4.0")
            .summary("Tabs, accordions, breadcrumbs, and pagination."),
    )
    .section(
        DocSection::new("data", "Data display", data)
            .numbered("4.1")
            .summary("Tables and code blocks."),
    )
    .section(
        DocSection::new("icons", "Icon set", icon_sheet)
            .numbered("5.0")
            .summary("Inline SVG glyphs — no icon font, no JS package."),
    )
}

const TONES: [(Tone, &str); 6] = [
    (Tone::Accent, "Accent"),
    (Tone::Neutral, "Neutral"),
    (Tone::Info, "Info"),
    (Tone::Warning, "Warning"),
    (Tone::Danger, "Danger"),
    (Tone::Dev, "Dev"),
];

/// Labelled demo row.
#[component]
fn Demo(#[prop(into)] label: String, children: Children) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-2.5">
            <Eyebrow>{label}</Eyebrow>
            <div class="flex flex-wrap items-center gap-3">{children()}</div>
        </div>
    }
}

fn foundations() -> impl IntoView {
    view! {
        <Lead>
            "Two enums carry the whole system. " <InlineCode>"Tone"</InlineCode>
            " decides colour, " <InlineCode>"Size"</InlineCode>
            " decides density, and every component takes both. That is why a badge, a callout, and a button set to "
            <InlineCode>"Tone::Warning"</InlineCode> " look like the same warning."
        </Lead>

        <Demo label="Tones">
            {TONES
                .iter()
                .map(|(tone, name)| {
                    view! {
                        <Badge tone=*tone>{*name}</Badge>
                    }
                })
                .collect_view()}
        </Demo>

        <Demo label="Surfaces">
            <div class="grid w-full gap-3 md:grid-cols-3">
                <div class=glass("rounded-2xl p-4 text-sm text-white/70")>
                    <div class="mb-1 font-medium text-white">"GLASS"</div>
                    "Hairline border over a 4% white wash. The default panel."
                </div>
                <div class=sunken("rounded-2xl p-4 text-sm text-white/70")>
                    <div class="mb-1 font-medium text-white">"SUNKEN"</div>
                    "Carved into a panel. List rows, inputs, code."
                </div>
                <div class="rounded-2xl border border-accent-400/30 bg-accent-500/10 p-4 text-sm text-accent-200">
                    <div class="mb-1 font-medium">"Tone::soft()"</div>
                    "Tinted surface, generated from the tone."
                </div>
            </div>
        </Demo>

        <Callout tone=Tone::Info title="Theming">
            "Every accent colour resolves through " <InlineCode>"rgb(var(--ac-N) / <alpha>)"</InlineCode>
            ". Override the seven variables in your own CSS and the library reskins without a rebuild."
        </Callout>

        <CodeBlock
            language="css"
            title="retheme to violet"
            code=r#"
                :root {
                  --ac-200: 221 214 254;
                  --ac-300: 196 181 253;
                  --ac-400: 167 139 250;
                  --ac-500: 139  92 246;
                  --ac-600: 124  58 237;
                  --ac-700: 109  40 217;
                  --ac-900:  46  16 101;
                }
            "#
        />
    }
}

fn typography() -> impl IntoView {
    view! {
        <H3>"Headings and prose"</H3>
        <Lead>"Lead copy opens a section — brighter and a step larger than body text."</Lead>
        <P>
            "Body copy sits at " <InlineCode>"text-white/70"</InlineCode>
            ". Inline code is accent-tinted so identifiers separate from prose without bolding, and "
            <Link href="https://leptos.dev" external=true>"external links"</Link>
            " carry their own arrow."
        </P>

        <Demo label="List">
            <List>
                <ListItem>
                    <strong class="text-white/85">"Each item is its own inset row."</strong>
                    " Docs list items run to a sentence or two; bullets read badly at that length."
                </ListItem>
                <ListItem>"Ordered lists use the same rows, via " <InlineCode>"ordered=true"</InlineCode> "."</ListItem>
            </List>
        </Demo>

        <Demo label="Keyboard">
            <span class="text-sm text-white/60">
                "Press " <Kbd>"⌘"</Kbd> " " <Kbd>"K"</Kbd> " to search, " <Kbd>"Esc"</Kbd> " to dismiss."
            </span>
        </Demo>

        <Divider label="1.1" />
        <Muted>"Divider takes an optional label pill — the docs use section numbers."</Muted>
    }
}

fn buttons() -> impl IntoView {
    let loading = RwSignal::new(false);
    let count = RwSignal::new(0);

    view! {
        <Demo label="Variants">
            <Button variant=Variant::Solid>"Solid"</Button>
            <Button variant=Variant::Soft>"Soft"</Button>
            <Button variant=Variant::Glass>"Glass"</Button>
            <Button variant=Variant::Ghost>"Ghost"</Button>
            <Button variant=Variant::Link>"Link"</Button>
        </Demo>

        <Demo label="Tones">
            {TONES
                .iter()
                .map(|(tone, name)| {
                    view! {
                        <Button tone=*tone variant=Variant::Soft>
                            {*name}
                        </Button>
                    }
                })
                .collect_view()}
        </Demo>

        <Demo label="Sizes">
            <Button size=Size::Xs>"Extra small"</Button>
            <Button size=Size::Sm>"Small"</Button>
            <Button size=Size::Md>"Medium"</Button>
            <Button size=Size::Lg>"Large"</Button>
        </Demo>

        <Demo label="With icons">
            <Button icon=icons::DOWNLOAD>"Download"</Button>
            <Button trailing_icon=icons::ARROW_RIGHT variant=Variant::Soft>"Read the docs"</Button>
            <Button
                icon=icons::TRASH_2
                tone=Tone::Danger
                variant=Variant::Soft
            >
                "Delete"
            </Button>
        </Demo>

        <Demo label="State">
            <Button
                loading=loading
                on_click=Callback::new(move |_| loading.update(|l| *l = !*l))
            >
                {move || if loading.get() { "Working…" } else { "Toggle loading" }}
            </Button>
            <Button disabled=true>"Disabled"</Button>
            <Button
                variant=Variant::Soft
                on_click=Callback::new(move |_| count.update(|c| *c += 1))
            >
                {move || format!("Clicked {} times", count.get())}
            </Button>
        </Demo>

        <Demo label="Icon buttons">
            <IconButton icon=icons::SETTINGS label="Settings" />
            <IconButton icon=icons::BELL label="Notifications" variant=Variant::Glass />
            <IconButton icon=icons::TRASH_2 label="Delete" tone=Tone::Danger variant=Variant::Soft />
            <IconButton icon=icons::PLUS label="Add" size=Size::Sm variant=Variant::Ghost />
        </Demo>

        <CodeBlock
            language="rust"
            title="button.rs"
            code=r#"
                view! {
                    <Button
                        variant=Variant::Soft
                        tone=Tone::Danger
                        icon=icons::TRASH_2
                        on_click=Callback::new(move |_| remove(id))
                    >
                        "Delete"
                    </Button>
                }
            "#
        />
    }
}

fn feedback() -> impl IntoView {
    let progress = RwSignal::new(0.62_f64);
    let dismissed = RwSignal::new(false);

    view! {
        <Demo label="Callouts">
            <div class="grid w-full gap-3">
                <Callout tone=Tone::Accent>
                    "The default note. Accent-tinted, for anything affirmative."
                </Callout>
                <Callout tone=Tone::Warning>
                    "Caution. Something needs care but nothing is broken yet."
                </Callout>
                <Callout tone=Tone::Danger title="ChartImage cleanup">
                    "Warning. The SDK exposes no dedicated free for chart-page images."
                </Callout>
            </div>
        </Demo>

        <Demo label="Badges">
            {TONES
                .iter()
                .map(|(tone, name)| view! { <Badge tone=*tone size=Size::Sm>{*name}</Badge> })
                .collect_view()}
            <Badge tone=Tone::Dev icon=icons::FLASK_CONICAL>"Dev"</Badge>
            <Badge squared=true tone=Tone::Neutral>"Squared"</Badge>
        </Demo>

        <Demo label="Alerts">
            <Show when=move || !dismissed.get() fallback=|| view! {
                <Muted>"Dismissed. Reload to bring it back."</Muted>
            }>
                <Alert
                    tone=Tone::Warning
                    title="Toolchain out of date"
                    description="infinity-rs requires Rust 1.80 or newer for the wasm32 target."
                    on_dismiss=Callback::new(move |_| dismissed.set(true))
                    class="w-full"
                />
            </Show>
        </Demo>

        <Demo label="Progress">
            <div class="w-full max-w-md space-y-3">
                <Progress value=progress show_value=true label="Build progress" />
                <Slider
                    value=progress
                    on_input=Callback::new(move |v: f64| progress.set(v))
                    min=0.0
                    max=1.0
                    step=0.01
                />
            </div>
        </Demo>

        <Demo label="Loading">
            <Spinner />
            <Spinner size=Size::Lg tone=Tone::Neutral />
            <div class="w-64 space-y-2">
                <Skeleton class="h-3 w-1/3" />
                <Skeleton class="h-3 w-full" />
                <Skeleton class="h-3 w-2/3" />
            </div>
            <Skeleton circle=true class="h-10 w-10" />
        </Demo>

        <Demo label="Identity">
            <Avatar name="Infinity MSFS" />
            <Avatar name="Ada Lovelace" size=Size::Lg />
            <StatusDot tone=Tone::Accent pulse=true label="Building" />
            <StatusDot tone=Tone::Danger label="Failed" />
        </Demo>
    }
}

fn surfaces() -> impl IntoView {
    view! {
        <Demo label="Cards">
            <div class="grid w-full gap-4 md:grid-cols-2">
                <Card
                    title="Gauges & systems"
                    eyebrow="Module"
                    icon=icons::WRENCH
                >
                    "Register a WASM gauge, wire its draw callback, and let the host own the frame loop."
                </Card>
                <Card
                    title="SimConnect"
                    eyebrow="Module"
                    icon=icons::PLANE
                    tone=Tone::Info
                    actions=ViewFn::from(|| view! { <Badge size=Size::Xs tone=Tone::Info>"Beta"</Badge> })
                >
                    "Typed client with RAII request handles. Dropping a subscription unsubscribes."
                </Card>
            </div>
        </Demo>

        <Demo label="Stats">
            <div class="grid w-full gap-3 sm:grid-cols-3">
                <Stat label="Bundle" value="308 kB" detail="gzipped, this page" icon=icons::DOWNLOAD />
                <Stat label="Components" value="63" detail="across 11 modules" icon=icons::SPARKLES />
                <Stat
                    label="JS dependencies"
                    value="0"
                    detail="highlighter included"
                    icon=icons::CIRCLE_CHECK
                />
            </div>
        </Demo>

        <Demo label="Tiles">
            <div class="grid w-full gap-4 lg:grid-cols-2">
                <Tile
                    title="Developer Docs"
                    eyebrow="Available now"
                    tagline="The Rust toolkit for building MSFS 2024 WASM gauges, systems, and SimConnect clients."
                    icon=icons::WRENCH
                    bullets=vec![
                        "Ship your first gauge in five minutes".to_string(),
                        "Per-module deep dives".to_string(),
                    ]
                    cta="Read the docs"
                    href="#navigation"
                />
                <Tile
                    title="Aircraft Documents"
                    eyebrow="Coming soon"
                    tagline="ATA-organised technical manuals — system descriptions, schematics, and procedures."
                    icon=icons::PLANE
                    bullets=vec![
                        "Browse by ATA chapter".to_string(),
                        "Normal and abnormal procedures".to_string(),
                    ]
                    cta="Available soon"
                    disabled=true
                    flag=("Soon".to_string(), Tone::Warning)
                />
            </div>
        </Demo>

        <Demo label="Empty state">
            <EmptyState
                title="No chapters imported yet"
                description="Documents appear here once the ATA import finishes."
                icon=icons::FILE_TEXT
                class="w-full"
            >
                <Button size=Size::Sm variant=Variant::Soft icon=icons::PLUS>
                    "Import a chapter"
                </Button>
            </EmptyState>
        </Demo>
    }
}

fn forms() -> impl IntoView {
    let name = RwSignal::new(String::new());
    let notes = RwSignal::new(String::new());
    let search = RwSignal::new(String::new());
    let notify = RwSignal::new(true);
    let telemetry = RwSignal::new(false);
    let agreed = RwSignal::new(false);
    let channel = RwSignal::new("stable".to_string());
    let target = RwSignal::new("wasm32".to_string());

    let targets = Signal::derive(|| {
        vec![
            SelectGroup::new(
                "WASM",
                vec![
                    SelectOption::new("wasm32", "wasm32-unknown-unknown"),
                    SelectOption::new("wasi", "wasm32-wasip1"),
                ],
            ),
            SelectGroup::new(
                "Native",
                vec![
                    SelectOption::new("x86", "x86_64-pc-windows-msvc"),
                    SelectOption::new("arm", "aarch64-apple-darwin").disabled(),
                ],
            ),
        ]
    });

    let name_error = Signal::derive(move || {
        let v = name.get();
        (!v.is_empty() && v.len() < 3)
            .then(|| "Project names need at least 3 characters.".to_string())
    });

    view! {
        <div class="grid gap-6 md:grid-cols-2">
            <div class="space-y-4">
                <Field
                    label="Project name"
                    hint="Used as the crate name."
                    required=true
                    control_id="project-name"
                    error=name_error
                >
                    <TextInput
                        value=name
                        on_input=Callback::new(move |v| name.set(v))
                        placeholder="my-gauge"
                        id="project-name"
                        invalid=Signal::derive(move || name_error.get().is_some())
                    />
                </Field>

                <Field label="Build target" control_id="target">
                    <Select
                        value=target
                        on_change=Callback::new(move |v| target.set(v))
                        groups=targets
                    />
                </Field>

                <Field label="Notes" hint="Markdown is fine.">
                    <TextArea
                        value=notes
                        on_input=Callback::new(move |v| notes.set(v))
                        placeholder="What does this gauge do?"
                        rows=3
                    />
                </Field>

                <Field label="Search">
                    <SearchInput
                        value=search
                        on_input=Callback::new(move |v| search.set(v))
                        placeholder="Filter modules…"
                    />
                </Field>
            </div>

            <div class="space-y-5">
                <Demo label="Switches">
                    <div class="w-full space-y-3">
                        <Switch
                            checked=notify
                            on_change=Callback::new(move |v| notify.set(v))
                            label="Build notifications"
                            description="Ping when a WASM build finishes."
                        />
                        <Switch
                            checked=telemetry
                            on_change=Callback::new(move |v| telemetry.set(v))
                            label="Anonymous telemetry"
                            tone=Tone::Neutral
                        />
                        <Switch
                            checked=Signal::from(false)
                            on_change=Callback::new(|_| ())
                            label="Locked setting"
                            disabled=true
                        />
                    </div>
                </Demo>

                <Demo label="Checkbox & radio">
                    <div class="w-full space-y-3">
                        <Checkbox
                            checked=agreed
                            on_change=Callback::new(move |v| agreed.set(v))
                            label="I have read the SDK license"
                        />
                        <div class="flex flex-col gap-2">
                            {["stable", "beta", "nightly"]
                                .into_iter()
                                .map(|c| {
                                    view! {
                                        <Radio
                                            name="channel"
                                            value=c
                                            selected=channel
                                            on_change=Callback::new(move |v| channel.set(v))
                                            label=c
                                        />
                                    }
                                })
                                .collect_view()}
                        </div>
                    </div>
                </Demo>

                <Well>
                    <div class="px-4 py-3 text-xs uppercase tracking-[0.16em] text-white/40">
                        "Current state"
                    </div>
                    <div class="px-4 py-3 font-mono text-xs text-white/60">
                        {move || {
                            format!(
                                "name={:?} target={} channel={} notify={} agreed={}",
                                name.get(),
                                target.get(),
                                channel.get(),
                                notify.get(),
                                agreed.get(),
                            )
                        }}
                    </div>
                </Well>
            </div>
        </div>
    }
}

fn overlays() -> impl IntoView {
    let modal = RwSignal::new(false);
    let confirm = RwSignal::new(false);
    let drawer = RwSignal::new(false);
    let toasts = RwSignal::new(Vec::<Toast>::new());
    let next_id = RwSignal::new(0_u64);

    let push_toast = move |tone: Tone, title: &'static str| {
        let id = next_id.get_untracked();
        next_id.set(id + 1);
        toasts.update(|t| {
            t.push(
                Toast::new(id, title)
                    .with_tone(tone)
                    .with_description("Dismiss me, or push another."),
            )
        });
    };

    view! {
        <Demo label="Modal & confirm">
            <Button variant=Variant::Soft on_click=Callback::new(move |_| modal.set(true))>
                "Open modal"
            </Button>
            <Button
                variant=Variant::Soft
                tone=Tone::Danger
                on_click=Callback::new(move |_| confirm.set(true))
            >
                "Delete something"
            </Button>
        </Demo>

        <Demo label="Drawer">
            <Button variant=Variant::Glass on_click=Callback::new(move |_| drawer.set(true))>
                "Open drawer"
            </Button>
        </Demo>

        <Demo label="Popover & tooltip">
            <Popover
                trigger=ViewFn::from(|| {
                    view! { <Button variant=Variant::Glass icon=icons::MENU>"Menu"</Button> }
                })
            >
                <MenuItem icon=icons::PENCIL>"Rename"</MenuItem>
                <MenuItem icon=icons::DOWNLOAD>"Export"</MenuItem>
                <MenuSeparator />
                <MenuItem icon=icons::TRASH_2 tone=Tone::Danger>"Delete"</MenuItem>
            </Popover>

            <Tooltip text="Copies an absolute deep link">
                <Button variant=Variant::Ghost icon=icons::LINK_2>"Hover me"</Button>
            </Tooltip>

            <Tooltip text="Placed below" placement=Placement::Bottom>
                <IconButton icon=icons::INFO label="Info" variant=Variant::Ghost />
            </Tooltip>
        </Demo>

        <Demo label="Toasts">
            <Button
                size=Size::Sm
                variant=Variant::Soft
                on_click=Callback::new(move |_| push_toast(Tone::Accent, "Build succeeded"))
            >
                "Success"
            </Button>
            <Button
                size=Size::Sm
                variant=Variant::Soft
                tone=Tone::Warning
                on_click=Callback::new(move |_| push_toast(Tone::Warning, "Slow frame detected"))
            >
                "Warning"
            </Button>
            <Button
                size=Size::Sm
                variant=Variant::Soft
                tone=Tone::Danger
                on_click=Callback::new(move |_| push_toast(Tone::Danger, "Link step failed"))
            >
                "Error"
            </Button>
        </Demo>

        <Modal
            open=modal
            on_close=Callback::new(move |_| modal.set(false))
            title="Create a gauge"
            description="Scaffolds a crate wired for the wasm32 target."
            icon=icons::SPARKLES
            footer=ViewFn::from(move || {
                view! {
                    <Button variant=Variant::Ghost on_click=Callback::new(move |_| modal.set(false))>
                        "Cancel"
                    </Button>
                    <Button variant=Variant::Soft on_click=Callback::new(move |_| modal.set(false))>
                        "Create"
                    </Button>
                }
            })
        >
            <P>
                "Escape, the backdrop, and the × all close this. Body scroll is locked while it is open."
            </P>
            <CommandLine command="cargo generate infinity-msfs/gauge-template" class="mt-4" />
        </Modal>

        <ConfirmDialog
            open=confirm
            on_close=Callback::new(move |_| confirm.set(false))
            on_confirm=Callback::new(move |_| confirm.set(false))
            title="Delete this chapter?"
            message="Removing an ATA chapter also removes its figures. This cannot be undone."
            confirm_label="Delete chapter"
        />

        <Drawer
            open=drawer
            on_close=Callback::new(move |_| drawer.set(false))
            title="Build settings"
            side=Side::Right
        >
            <div class="space-y-3">
                <P>"Drawers slide from the left, right, or bottom."</P>
                <List>
                    <ListItem>"Same dismissal contract as the modal."</ListItem>
                    <ListItem>"Scroll lock restores whatever overflow the page had."</ListItem>
                </List>
            </div>
        </Drawer>

        <ToastHost
            toasts=toasts
            on_dismiss=Callback::new(move |id: u64| {
                toasts.update(|t| t.retain(|x| x.id != id))
            })
        />
    }
}

fn navigation() -> impl IntoView {
    let tab = RwSignal::new("overview".to_string());
    let page = RwSignal::new(1_usize);

    let tabs = Signal::derive(|| {
        vec![
            TabItem::new("overview", "Overview").with_icon(icons::BOOK_OPEN),
            TabItem::new("api", "API").with_badge("42"),
            TabItem::new("changelog", "Changelog"),
            TabItem::new("archived", "Archived").disabled(),
        ]
    });

    view! {
        <Demo label="Breadcrumbs">
            <Breadcrumbs items=vec![
                Crumb::link("Docs", "#"),
                Crumb::link("Modules", "#"),
                Crumb::new("Charts"),
            ] />
        </Demo>

        <Demo label="Tabs">
            <div class="w-full space-y-4">
                <Tabs value=tab on_change=Callback::new(move |v| tab.set(v)) items=tabs />
                <div class=glass("rounded-2xl p-5 text-sm text-white/65")>
                    {move || {
                        match tab.get().as_str() {
                            "api" => "The API panel. Tabs render only the strip — you decide what the body is.",
                            "changelog" => "The changelog panel.",
                            _ => "The overview panel. Switching tabs swaps this text.",
                        }
                    }}
                </div>
            </div>
        </Demo>

        <Demo label="Accordions">
            <div class="w-full space-y-2">
                <Accordion
                    title="How does scroll-spy work?"
                    summary="IntersectionObserver, upper third of the viewport"
                    icon=icons::INFO
                    open_by_default=true
                >
                    <P>
                        "The observer's root margin shrinks the band to the upper third, so the active section is the one you are reading — not the one peeking in from the bottom."
                    </P>
                </Accordion>
                <Accordion title="Can I nest them?" icon=icons::HASH>
                    "Yes. Each accordion owns its own open state."
                </Accordion>
            </div>
        </Demo>

        <Demo label="Pagination">
            <Pagination page=page total=Signal::from(20) on_change=Callback::new(move |p| page.set(p)) />
        </Demo>

        <Demo label="Nav list">
            <div class="w-full max-w-xs">
                <NavList
                    items=Signal::derive(|| {
                        vec![
                            NavItem::new("vars", "Vars").with_icon(icons::HASH),
                            NavItem::new("commbus", "Comm bus").with_icon(icons::LINK_2),
                            NavItem::new("charts", "Charts").with_icon(icons::FILE_TEXT),
                        ]
                    })
                    active="commbus".to_string()
                />
            </div>
        </Demo>
    }
}

fn data() -> impl IntoView {
    view! {
        <Demo label="Table">
            <Table
                columns=vec![
                    Column::new("Field"),
                    Column::new("Type").width("w-48"),
                    Column::new("Bytes").aligned(Align::End).width("w-24"),
                ]
                caption="FsIcao layout"
            >
                <Row>
                    <Cell index=0 header=true><InlineCode>"type"</InlineCode></Cell>
                    <Cell index=1><InlineCode>"char"</InlineCode></Cell>
                    <Cell index=2>"1"</Cell>
                </Row>
                <Row>
                    <Cell index=0 header=true><InlineCode>"region"</InlineCode></Cell>
                    <Cell index=1><InlineCode>"[u8; 3]"</InlineCode></Cell>
                    <Cell index=2>"3"</Cell>
                </Row>
                <Row highlighted=true>
                    <Cell index=0 header=true><InlineCode>"airport"</InlineCode></Cell>
                    <Cell index=1><InlineCode>"[u8; 9]"</InlineCode></Cell>
                    <Cell index=2>"9"</Cell>
                </Row>
            </Table>
        </Demo>

        <Demo label="Code blocks">
            <div class="w-full space-y-4">
                <CodeBlock
                    language="rust"
                    title="charts.rs"
                    line_numbers=true
                    highlight_lines=vec![6]
                    code=r#"
                        use infinity_rs::charts::{self, ChartIndex};

                        /// Look up every chart the provider has for an airport.
                        pub fn load(icao: &FsIcao) -> Result<(), ChartError> {
                            let index: ChartIndex = charts::get_index(icao)?;
                            for category in index.categories() {
                                println!("category: {}", category.name());
                            }
                            Ok(())
                        }
                    "#
                />
                <CodeBlock
                    language="bash"
                    dense=true
                    code=r#"
                        # Add the wasm target and build
                        rustup target add wasm32-unknown-unknown
                        cargo build --release --target wasm32-unknown-unknown
                    "#
                />
                <CodeBlock
                    language="toml"
                    title="Cargo.toml"
                    code=r#"
                        [dependencies]
                        infinity-ui = { version = "0.1", features = ["csr"] }
                        leptos = { version = "0.8", features = ["csr"] }
                    "#
                />
                <CommandLine command="trunk serve --open" />
            </div>
        </Demo>
    }
}

fn icon_sheet() -> impl IntoView {
    view! {
        <P>
            {format!("{} glyphs, each a `&'static str` of SVG markup stamped into a shared shell. ", icons::ALL.len())}
            "Colour comes from " <InlineCode>"currentColor"</InlineCode>
            ", size from a class — so an icon inside a toned button needs no props at all."
        </P>

        <div class="grid grid-cols-3 gap-2 sm:grid-cols-4 lg:grid-cols-6">
            {icons::ALL
                .iter()
                .map(|(name, icon)| {
                    view! {
                        <div class=sunken(
                            "flex flex-col items-center gap-2 rounded-2xl px-2 py-4 text-center",
                        )>
                            <Icon icon=*icon class="h-5 w-5 text-accent-400" />
                            <span class="w-full truncate text-[10px] uppercase tracking-[0.1em] text-white/40">
                                {*name}
                            </span>
                        </div>
                    }
                })
                .collect_view()}
        </div>
    }
}
