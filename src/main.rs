use console::Term;
use dialoguer::{theme::ColorfulTheme, Select};
use dirs::config_dir;
use include_dir::{include_dir, Dir};
use std::fs;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use structured_data::structs::{merge_colors, Colors};
use toml::Value;

mod structured_data;

static THEMES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/themes");
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let config_path = config_dir()
        .map(|path| path.join("alacritty/alacritty.toml"))
        .ok_or("Could not determine config directory")?;

    let original_config = fs::read_to_string(&config_path).unwrap_or_default();
    let original_colors = extract_colors_from_config(&original_config)?;

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    ensure_themes_directory()?;
    let themes_path = config_dir()
        .ok_or("Could not determine config directory")?
        .join("alacritty_themes");

    let default_theme = load_theme(&themes_path.join("Default.dark.toml"))?;

    let result = select_theme_with_preview(&themes_path, &config_path, &default_theme, &running);

    if !running.load(Ordering::SeqCst) || result?.is_none() {
        restore_config(&config_path, original_colors)?;
    }

    Ok(())
}

fn select_theme_with_preview(
    themes_path: &Path,
    config_path: &Path,
    default_theme: &Colors,
    running: &Arc<AtomicBool>,
) -> Result<Option<PathBuf>> {
    let entries: Vec<_> = fs::read_dir(themes_path)?
        .filter_map(|entry| entry.ok())
        .collect();
    let theme_names: Vec<_> = entries
        .iter()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();

    let binding = ColorfulTheme::default();

    let select = Select::with_theme(&binding)
        .with_prompt("Select a theme (Press ↵ to apply, Esc to cancel)")
        .items(&theme_names);

    let term = Term::stderr();

    let selection = select.interact_on_opt(&term)?.and_then(|i| {
        if running.load(Ordering::SeqCst) {
            let path = entries[i].path();
            if let Ok(theme) = load_theme(&path) {
                let merged = merge_colors(default_theme, &theme);
                let _ = update_alacritty_config(config_path, merged);
            }
            Some(path)
        } else {
            None
        }
    });

    Ok(selection)
}

fn extract_colors_from_config(config_content: &str) -> Result<Colors> {
    let config: Value = toml::from_str(config_content)
        .unwrap_or_else(|_| toml::Value::Table(toml::map::Map::new()));

    let colors = config
        .get("colors")
        .cloned()
        .unwrap_or_else(|| Value::Table(toml::map::Map::new()))
        .try_into()?;

    Ok(colors)
}

fn restore_config(config_path: &Path, original_colors: Colors) -> Result<()> {
    update_alacritty_config(config_path, original_colors)
}

fn update_alacritty_config(config_path: &Path, colors: Colors) -> Result<()> {
    let content = fs::read_to_string(config_path).unwrap_or_default();
    let mut config: Value =
        toml::from_str(&content).unwrap_or_else(|_| toml::Value::Table(toml::map::Map::new()));

    if let Some(table) = config.as_table_mut() {
        if let Ok(colors_value) = toml::Value::try_from(&colors) {
            if let Some(colors_table) = colors_value.as_table() {
                if !colors_table.is_empty() {
                    table.insert("colors".to_string(), Value::Table(colors_table.clone()));
                }
            }
        }
    }

    fs::write(config_path, toml::to_string_pretty(&config)?)?;
    Ok(())
}

fn load_theme(path: &Path) -> Result<Colors> {
    let content = fs::read_to_string(path)?;
    let config: Value = toml::from_str(&content)?;
    let colors = config
        .get("colors")
        .ok_or("No colors section found")?
        .clone()
        .try_into()?;
    Ok(colors)
}

fn ensure_themes_directory() -> Result<()> {
    if let Some(config_home) = config_dir() {
        let themes_path = config_home.join("alacritty_themes");
        if !themes_path.exists() {
            create_dir_all(&themes_path)?;
            for file in THEMES_DIR.files() {
                fs::write(themes_path.join(file.path()), file.contents())?;
            }
        }
    }
    Ok(())
}
