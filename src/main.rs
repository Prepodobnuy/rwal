use rand::Rng;

use crate::config::{hex_to_rgb, rgb_to_hex};

mod backends;
mod config;
mod dirs;
mod rwal;

const HELP_MESSAGE: &str = r#"
usage: rwal -i [path/to/image]

flags:
    -v                      verbose logging
    -q                      quite logging
    -i <path>               image/path-with-images to generate coloscheme from
    -l                      generate light colorscheme
    -c                      skip cache
    --help -h               show this message
    --backend <backend>     set backend ("colorz" | "colorthief")
    --thumb-w <value>       set thumb width (min=1)
    --thumb-h <value>       set thumb height (min=1)
    --clamp-s-min <value>   set min saturation clamp (0.0 - 1.0)
    --clamp-s-max <value>   set max saturation clamp (0.0 - 1.0)
    --clamp-v-min <value>   set min value clamp (0.0 - 1.0)
    --clamp-v-max <value>   set max value clamp (0.0 - 1.0)
    --skip-s-min <value>    set min saturation skip (0.0 - 1.0)
    --skip-s-max <value>    set max saturation skip (0.0 - 1.0)
    --skip-v-min <value>    set min value skip (0.0 - 1.0)
    --skip-v-max <value>    set max value skip (0.0 - 1.0)
    --skip-value            skip value
    --skip-saturation       skip saturation
    --clamp-value           clamp value
    --clamp-saturation      clamp saturation
    --bg-idx <value>        palette color to mix with bg (0-7)
    --fg-idx <value>        palette color to mix with fg (0-7)
    --bg-str <value>        amount of palette color to apply to bg (0-100)
    --fg-str <value>        amount of palette color to apply to fg (0-100)
    --bg <value>            background color (#HHEEXX)
    --fg <value>            foreground color (#HHEEXX)
"#;

fn main() {
    let mut flag = flag::Flag::new();

    if flag.get_bool("-h") || flag.get_bool("--help") {
        println!("{HELP_MESSAGE}");
        return;
    }

    unsafe {
        if flag.get_bool("-v") {
            std::env::set_var("RUST_LOG", "trace");
        } else if flag.get_bool("-q") {
            std::env::set_var("RUST_LOG", "none");
        } else {
            std::env::set_var("RUST_LOG", "info");
        }
    }

    pretty_env_logger::init();

    let mut config = match config::Config::from_file(crate::dirs::CONFIG_FILE.clone()) {
        Ok(config) => {
            log::info!("Config collected");
            config
        }
        Err(e) => {
            log::error!("{}", e);
            log::warn!("Failed to read config, using default");
            Default::default()
        }
    };

    log::info!("Reading flags");

    config.backend = flag
        .get_str("--backend")
        .map(backends::Backend::from)
        .unwrap_or(config.backend);

    config.thumb_w = flag
        .get_u32("--thumb-w")
        .map(|v| v.clamp(1, 99999))
        .unwrap_or(config.thumb_w);

    config.thumb_h = flag
        .get_u32("--thumb-h")
        .map(|v| v.clamp(1, 99999))
        .unwrap_or(config.thumb_h);

    config.clamp_saturation_min = flag
        .get_f32("--clamp-s-min")
        .map(|v| v.clamp(0.0, 1.0))
        .unwrap_or(config.clamp_saturation_min);

    config.clamp_saturation_max = flag
        .get_f32("--clamp-s-max")
        .map(|v| v.clamp(0.0, 1.0))
        .unwrap_or(config.clamp_saturation_max);

    config.clamp_value_min = flag
        .get_f32("--clamp-v-min")
        .map(|v| v.clamp(0.0, 1.0))
        .unwrap_or(config.clamp_value_min);

    config.clamp_value_max = flag
        .get_f32("--clamp-v-max")
        .map(|v| v.clamp(0.0, 1.0))
        .unwrap_or(config.clamp_value_max);

    config.skip_saturation_min = flag
        .get_f32("--skip-s-min")
        .map(|v| v.clamp(0.0, 1.0))
        .unwrap_or(config.skip_saturation_min);

    config.skip_saturation_max = flag
        .get_f32("--skip-s-max")
        .map(|v| v.clamp(0.0, 1.0))
        .unwrap_or(config.skip_saturation_max);

    config.skip_value_min = flag
        .get_f32("--skip-v-min")
        .map(|v| v.clamp(0.0, 1.0))
        .unwrap_or(config.skip_value_min);

    config.skip_value_max = flag
        .get_f32("--skip-v-max")
        .map(|v| v.clamp(0.0, 1.0))
        .unwrap_or(config.skip_value_max);

    config.bg_color = flag
        .get_str("--bg")
        .and_then(|v| hex_to_rgb(&v).ok())
        .unwrap_or(config.bg_color);

    config.fg_color = flag
        .get_str("--fg")
        .and_then(|v| hex_to_rgb(&v).ok())
        .unwrap_or(config.fg_color);

    config.bg_idx = flag
        .get_u32("--bg-idx")
        .map(|v| v as usize)
        .unwrap_or(config.bg_idx);

    config.fg_idx = flag
        .get_u32("--fg-idx")
        .map(|v| v as usize)
        .unwrap_or(config.fg_idx);

    config.bg_strength = flag
        .get_u32("--bg-str")
        .map(|v| v as u8)
        .unwrap_or(config.bg_strength);

    config.fg_strength = flag
        .get_u32("--fg-str")
        .map(|v| v as u8)
        .unwrap_or(config.fg_strength);

    config.skip_value |= flag.get_bool("--skip-value");
    config.skip_saturation |= flag.get_bool("--skip-saturation");
    config.clamp_value |= flag.get_bool("--clamp-value");
    config.clamp_saturation |= flag.get_bool("--clamp-saturation");
    config.light |= flag.get_bool("-l");

    if config.light {
        std::mem::swap(&mut config.bg_color, &mut config.fg_color);
    }

    let Some(image) = flag.get_str("-i") else {
        log::info!("No image path specified");
        log::info!("Exiting...");
        return;
    };

    let path = std::path::Path::new(&image);
    let mut image = image.clone();

    if !path.exists() {
        log::info!("path {} does not exist", &image);
        log::info!("Exiting...");
        return;
    }

    if path.is_dir() {
        log::info!("Collecting files from {}", &image);
        let images = collect_images(path);

        if images.is_empty() {
            log::info!("No image files found at {}", &image);
            log::info!("Exiting...");
            return;
        }

        let mut rand = rand::rng();
        let index = rand.random_range(0..images.len());
        image = images[index].to_string_lossy().to_string();

        log::info!("Choosen image {}", image);
    }

    let skip_cache = flag.get_bool("-c");

    let rwal = rwal::Rwal {
        backend: config.backend,
        image_resize: (config.thumb_w, config.thumb_h),

        bg_idx: config.bg_idx,
        bg_color: config.bg_color,
        bg_strength: config.bg_strength,

        fg_idx: config.fg_idx,
        fg_color: config.fg_color,
        fg_strength: config.fg_strength,

        clamp_saturation: config.clamp_saturation,
        saturation_clamp: (config.clamp_saturation_min, config.clamp_saturation_max),

        skip_saturation: config.skip_saturation,
        saturation_skip: (config.skip_saturation_min, config.skip_saturation_max),

        clamp_value: config.clamp_value,
        value_clamp: (config.clamp_value_min, config.clamp_value_max),

        skip_value: config.skip_value,
        value_skip: (config.skip_value_min, config.skip_value_max),
    };

    if !crate::dirs::CACHE_DIR.exists() {
        let _ = std::fs::create_dir_all(crate::dirs::CACHE_DIR.clone());
    }

    if !crate::dirs::PREV_COLORSCHEMES_DIR.exists() {
        let _ = std::fs::create_dir_all(crate::dirs::PREV_COLORSCHEMES_DIR.clone());
    }

    if skip_cache {
        log::info!("Skipping cache");

        let colorscheme = match rwal.generate_colorscheme(&image) {
            Ok(colorscheme) => colorscheme,
            Err(e) => {
                log::error!("Failed to get colorscheme: {:#?}", e);
                return;
            }
        };

        let _ = std::fs::write(
            crate::dirs::HTML_PREVIEW_FILE.clone(),
            colorscheme.html_preview(),
        );

        let _ = std::fs::write(
            crate::dirs::CURRENT_COLORSCHEME_FILE.clone(),
            colorscheme
                .into_array()
                .into_iter()
                .map(rgb_to_hex)
                .collect::<Vec<String>>()
                .join("\n"),
        );

        return;
    }

    let name = image
        .split("/")
        .last()
        .map(|p| p.to_string())
        .unwrap_or(path.to_string_lossy().to_string());
    let cache_name = format!("{}{}", config.cache_string(), name);
    let mut cache_path = crate::dirs::PREV_COLORSCHEMES_DIR.clone();
    cache_path.push(cache_name);

    if cache_path.exists() {
        log::info!("Cache exists");
        let _ = std::fs::copy(&cache_path, crate::dirs::CURRENT_COLORSCHEME_FILE.clone());
        log::info!("Exiting...");
        return;
    }

    let colorscheme = match rwal.generate_colorscheme(&image) {
        Ok(colorscheme) => colorscheme,
        Err(e) => {
            log::error!("Failed to get colorscheme: {:#?}", e);
            return;
        }
    };

    let _ = std::fs::write(
        crate::dirs::HTML_PREVIEW_FILE.clone(),
        colorscheme.html_preview(),
    );

    let _ = std::fs::write(
        &cache_path,
        colorscheme
            .into_array()
            .into_iter()
            .map(rgb_to_hex)
            .collect::<Vec<String>>()
            .join("\n"),
    );

    let _ = std::fs::write(
        crate::dirs::CURRENT_COLORSCHEME_FILE.clone(),
        colorscheme
            .into_array()
            .into_iter()
            .map(rgb_to_hex)
            .collect::<Vec<String>>()
            .join("\n"),
    );
}

fn collect_images(path: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut result = Vec::new();

    let Ok(rd) = path.read_dir() else {
        return result;
    };

    for entry in rd {
        let Ok(entry) = entry else { continue };

        let path = entry.path();

        if path.is_dir() {
            result.extend(collect_images(&path));
            continue;
        }

        if path.is_file()
            && let Some(extension) = path.extension()
            && let Some(extension) = extension.to_str()
            && matches!(extension, "jpg" | "jpeg" | "png" | "tiff" | "webp")
        {
            result.push(path);
        }
    }

    result
}
