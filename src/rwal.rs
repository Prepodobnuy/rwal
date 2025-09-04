use image::RgbImage;
use palette::FromColor;
use palette::Hsv;
use palette::Srgb;

use crate::backends::Backend;
use crate::backends::RwalBackend;

pub struct Rwal {
    pub backend: Backend,
    pub image_resize: (u32, u32),

    pub bg_idx: usize,
    pub bg_color: (u8, u8, u8),
    pub bg_strength: u8,

    pub fg_idx: usize,
    pub fg_strength: u8,
    pub fg_color: (u8, u8, u8),

    pub clamp_saturation: bool,
    pub saturation_clamp: (f32, f32),

    pub skip_saturation: bool,
    pub saturation_skip: (f32, f32),

    pub clamp_value: bool,
    pub value_clamp: (f32, f32),

    pub skip_value: bool,
    pub value_skip: (f32, f32),
}

impl Rwal {
    fn prepare_colors(&self, image: RgbImage) -> Vec<(u8, u8, u8)> {
        let s_min = self.saturation_clamp.0;
        let s_max = self.saturation_clamp.1;
        let v_min = self.value_clamp.0;
        let v_max = self.value_clamp.1;

        let s_skip_min = self.saturation_skip.0;
        let s_skip_max = self.saturation_skip.1;
        let v_skip_min = self.value_skip.0;
        let v_skip_max = self.value_skip.1;

        image
            .pixels()
            .map(|p| {
                let srgb_u8 = Srgb::new(p[0], p[1], p[2]);
                let srgb_f32: Srgb<f32> = srgb_u8.into_format();

                Hsv::from_color(srgb_f32)
            })
            .filter(|c| {
                if !self.skip_saturation {
                    true
                } else {
                    c.saturation > s_skip_min && c.saturation < s_skip_max
                }
            })
            .filter(|c| {
                if !self.skip_value {
                    true
                } else {
                    c.value > v_skip_min && c.value < v_skip_max
                }
            })
            .map(|c| {
                let mut hsv: Hsv = c;

                if self.clamp_saturation {
                    hsv.saturation = hsv.saturation.clamp(s_min, s_max);
                }
                if self.clamp_value {
                    hsv.value = hsv.value.clamp(v_min, v_max);
                }

                let clamped_rgb: Srgb<f32> = Srgb::from_color(hsv);
                let clamped_rgb_u8: Srgb<u8> = clamped_rgb.into_format();

                (
                    clamped_rgb_u8.red,
                    clamped_rgb_u8.green,
                    clamped_rgb_u8.blue,
                )
            })
            .collect()
    }

    pub fn generate_colorscheme(&self, path: &str) -> Result<Colorscheme, &'static str> {
        let img = image::open(path).map_err(|_| "Failed to open image")?;
        let img = img.resize_exact(
            self.image_resize.0,
            self.image_resize.1,
            image::imageops::Nearest,
        );

        let colors = self.prepare_colors(img.to_rgb8());

        let Some(palette) = self.backend.generate_palette(&colors, 8) else {
            return Err("Failed to generate palette");
        };

        let palette = sort_by_hue(&palette);

        if palette.len() < 8 {
            return Err("Not enough colors generated");
        }

        let palette = sort_by_hue(&palette);

        let bg = mix_colors(self.bg_color, palette[self.bg_idx], self.bg_strength);
        let fg = mix_colors(self.fg_color, palette[self.fg_idx], self.fg_strength);

        Ok(Colorscheme {
            t0: bg,
            t1: palette[1],
            t2: palette[2],
            t3: palette[3],
            t4: palette[4],
            t5: palette[5],
            t6: palette[6],
            t7: fg,
            t8: mix_colors(bg, (255, 255, 255), 10),
            t9: mix_colors(palette[1], (255, 255, 255), 30),
            t10: mix_colors(palette[2], (255, 255, 255), 30),
            t11: mix_colors(palette[3], (255, 255, 255), 30),
            t12: mix_colors(palette[4], (255, 255, 255), 30),
            t13: mix_colors(palette[5], (255, 255, 255), 30),
            t14: mix_colors(palette[6], (255, 255, 255), 30),
            t15: mix_colors(fg, (255, 255, 255), 10),
        })
    }
}

