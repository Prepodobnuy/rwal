pub mod colorthief;
pub mod colorz;

use serde::Deserialize;

pub trait RwalBackend {
    fn generate_palette(&self, colors: &[(u8, u8, u8)], count: usize) -> Option<Vec<(u8, u8, u8)>>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    #[default]
    ColorZ,
    Colorthief,
}

impl From<String> for Backend {
    fn from(value: String) -> Self {
        match value.to_string().as_str() {
            "colorthief" | "ColorThief" => Backend::Colorthief,
            _ => Backend::ColorZ,
        }
    }
}

impl ToString for Backend {
    fn to_string(&self) -> String {
        match self {
            Backend::Colorthief => "colorthief",
            Backend::ColorZ => "colorz",
        }
        .to_string()
    }
}

impl RwalBackend for Backend {
    fn generate_palette(&self, colors: &[(u8, u8, u8)], count: usize) -> Option<Vec<(u8, u8, u8)>> {
        match self {
            Backend::ColorZ => colorz::ColorZ.generate_palette(colors, count),
            Backend::Colorthief => colorthief::ColorThief.generate_palette(colors, count),
        }
    }
}
