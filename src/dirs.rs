use std::path::PathBuf;
use std::sync::LazyLock;

pub static CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut path = dirs::config_dir().unwrap();
    path.push("rwal");
    path
});

pub static CONFIG_FILE: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut path = CONFIG_DIR.clone();
    path.push("config.toml");
    path
});

pub static CACHE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut path = dirs::cache_dir().unwrap();
    path.push("rwal");
    path
});

pub static HTML_PREVIEW_FILE: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut path = CACHE_DIR.clone();
    path.push("preview.html");
    path
});

pub static PREV_COLORSCHEMES_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut path = CACHE_DIR.clone();
    path.push("colorshemes");
    path
});

pub static CURRENT_COLORSCHEME_FILE: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut path = CACHE_DIR.clone();
    path.push("colors");
    path
});
