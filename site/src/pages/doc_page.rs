use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;

use crate::pages::NotFound;
use crate::routes::doc_by_slug;

/// A single documentation page, resolved from the `:slug` route parameter.
#[component]
pub fn DocPage() -> impl IntoView {
    let params = use_params_map();
    let doc = move || {
        params
            .read()
            .get("slug")
            .and_then(|slug| doc_by_slug(&slug))
    };

    view! {
        {move || match doc() {
            Some(doc) => {
                view! {
                    <Title text=format!("{} | kivro", doc.title) />
                    <article class="doc">
                        <h1>{doc.title}</h1>
                    </article>
                }
                    .into_any()
            }
            None => view! { <NotFound /> }.into_any(),
        }}
    }
}
