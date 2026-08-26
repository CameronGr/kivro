//! Site chrome: the header, the documentation navigation, and the footer.

mod footer;
mod header;
mod sidebar;

pub use footer::Footer;
pub use header::Header;
pub use sidebar::{DocNav, PrevNext, Sidebar};

use leptos::prelude::*;

/// A callback that routes to a path without reloading the wasm bundle.
///
/// Every internal navigation goes through this, so `<a href>` is reserved for
/// links that genuinely leave the site.
pub fn use_go() -> Callback<String> {
    let navigate = leptos_router::hooks::use_navigate();
    Callback::new(move |to: String| navigate(&to, Default::default()))
}

/// Bind a router callback to one fixed destination.
pub fn go_to(go: Callback<String>, to: impl Into<String>) -> Callback<()> {
    let to = to.into();
    Callback::new(move |()| go.run(to.clone()))
}

/// The documentation slug in a path such as `/docs/cli`, if there is one.
pub fn slug_of(path: &str) -> Option<String> {
    path.trim_end_matches('/')
        .strip_prefix("/docs/")
        .filter(|rest| !rest.is_empty() && !rest.contains('/'))
        .map(str::to_string)
}
