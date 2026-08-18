//! Single source of truth for the site's URL structure.
//!
//! Paths live here so navigation, the sidebar and the router cannot drift
//! apart. Add a page by adding a `Doc` entry and a matching route in
//! [`crate::app::App`].

/// A page in the documentation, as listed in the sidebar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Doc {
    /// URL slug, appended to [`DOCS`].
    pub slug: &'static str,
    /// Label shown in navigation.
    pub title: &'static str,
}

impl Doc {
    /// Absolute path to this page.
    pub fn href(&self) -> String {
        format!("{DOCS}/{}", self.slug)
    }
}

pub const HOME: &str = "/";
pub const DOCS: &str = "/docs";

/// Sidebar order, top to bottom.
pub const DOCS_NAV: &[Doc] = &[
    Doc {
        slug: "getting-started",
        title: "Getting started",
    },
    Doc {
        slug: "cli",
        title: "CLI",
    },
    Doc {
        slug: "manifest",
        title: "Manifest",
    },
    Doc {
        slug: "bundle",
        title: "Bundle",
    },
    Doc {
        slug: "integration",
        title: "Integration",
    },
    Doc {
        slug: "architecture",
        title: "Architecture",
    },
    Doc {
        slug: "security",
        title: "Security",
    },
];

/// Look up a page by its slug.
pub fn doc_by_slug(slug: &str) -> Option<&'static Doc> {
    DOCS_NAV.iter().find(|doc| doc.slug == slug)
}
