use std::cmp::Ordering;

use kmeans_colors::get_kmeans;

use palette::IntoColor;
use palette::Lab;
use palette::Srgb;

use super::RwalBackend;

pub struct ColorZ;

impl RwalBackend for ColorZ {
    fn generate_palette(&self, colors: &[(u8, u8, u8)], count: usize) -> Option<Vec<(u8, u8, u8)>> {
        if colors.is_empty() {
            return None;
        }

        let lab_colors: Vec<Lab> = colors
            .iter()
            .flat_map(|&(r, g, b)| {
                let srgb = Srgb::new(r, g, b).into_format::<f32>();
                Some(srgb.into_color())
            })
            .collect();

        let clusters = (0..3)
            .map(|i| get_kmeans(count, 100, 0.001, false, &lab_colors, 64 + i as u64))
            .min_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(Ordering::Equal))
            .unwrap();

        let mut palette_colors = Vec::with_capacity(count);

        for centroid in &clusters.centroids {
            let srgb: Srgb = (*centroid).into_color();
            let srgb_u8 = srgb.into_format::<u8>();

            palette_colors.push((srgb_u8.red, srgb_u8.green, srgb_u8.blue));
        }

        while palette_colors.len() < count {
            palette_colors.push((0, 0, 0));
        }

        Some(palette_colors)
    }
}
