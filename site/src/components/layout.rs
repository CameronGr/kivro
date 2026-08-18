use leptos::prelude::*;

use crate::components::{Footer, Header, Sidebar};

/// Page chrome wrapped around every route: header, sidebar, content, footer.
#[component]
pub fn Layout(children: Children) -> impl IntoView {
    view! {
        <div class="layout">
            <Header />
            <div class="layout__body">
                <Sidebar />
                <main class="content" id="content">
                    {children()}
                </main>
            </div>
            <Footer />
        </div>
    }
}
