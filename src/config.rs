use serde::Deserialize;

use crate::backends::Backend;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub backend: Backend,
    pub thumb_w: u32,
    pub thumb_h: u32,

    #[serde(deserialize_with = "deserialize_hex_color")]
    pub bg_color: (u8, u8, u8),
    pub bg_idx: usize,
    pub bg_strength: u8,

    #[serde(deserialize_with = "deserialize_hex_color")]
    pub fg_color: (u8, u8, u8),
    pub fg_idx: usize,
    pub fg_strength: u8,

    pub light: bool,

    pub clamp_saturation: bool,
    pub clamp_value: bool,
    pub skip_saturation: bool,
    pub skip_value: bool,

    pub clamp_value_min: f32,
    pub clamp_value_max: f32,

    pub clamp_saturation_min: f32,
    pub clamp_saturation_max: f32,

    pub skip_value_min: f32,
    pub skip_value_max: f32,

    pub skip_saturation_min: f32,
    pub skip_saturation_max: f32,
}

impl Config {
    pub fn from_file<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        log::info!("Reading config");
        let contents = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;

        config.validate()?;

        Ok(config)
    }

    pub fn cache_string(&self) -> String {
        format!(
            "{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}",
            self.backend.to_string(),
            self.thumb_w,
            self.thumb_h,
            rgb_to_hex(self.bg_color),
            self.bg_idx,
            self.bg_strength,
            rgb_to_hex(self.fg_color),
            self.fg_idx,
            self.fg_strength,
            self.light,
            self.clamp_saturation,
            self.clamp_value,
            self.skip_saturation,
            self.skip_value,
            self.clamp_value_min,
            self.clamp_value_max,
            self.clamp_saturation_min,
            self.clamp_saturation_max,
            self.skip_value_min,
            self.skip_value_max,
            self.skip_saturation_min,
            self.skip_saturation_max,
        )
    }

    fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Validating config");
        if self.thumb_w < 1 {
            return Err("thumb_w must be at least 1".into());
        }
        if self.thumb_h < 1 {
            return Err("thumb_h must be at least 1".into());
        }

        if self.bg_idx > 7 {
            return Err("bg_idx must be between 1 and 7".into());
        }
        if self.fg_idx > 7 {
            return Err("fg_idx must be between 1 and 7".into());
        }

        if self.bg_strength > 100 {
            return Err("bg_strength must be between 0 and 100".into());
        }
        if self.fg_strength > 100 {
            return Err("fg_strength must be between 0 and 100".into());
        }

        let float_validations = [
            ("clamp_value_min", self.clamp_value_min),
            ("clamp_value_max", self.clamp_value_max),
            ("clamp_saturation_min", self.clamp_saturation_min),
            ("clamp_saturation_max", self.clamp_saturation_max),
            ("skip_value_min", self.skip_value_min),
            ("skip_value_max", self.skip_value_max),
            ("skip_saturation_min", self.skip_saturation_min),
            ("skip_saturation_max", self.skip_saturation_max),
        ];

        for (name, value) in float_validations {
            if !(0.0..1.0).contains(&value) {
                return Err(format!("{} must be between 0.0 and 1.0", name).into());
            }
        }

        if self.clamp_value_min > self.clamp_value_max {
            return Err("clamp_value_min must be <= clamp_value_max".into());
        }
        if self.clamp_saturation_min > self.clamp_saturation_max {
            return Err("clamp_saturation_min must be <= clamp_saturation_max".into());
        }
        if self.skip_value_min > self.skip_value_max {
            return Err("skip_value_min must be <= skip_value_max".into());
        }
        if self.skip_saturation_min > self.skip_saturation_max {
            return Err("skip_saturation_min must be <= skip_saturation_max".into());
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            backend: Backend::ColorZ,
            thumb_w: 100,
            thumb_h: 100,
            bg_color: (0, 0, 0),
            bg_idx: 0,
            bg_strength: 10,
            fg_color: (255, 255, 255),
            fg_idx: 0,
            fg_strength: 10,
            light: false,
            clamp_saturation: true,
            clamp_value: true,
            skip_saturation: true,
            skip_value: false,
            clamp_value_min: 0.4,
            clamp_value_max: 0.5,
            clamp_saturation_min: 0.4,
            clamp_saturation_max: 0.41,
            skip_value_min: 0.1,
            skip_value_max: 0.9,
            skip_saturation_min: 0.3,
            skip_saturation_max: 0.7,
        }
    }
}

fn deserialize_hex_color<'de, D>(deserializer: D) -> Result<(u8, u8, u8), D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    hex_to_rgb(&s).map_err(serde::de::Error::custom)
}

pub fn hex_to_rgb(hex: &str) -> Result<(u8, u8, u8), String> {
    if !hex.starts_with('#') || hex.len() != 7 {
        return Err(format!("Invalid hex color format: {}", hex));
    }

    let r =
        u8::from_str_radix(&hex[1..3], 16).map_err(|e| format!("Invalid red component: {}", e))?;
    let g = u8::from_str_radix(&hex[3..5], 16)
        .map_err(|e| format!("Invalid green component: {}", e))?;
    let b =
        u8::from_str_radix(&hex[5..7], 16).map_err(|e| format!("Invalid blue component: {}", e))?;

    Ok((r, g, b))
}

pub fn rgb_to_hex(rgb: (u8, u8, u8)) -> String {
    format!("#{:02x}{:02x}{:02x}", rgb.0, rgb.1, rgb.2)
}
