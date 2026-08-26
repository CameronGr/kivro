pub trait ClassPart {
    fn write_class(self, out: &mut String);
}

fn push(out: &mut String, raw: &str) {
    let raw = raw.trim();
    if raw.is_empty() {
        return;
    }
    if !out.is_empty() {
        out.push(' ');
    }
    out.push_str(raw);
}

impl ClassPart for &str {
    fn write_class(self, out: &mut String) {
        push(out, self);
    }
}

impl ClassPart for String {
    fn write_class(self, out: &mut String) {
        push(out, &self);
    }
}

impl ClassPart for &String {
    fn write_class(self, out: &mut String) {
        push(out, self);
    }
}

impl<T: ClassPart> ClassPart for Option<T> {
    fn write_class(self, out: &mut String) {
        if let Some(inner) = self {
            inner.write_class(out);
        }
    }
}

#[macro_export]
macro_rules! cn {
    ($($part:expr),* $(,)?) => {{
        let mut __cn = ::std::string::String::new();
        $( $crate::ui::style::ClassPart::write_class($part, &mut __cn); )*
        __cn
    }};
}

/// The signature frosted panel: hairline white border over a 4% white wash.
pub const GLASS: &str = "border border-white/10 bg-white/[0.04] backdrop-blur-sm";

/// A heavier frosted panel for things that float above the page (menus, modals).
pub const GLASS_RAISED: &str =
    "border border-white/10 bg-black/80 backdrop-blur-xl shadow-lg shadow-black/40";

/// Inset surface — used for list rows, inputs, and anything that should read as
/// carved *into* a glass card rather than sitting on top of it.
pub const SUNKEN: &str = "border border-white/10 bg-black/20";

/// Hover treatment shared by every interactive glass surface.
pub const HOVER_ACCENT: &str = "hover:border-accent-400/25 hover:bg-accent-500/10 hover:text-white";

/// Keyboard focus ring. Applied on `focus-visible` only so mouse users never
/// see it.
pub const FOCUS_RING: &str = "outline-none focus-visible:ring-2 focus-visible:ring-accent-400/40 focus-visible:ring-offset-0";

/// Standard transition for hover/active state changes.
pub const TRANSITION: &str = "transition duration-150";

// ---------------------------------------------------------------------------
// Type
// ---------------------------------------------------------------------------

/// Small all-caps label above a heading.
pub const EYEBROW: &str = "text-xs uppercase tracking-[0.16em] text-white/42";

/// Wide-tracked all-caps used inside pills and section chrome.
pub const OVERLINE: &str = "text-[11px] font-semibold uppercase tracking-[0.18em]";

/// Body copy default.
pub const BODY: &str = "text-[15px] leading-7 text-white/70";

/// The scroll offset every anchor target needs so the sticky bar does not eat
/// the heading. Mirrors `SCROLL_OFFSET_PX` in the hooks module.
pub const SCROLL_MARGIN: &str = "scroll-mt-[8rem]";

/// Build the frosted-panel class string with extra classes appended.
pub fn glass(extra: &str) -> String {
    cn!(GLASS, extra)
}

/// Build the inset-surface class string with extra classes appended.
pub fn sunken(extra: &str) -> String {
    cn!(SUNKEN, extra)
}
