#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub enum Tone {
    #[default]
    Accent,
    Neutral,
    Info,
    Warning,
    Danger,
    Dev,
}

impl Tone {
    pub const fn border(self) -> &'static str {
        match self {
            Tone::Accent => "border-accent-400/30",
            Tone::Neutral => "border-white/10",
            Tone::Info => "border-sky-400/30",
            Tone::Warning => "border-amber-400/30",
            Tone::Danger => "border-red-400/30",
            Tone::Dev => "border-violet-400/30",
        }
    }

    /// Background wash for a soft (tinted) surface.
    pub const fn bg(self) -> &'static str {
        match self {
            Tone::Accent => "bg-accent-500/10",
            Tone::Neutral => "bg-white/[0.04]",
            Tone::Info => "bg-sky-500/10",
            Tone::Warning => "bg-amber-500/10",
            Tone::Danger => "bg-red-500/10",
            Tone::Dev => "bg-violet-500/15",
        }
    }

    /// Foreground for text sitting on the soft surface.
    pub const fn text(self) -> &'static str {
        match self {
            Tone::Accent => "text-accent-200",
            Tone::Neutral => "text-white/70",
            Tone::Info => "text-sky-200",
            Tone::Warning => "text-amber-200",
            Tone::Danger => "text-red-200",
            Tone::Dev => "text-violet-300",
        }
    }

    /// Colour for a standalone glyph (icon, bullet, rule).
    pub const fn icon(self) -> &'static str {
        match self {
            Tone::Accent => "text-accent-400",
            Tone::Neutral => "text-white/45",
            Tone::Info => "text-sky-400",
            Tone::Warning => "text-amber-400",
            Tone::Danger => "text-red-400",
            Tone::Dev => "text-violet-400",
        }
    }

    /// Filled treatment — used sparingly, mostly for the primary button.
    pub const fn solid(self) -> &'static str {
        match self {
            Tone::Accent => "bg-accent-500/90 text-black hover:bg-accent-400",
            Tone::Neutral => "bg-white/90 text-black hover:bg-white",
            Tone::Info => "bg-sky-500/90 text-black hover:bg-sky-400",
            Tone::Warning => "bg-amber-500/90 text-black hover:bg-amber-400",
            Tone::Danger => "bg-red-500/90 text-white hover:bg-red-500",
            Tone::Dev => "bg-violet-500/90 text-white hover:bg-violet-500",
        }
    }

    /// Focus ring colour.
    pub const fn ring(self) -> &'static str {
        match self {
            Tone::Accent => "focus-visible:ring-accent-400/40",
            Tone::Neutral => "focus-visible:ring-white/30",
            Tone::Info => "focus-visible:ring-sky-400/40",
            Tone::Warning => "focus-visible:ring-amber-400/40",
            Tone::Danger => "focus-visible:ring-red-400/40",
            Tone::Dev => "focus-visible:ring-violet-400/40",
        }
    }

    /// The soft surface as one string: border + background + text.
    pub fn soft(self) -> String {
        crate::cn!(self.border(), self.bg(), self.text())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub enum Size {
    Xs,
    Sm,
    #[default]
    Md,
    Lg,
}

impl Size {
    /// Padding + text size for a text button or pill.
    pub const fn control(self) -> &'static str {
        match self {
            Size::Xs => "px-2.5 py-1 text-[11px] gap-1.5",
            Size::Sm => "px-3 py-1.5 text-xs gap-1.5",
            Size::Md => "px-4 py-2 text-sm gap-2",
            Size::Lg => "px-5 py-2.5 text-base gap-2.5",
        }
    }

    /// Padding for an input-like control (taller than a button at the same step).
    pub const fn field(self) -> &'static str {
        match self {
            Size::Xs => "px-2.5 py-1.5 text-xs",
            Size::Sm => "px-3 py-2 text-xs",
            Size::Md => "px-4 py-2.5 text-sm",
            Size::Lg => "px-4 py-3 text-base",
        }
    }

    /// Square side for an icon-only control.
    pub const fn square(self) -> &'static str {
        match self {
            Size::Xs => "h-7 w-7",
            Size::Sm => "h-8 w-8",
            Size::Md => "h-10 w-10",
            Size::Lg => "h-12 w-12",
        }
    }

    /// Matching glyph size.
    pub const fn icon(self) -> &'static str {
        match self {
            Size::Xs => "h-3 w-3",
            Size::Sm => "h-3.5 w-3.5",
            Size::Md => "h-4 w-4",
            Size::Lg => "h-5 w-5",
        }
    }

    /// Corner radius. The system leans large: 2xl for controls, 3xl for panels.
    pub const fn radius(self) -> &'static str {
        match self {
            Size::Xs | Size::Sm => "rounded-xl",
            Size::Md | Size::Lg => "rounded-2xl",
        }
    }
}

/// Rendering treatment for buttons and other pressable surfaces.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub enum Variant {
    /// Tinted glass with a coloured hairline — the house default.
    #[default]
    Soft,
    /// Filled. Reserve for the single primary action on a view.
    Solid,
    /// Neutral inset surface that warms to the tone on hover.
    Glass,
    /// No chrome until hover.
    Ghost,
    /// Renders as underlined text.
    Link,
}
