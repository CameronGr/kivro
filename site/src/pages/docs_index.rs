use leptos::prelude::*;
use leptos_meta::Title;

/// Landing page for `/docs`.
#[component]
pub fn DocsIndex() -> impl IntoView {
    view! {
        <Title text="Docs | kivro" />
        <h1>"Docs"</h1>
    }
}
