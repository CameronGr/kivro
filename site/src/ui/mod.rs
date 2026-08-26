pub mod components;
pub mod highlight;
pub mod hooks;
pub mod icons;
pub mod style;
pub mod theme;

pub use components::*;
pub use highlight::Lang;
pub use icons::{Icon, IconData};
pub use theme::{Size, Tone, Variant};

pub mod prelude {
    pub use leptos::prelude::*;

    pub use crate::cn;
    pub use crate::ui::components::*;
    pub use crate::ui::highlight::Lang;
    pub use crate::ui::hooks::*;
    pub use crate::ui::icons::{self, Icon, IconData};
    pub use crate::ui::style::{ClassPart, glass, sunken};
    pub use crate::ui::theme::{Size, Tone, Variant};
}
