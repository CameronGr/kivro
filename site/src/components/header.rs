use leptos::prelude::*;
use leptos_router::components::A;

use crate::routes;

/// Top bar: wordmark and top-level navigation.
#[component]
pub fn Header() -> impl IntoView {
    view! {
        <header class="header">
            <A href=routes::HOME attr:class="header__brand">
                "kivro"
            </A>
            <nav class="header__nav" aria-label="Primary">
                <A href=routes::DOCS>"Docs"</A>
                <a href="https://github.com/CameronGr/kivro" rel="noreferrer noopener">
                    "GitHub"
                </a>
            </nav>
        </header>
    }
}
