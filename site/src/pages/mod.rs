//! One module per route.

mod doc;
mod docs_index;
mod home;
mod not_found;

pub use doc::DocRoute;
pub use docs_index::DocsIndex;
pub use home::Home;
pub use not_found::NotFound;