#[derive(Clone, Copy)]
pub struct Colorscheme {
    pub t0: (u8, u8, u8),
    pub t1: (u8, u8, u8),
    pub t2: (u8, u8, u8),
    pub t3: (u8, u8, u8),
    pub t4: (u8, u8, u8),
    pub t5: (u8, u8, u8),
    pub t6: (u8, u8, u8),
    pub t7: (u8, u8, u8),

    pub t8: (u8, u8, u8),
    pub t9: (u8, u8, u8),
    pub t10: (u8, u8, u8),
    pub t11: (u8, u8, u8),
    pub t12: (u8, u8, u8),
    pub t13: (u8, u8, u8),
    pub t14: (u8, u8, u8),
    pub t15: (u8, u8, u8),
}

impl Colorscheme {
    pub fn html_preview(&self) -> String {
        const DIV: &str = include_str!("./div.html");
        const PREV: &str = include_str!("./preview.html");

        let mut dark_divs = Vec::new();
        let mut light_divs = Vec::new();

        let bg = self.t0;
        let fg = self.t7;

        let dark = [
            self.t0, self.t1, self.t2, self.t3, self.t4, self.t5, self.t6, self.t7,
        ];
        let light = [
            self.t8, self.t9, self.t10, self.t11, self.t12, self.t13, self.t14, self.t15,
        ];

        for c in dark {
            let div = DIV
                .replace("R", &c.0.to_string())
                .replace("G", &c.1.to_string())
                .replace("B", &c.2.to_string());
            dark_divs.push(div);
        }

        for c in light {
            let div = DIV
                .replace("R", &c.0.to_string())
                .replace("G", &c.1.to_string())
                .replace("B", &c.2.to_string());
            light_divs.push(div);
        }

        PREV.replace("{{DDIV}}", &dark_divs.join(""))
            .replace("{{LDIV}}", &light_divs.join(""))
            .replace("{{BR}}", &bg.0.to_string())
            .replace("{{BG}}", &bg.1.to_string())
            .replace("{{BB}}", &bg.2.to_string())
            .replace("{{FR}}", &fg.0.to_string())
            .replace("{{FG}}", &fg.1.to_string())
            .replace("{{FB}}", &fg.2.to_string())
    }

    pub fn into_array(self) -> [(u8, u8, u8); 16] {
        [
            self.t0, self.t1, self.t2, self.t3, self.t4, self.t5, self.t6, self.t7, self.t8,
            self.t9, self.t10, self.t11, self.t12, self.t13, self.t14, self.t15,
        ]
    }
}

fn sort_by_hue(palette: &[(u8, u8, u8)]) -> Vec<(u8, u8, u8)> {
    let mut hsv_palette: Vec<Hsv> = palette
        .iter()
        .map(|c| {
            let srgb_u8 = Srgb::new(c.0, c.1, c.2);
            let srgb_f32: Srgb<f32> = srgb_u8.into_format();
            Hsv::from_color(srgb_f32)
        })
        .collect();

    hsv_palette.sort_by(|f, s| {
        let f_hue: f32 = f.hue.into();
        let s_hue: f32 = s.hue.into();
        f_hue.partial_cmp(&s_hue).unwrap()
    });

    hsv_palette
        .into_iter()
        .map(|hsv| {
            let rgb: Srgb<f32> = Srgb::from_color(hsv);
            let rgb_u8: Srgb<u8> = rgb.into_format();
            (rgb_u8.red, rgb_u8.green, rgb_u8.blue)
        })
        .collect()
}

fn mix_colors(f: (u8, u8, u8), s: (u8, u8, u8), pos: u8) -> (u8, u8, u8) {
    let pos = pos.clamp(0, 100) as u16;

    let interpolate = |a: u8, b: u8| -> u8 {
        let a = a as u16;
        let b = b as u16;
        let result = a * (100 - pos) + b * pos;
        (result / 100) as u8
    };

    (
        interpolate(f.0, s.0),
        interpolate(f.1, s.1),
        interpolate(f.2, s.2),
    )
}
