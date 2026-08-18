use leptos::prelude::*;
use leptos_meta::{Meta, Title, provide_meta_context};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use crate::components::Layout;
use crate::pages::{DocPage, DocsIndex, Home, NotFound};

/// Root component: metadata context, router, and the shared page chrome.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text="kivro" />
        <Meta name="description" content="Documentation for kivro." />

        <Router>
            <Layout>
                <Routes fallback=NotFound>
                    <Route path=path!("/") view=Home />
                    <Route path=path!("/docs") view=DocsIndex />
                    <Route path=path!("/docs/:slug") view=DocPage />
                </Routes>
            </Layout>
        </Router>
    }
}
