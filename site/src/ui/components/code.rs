use std::time::Duration;

use leptos::prelude::*;

use crate::cn;
use crate::ui::highlight::{Lang, highlight};
use crate::ui::hooks::use_clipboard;
use crate::ui::icons::{CHECK, COPY, Icon, TERMINAL};
use crate::ui::style::{GLASS, TRANSITION};

#[component]
pub fn CodeBlock(
    #[prop(into)] code: String,
    #[prop(into, optional)] language: Option<String>,
    #[prop(into, optional)] title: Option<String>,
    #[prop(optional)] dense: bool,
    #[prop(optional)] line_numbers: bool,
    #[prop(optional, into)] highlight_lines: Vec<usize>,
    #[prop(into, optional)] class: String,
) -> impl IntoView {
    let lang = language.as_deref().map(Lang::from_tag).unwrap_or_default();
    let source = dedent(&code);
    let lines = highlight(&source, lang);
    let header = title.unwrap_or_else(|| lang.label().to_string());
    let show_terminal = lang == Lang::Bash;

    let (copied, copy) = use_clipboard(Duration::from_millis(1500));
    let payload = source.clone();

    let pad = if dense { "px-4 py-3" } else { "px-6 py-5" };
    let text = if dense {
        "text-[0.78rem]"
    } else {
        "text-[0.85rem]"
    };

    let bleed = if dense {
        "-mx-4 border-l-2 border-accent-400/60 bg-accent-500/[0.08] px-4"
    } else {
        "-mx-6 border-l-2 border-accent-400/60 bg-accent-500/[0.08] px-6"
    };
    let gutter_width = lines.len().to_string().len().max(2);

    view! {
        <div class=cn!(GLASS, "overflow-hidden rounded-xl", class)>
            <div class="flex items-center justify-between border-b border-white/10 bg-white/[0.03] px-4 py-2.5 text-xs uppercase tracking-[0.16em] text-white/45">
                <div class="flex items-center gap-2">
                    {show_terminal
                        .then(|| view! { <Icon icon=TERMINAL class="h-4 w-4 text-accent-400" /> })}
                    <span class="truncate">{header}</span>
                </div>
                <button
                    type="button"
                    class=cn!(
                        "inline-flex items-center gap-1.5 rounded-xl border border-white/10 bg-black/20",
                        "px-2 py-1 text-[11px] normal-case tracking-normal text-white/65",
                        TRANSITION,
                        "hover:border-accent-400/25 hover:bg-accent-500/10 hover:text-white",
                    )
                    on:click=move |_| copy(payload.clone())
                >
                    {move || {
                        if copied.get() {
                            view! {
                                <Icon icon=CHECK class="h-3.5 w-3.5 text-accent-400" />
                                "Copied"
                            }
                                .into_any()
                        } else {
                            view! {
                                <Icon icon=COPY class="h-3.5 w-3.5" />
                                "Copy"
                            }
                                .into_any()
                        }
                    }}
                </button>
            </div>

            <div class="overflow-x-auto">
                <pre class=cn!(
                    "iui-code m-0 inline-block min-w-full font-mono leading-[1.55] text-[#e6edf3]",
                    pad,
                    text,
                )>
                    <code>
                        {lines
                            .into_iter()
                            .enumerate()
                            .map(|(idx, tokens)| {
                                let number = idx + 1;
                                let marked = highlight_lines.contains(&number);
                                view! {
                                    <span class=cn!("block", marked.then_some(bleed))>
                                        {line_numbers
                                            .then(|| {
                                                view! {
                                                    <span
                                                        class="mr-4 inline-block select-none text-right text-white/25"
                                                        style:width=format!("{gutter_width}ch")
                                                    >
                                                        {number}
                                                    </span>
                                                }
                                            })}
                                        {if tokens.is_empty() {
                                            view! { <span>{"\u{200b}"}</span> }.into_any()
                                        } else {
                                            tokens
                                                .into_iter()
                                                .map(|tok| {
                                                    view! {
                                                        <span
                                                            style:color=tok.kind.color()
                                                            style:font-style=tok
                                                                .kind
                                                                .italic()
                                                                .then_some("italic")
                                                        >
                                                            {tok.text}
                                                        </span>
                                                    }
                                                })
                                                .collect_view()
                                                .into_any()
                                        }}
                                    </span>
                                }
                            })
                            .collect_view()}
                    </code>
                </pre>
            </div>
        </div>
    }
}

#[component]
pub fn CommandLine(
    #[prop(into)] command: String,
    #[prop(into, default = "$".to_string())] prompt: String,
    #[prop(into, optional)] class: String,
) -> impl IntoView {
    let (copied, copy) = use_clipboard(Duration::from_millis(1500));
    let payload = command.clone();
    view! {
        <div class=cn!(
            GLASS,
            "flex items-center gap-3 rounded-xl px-4 py-3 font-mono text-sm",
            class,
        )>
            <span class="select-none text-accent-400">{prompt}</span>
            <span class="min-w-0 flex-1 truncate text-white/80">{command}</span>
            <button
                type="button"
                aria-label="Copy command"
                class=cn!("shrink-0 rounded-lg p-1 text-white/50 hover:text-white", TRANSITION)
                on:click=move |_| copy(payload.clone())
            >
                {move || {
                    if copied.get() {
                        view! { <Icon icon=CHECK class="h-4 w-4 text-accent-400" /> }.into_any()
                    } else {
                        view! { <Icon icon=COPY class="h-4 w-4" /> }.into_any()
                    }
                }}
            </button>
        </div>
    }
}

fn dedent(src: &str) -> String {
    let lines: Vec<&str> = src.trim_matches('\n').lines().collect();
    let indent = lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);

    lines
        .iter()
        .map(|l| {
            if l.len() >= indent {
                &l[indent..]
            } else {
                l.trim_start()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::dedent;

    #[test]
    fn removes_common_indent_only() {
        let src = "\n        fn main() {\n            body();\n        }\n    ";
        assert_eq!(dedent(src), "fn main() {\n    body();\n}");
    }

    #[test]
    fn blank_lines_do_not_set_the_indent() {
        let src = "\n    a\n\n    b\n";
        assert_eq!(dedent(src), "a\n\nb");
    }

    #[test]
    fn unindented_source_is_untouched() {
        assert_eq!(dedent("a\nb"), "a\nb");
    }
}
