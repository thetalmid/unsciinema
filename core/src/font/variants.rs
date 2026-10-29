//! The seven Unscii font variants shipped with the app.
//!
//! Provenance: the `.hex` files in `public/fonts/` were regenerated from the
//! included Unscii 2.1 source (`unscii-2.1-src/`, by viznut) — see the salvage
//! README. Upstream states everything viznut created is Public Domain; only
//! `unscii-16-full` (not shipped here) is GPL because of Unifont pieces.
//! Matching vectorized WOFF files were downloaded from https://viznut.fi/unscii/
//! for HTML export embedding.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontVariant {
    Unscii8 = 0,
    Unscii8Alt = 1,
    Unscii8Fantasy = 2,
    Unscii8Mcr = 3,
    Unscii8Tall = 4,
    Unscii8Thin = 5,
    Unscii16 = 6,
}

impl FontVariant {
    pub const ALL: [FontVariant; 7] = [
        FontVariant::Unscii8,
        FontVariant::Unscii8Alt,
        FontVariant::Unscii8Fantasy,
        FontVariant::Unscii8Mcr,
        FontVariant::Unscii8Tall,
        FontVariant::Unscii8Thin,
        FontVariant::Unscii16,
    ];

    pub fn from_u32(v: u32) -> Option<FontVariant> {
        match v {
            0 => Some(FontVariant::Unscii8),
            1 => Some(FontVariant::Unscii8Alt),
            2 => Some(FontVariant::Unscii8Fantasy),
            3 => Some(FontVariant::Unscii8Mcr),
            4 => Some(FontVariant::Unscii8Tall),
            5 => Some(FontVariant::Unscii8Thin),
            6 => Some(FontVariant::Unscii16),
            _ => None,
        }
    }

    /// UI / file-stem name (matches the `.hex` and `.woff` file names).
    pub fn name(&self) -> &'static str {
        match self {
            FontVariant::Unscii8 => "unscii-8",
            FontVariant::Unscii8Alt => "unscii-8-alt",
            FontVariant::Unscii8Fantasy => "unscii-8-fantasy",
            FontVariant::Unscii8Mcr => "unscii-8-mcr",
            FontVariant::Unscii8Tall => "unscii-8-tall",
            FontVariant::Unscii8Thin => "unscii-8-thin",
            FontVariant::Unscii16 => "unscii-16",
        }
    }

    pub fn hex_data(&self) -> &'static str {
        match self {
            FontVariant::Unscii8 => include_str!("../../../public/fonts/unscii-8.hex"),
            FontVariant::Unscii8Alt => include_str!("../../../public/fonts/unscii-8-alt.hex"),
            FontVariant::Unscii8Fantasy => {
                include_str!("../../../public/fonts/unscii-8-fantasy.hex")
            }
            FontVariant::Unscii8Mcr => include_str!("../../../public/fonts/unscii-8-mcr.hex"),
            FontVariant::Unscii8Tall => include_str!("../../../public/fonts/unscii-8-tall.hex"),
            FontVariant::Unscii8Thin => include_str!("../../../public/fonts/unscii-8-thin.hex"),
            FontVariant::Unscii16 => include_str!("../../../public/fonts/unscii-16.hex"),
        }
    }

    /// File name of the matching upstream WOFF (used by HTML export).
    pub fn woff_name(&self) -> &'static str {
        match self {
            FontVariant::Unscii8 => "unscii-8.woff",
            FontVariant::Unscii8Alt => "unscii-8-alt.woff",
            FontVariant::Unscii8Fantasy => "unscii-8-fantasy.woff",
            FontVariant::Unscii8Mcr => "unscii-8-mcr.woff",
            FontVariant::Unscii8Tall => "unscii-8-tall.woff",
            FontVariant::Unscii8Thin => "unscii-8-thin.woff",
            FontVariant::Unscii16 => "unscii-16.woff",
        }
    }
}
