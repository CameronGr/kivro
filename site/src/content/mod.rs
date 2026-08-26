//! The documentation set.
//!
//! Every page is a [`Doc`] built by one module. [`ENTRIES`] is the single
//! ordered registry: the sidebar, the docs index, the router and the previous/
//! next links are all derived from it, so adding a page means adding one row.

pub mod kit;

mod architecture;
mod bundles;
mod cli;
mod concepts;
mod install;
mod integration;
mod library;
mod manifest;
mod overview;
mod quickstart;
mod roadmap;
mod security;
mod troubleshooting;

use crate::ui::components::Doc;
use crate::ui::icons::{self, IconData};

/// One documentation page, before it is built.
pub struct Entry {
    /// URL slug, unique across the site.
    pub slug: &'static str,
    /// Title, as it appears in navigation.
    pub title: &'static str,
    /// Sidebar group this page belongs to.
    pub group: &'static str,
    /// One line for the index cards and the sidebar tooltip.
    pub blurb: &'static str,
    /// Glyph used wherever the page is listed.
    pub icon: IconData,
    /// Builds the page. Cheap: sections hold function pointers, not rendered views.
    pub build: fn() -> Doc,
}

/// Every page, in reading order. Groups must stay contiguous.
pub const ENTRIES: &[Entry] = &[
    Entry {
        slug: "overview",
        title: "Overview",
        group: "Getting started",
        blurb: "What kivro is, why it exists, and what it deliberately does not do.",
        icon: icons::BOOK_OPEN,
        build: overview::doc,
    },
    Entry {
        slug: "install",
        title: "Installation",
        group: "Getting started",
        blurb: "Build the CLI, and what each platform needs before it will work.",
        icon: icons::DOWNLOAD,
        build: install::doc,
    },
    Entry {
        slug: "quickstart",
        title: "Quick start",
        group: "Getting started",
        blurb: "Empty directory to running command, plus the .env migration and team handover.",
        icon: icons::SPARKLES,
        build: quickstart::doc,
    },
    Entry {
        slug: "concepts",
        title: "Core concepts",
        group: "Guides",
        blurb: "Scopes, resolution order, declarations, and the refusal to degrade quietly.",
        icon: icons::INFO,
        build: concepts::doc,
    },
    Entry {
        slug: "integration",
        title: "Integration",
        group: "Guides",
        blurb: "Rust, Node, Make, CI and Docker — and the five anti-patterns worth naming.",
        icon: icons::LINK_2,
        build: integration::doc,
    },
    Entry {
        slug: "troubleshooting",
        title: "Troubleshooting",
        group: "Guides",
        blurb: "Every error kind, what causes it, and what fixes it.",
        icon: icons::WRENCH,
        build: troubleshooting::doc,
    },
    Entry {
        slug: "cli",
        title: "CLI reference",
        group: "Reference",
        blurb: "Thirteen commands, five global options, nine exit codes.",
        icon: icons::TERMINAL,
        build: cli::doc,
    },
    Entry {
        slug: "manifest",
        title: "Manifest format",
        group: "Reference",
        blurb: "The .kivro.toml specification, key by key, including how it evolves.",
        icon: icons::FILE_TEXT,
        build: manifest::doc,
    },
    Entry {
        slug: "library",
        title: "Library API",
        group: "Reference",
        blurb: "Project, Environment, SecretSet, SecretString, and testing against a fake store.",
        icon: icons::HASH,
        build: library::doc,
    },
    Entry {
        slug: "architecture",
        title: "Architecture",
        group: "Design & security",
        blurb: "Seven crates, the storage model, and the reasoning behind both.",
        icon: icons::SETTINGS,
        build: architecture::doc,
    },
    Entry {
        slug: "security",
        title: "Security model",
        group: "Design & security",
        blurb: "The threat model, in and out of scope, with the limits stated plainly.",
        icon: icons::SHIELD_ALERT,
        build: security::doc,
    },
    Entry {
        slug: "bundles",
        title: "Encrypted bundles",
        group: "Design & security",
        blurb: "Sharing secrets, and the age-based file format that carries them.",
        icon: icons::USER,
        build: bundles::doc,
    },
    Entry {
        slug: "roadmap",
        title: "Status and roadmap",
        group: "Design & security",
        blurb: "What 0.1 ships, what it promises, and what is deliberately absent.",
        icon: icons::PLANE,
        build: roadmap::doc,
    },
];

/// The registry entry for a slug.
pub fn entry(slug: &str) -> Option<&'static Entry> {
    ENTRIES.iter().find(|e| e.slug == slug)
}

/// Sidebar groups, in registry order, each with its entries.
pub fn groups() -> Vec<(&'static str, Vec<&'static Entry>)> {
    let mut groups: Vec<(&'static str, Vec<&'static Entry>)> = Vec::new();
    for entry in ENTRIES {
        match groups.last_mut() {
            Some((label, items)) if *label == entry.group => items.push(entry),
            _ => groups.push((entry.group, vec![entry])),
        }
    }
    groups
}

/// The pages either side of `slug`, for the previous/next footer.
pub fn neighbours(slug: &str) -> (Option<&'static Entry>, Option<&'static Entry>) {
    let Some(index) = ENTRIES.iter().position(|e| e.slug == slug) else {
        return (None, None);
    };
    let previous = index.checked_sub(1).and_then(|i| ENTRIES.get(i));
    let next = ENTRIES.get(index + 1);
    (previous, next)
}
