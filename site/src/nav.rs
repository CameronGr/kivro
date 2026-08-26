//! Route paths and the small amount of site-wide identity that several
//! components need to agree on.

/// Landing page.
pub const HOME: &str = "/";
/// Documentation index.
pub const DOCS: &str = "/docs";

/// Upstream repository.
pub const REPO: &str = "https://github.com/CameronGr/kivro";
/// Released version of the crate this site documents.
pub const VERSION: &str = "0.1.0";
/// Minimum toolchain the crate builds on.
pub const RUST_VERSION: &str = "1.96";

/// Path of one documentation page.
pub fn doc_path(slug: &str) -> String {
    format!("{DOCS}/{slug}")
}

/// A repository file, linked at the pinned default branch.
pub fn repo_file(path: &str) -> String {
    format!("{REPO}/blob/master/{path}")
}
