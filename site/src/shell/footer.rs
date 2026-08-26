//! The site footer.

use crate::content;
use crate::nav;
use crate::shell::{go_to, use_go};
use crate::ui::prelude::*;
use crate::ui::style::TRANSITION;

#[component]
pub fn Footer() -> impl IntoView {
    let go = use_go();

    view! {
        <footer class="mt-16 border-t border-white/[0.08] px-4 py-10 md:px-8">
            <div class="mx-auto flex max-w-[1400px] flex-col gap-10">
                <div class="grid gap-8 sm:grid-cols-2 lg:grid-cols-4">
                    <div class="flex flex-col gap-3">
                        <div class="flex items-center gap-2.5">
                            <span class="flex h-8 w-8 items-center justify-center rounded-xl border border-accent-400/30 bg-accent-500/10">
                                <Icon icon=icons::SHIELD_ALERT class="h-4 w-4 text-accent-400" />
                            </span>
                            <span class="text-sm font-semibold tracking-tight text-white">
                                "kivro"
                            </span>
                        </div>
                        <p class="max-w-xs text-xs leading-6 text-white/45">
                            "Secrets in the OS credential store, injected into processes on "
                            "demand. The manifest is committed; the values never are."
                        </p>
                        <StatusDot
                            tone=Tone::Accent
                            label=format!("v{} · MIT", nav::VERSION)
                        />
                    </div>

                    {content::groups()
                        .into_iter()
                        .take(3)
                        .map(|(label, entries)| {
                            view! {
                                <div class="flex flex-col gap-2.5">
                                    <Eyebrow>{label}</Eyebrow>
                                    <ul class="flex flex-col gap-1.5">
                                        {entries
                                            .into_iter()
                                            .map(|entry| {
                                                let click = go_to(go, nav::doc_path(entry.slug));
                                                view! {
                                                    <li>
                                                        <button
                                                            type="button"
                                                            class=cn!(
                                                                "text-sm text-white/55 hover:text-accent-200",
                                                                TRANSITION,
                                                            )
                                                            on:click=move |_| click.run(())
                                                        >
                                                            {entry.title}
                                                        </button>
                                                    </li>
                                                }
                                            })
                                            .collect_view()}
                                    </ul>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>

                <div class="flex flex-col gap-3 border-t border-white/[0.06] pt-6 text-xs text-white/35 sm:flex-row sm:items-center sm:justify-between">
                    <span>"Documentation for the kivro crate. Written against 0.1.0."</span>
                    <div class="flex items-center gap-4">
                        <Link href=nav::REPO external=true tone=Tone::Neutral>
                            "Repository"
                        </Link>
                        <Link
                            href=nav::repo_file("docs/SECURITY.md")
                            external=true
                            tone=Tone::Neutral
                        >
                            "Security"
                        </Link>
                        <Link href="https://leptos.dev" external=true tone=Tone::Neutral>
                            "Built with Leptos"
                        </Link>
                    </div>
                </div>
            </div>
        </footer>
    }
}
