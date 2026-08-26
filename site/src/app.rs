//! The application shell: metadata, the route table, and the chrome around it.

use leptos_meta::{Meta, Title, provide_meta_context};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use crate::pages::{DocRoute, DocsIndex, Home, NotFound};
use crate::shell::{Footer, Header};
use crate::ui::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text="kivro — secrets in the OS keyring, not in .env" />
        <Meta
            name="description"
            content="kivro keeps the list of secrets a project needs in a committed manifest and \
                     the values in the OS credential store, injecting them straight into the \
                     process that needs them."
        />

        <Router>
            <div class="flex min-h-screen flex-col bg-black text-white">
                <Header />
                // `grow` rather than `flex-1`: the basis stays `auto`, so this
                // column is never sized from leftover viewport height.
                <main class="grow">
                    <Routes fallback=|| view! { <NotFound /> }>
                        <Route path=path!("/") view=HomeRoute />
                        <Route path=path!("/docs") view=DocsIndexRoute />
                        <Route path=path!("/docs/:slug") view=DocRoute />
                    </Routes>
                </main>
                <Footer />
            </div>
        </Router>
    }
}

#[component]
fn HomeRoute() -> impl IntoView {
    view! {
        <Title text="kivro — secrets in the OS keyring, not in .env" />
        <AnimatedPage>
            <Home />
        </AnimatedPage>
    }
}

#[component]
fn DocsIndexRoute() -> impl IntoView {
    view! {
        <Title text="Documentation · kivro" />
        <AnimatedPage>
            <DocsIndex />
        </AnimatedPage>
    }
}
