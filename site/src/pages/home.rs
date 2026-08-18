use leptos::prelude::*;
use leptos_meta::Title;

/// Landing page.
#[component]
pub fn Home() -> impl IntoView {
    view! {
        <Title text="kivro" />
        <h1>"kivro"</h1>
    }
}
