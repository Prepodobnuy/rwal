use super::RwalBackend;

pub struct ColorThief;

impl RwalBackend for ColorThief {
    fn generate_palette(&self, colors: &[(u8, u8, u8)], _: usize) -> Option<Vec<(u8, u8, u8)>> {
        let pixels = colors
            .iter()
            .map(|c| [c.0, c.1, c.2])
            .collect::<Vec<[u8; 3]>>()
            .concat();

        let colors =
            color_thief::get_palette(&pixels, color_thief::ColorFormat::Rgb, 5, 255).ok()?;

        Some(colors.into_iter().map(|c| (c.r, c.g, c.b)).collect())
    }
}
