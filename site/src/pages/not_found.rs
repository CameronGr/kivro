use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;

use crate::routes;

/// Fallback for unmatched routes.
#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <Title text="Not found | kivro" />
        <h1>"Not found"</h1>
        <A href=routes::HOME>"Back to home"</A>
    }
}
