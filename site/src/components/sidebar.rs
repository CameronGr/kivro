use leptos::prelude::*;
use leptos_router::components::A;

use crate::routes::DOCS_NAV;

/// Documentation navigation, driven by [`crate::routes::DOCS_NAV`].
#[component]
pub fn Sidebar() -> impl IntoView {
    view! {
        <aside class="sidebar">
            <nav aria-label="Documentation">
                <ul class="sidebar__list">
                    {DOCS_NAV
                        .iter()
                        .map(|doc| {
                            view! {
                                <li>
                                    <A href=doc.href() attr:class="sidebar__link">
                                        {doc.title}
                                    </A>
                                </li>
                            }
                        })
                        .collect_view()}
                </ul>
            </nav>
        </aside>
    }
}
