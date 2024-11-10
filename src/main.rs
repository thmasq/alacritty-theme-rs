use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use dirs::config_dir;
use include_dir::{include_dir, Dir};
use std::fs;
use std::fs::create_dir_all;
use std::io::{self};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use structured_data::structs::{merge_colors, Colors};
use toml::Value;
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, List, ListItem, Paragraph};
use tui::Terminal;
use utils::example::return_example;

mod structured_data;
mod utils;

const THEMES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/themes");

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
	let config_path = config_dir()
		.map(|path| path.join("alacritty/alacritty.toml"))
		.ok_or("Could not determine Alacritty's config directory")?;

	let original_config = fs::read_to_string(&config_path).unwrap_or_default();
	let original_colors = extract_colors_from_config(&original_config)?;

	let running = Arc::new(AtomicBool::new(true));

	ensure_themes_directory()?;
	let themes_path = config_dir()
		.ok_or("Could not determine XDG_CONFIG directory")?
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
	let entries: Vec<_> = std::fs::read_dir(themes_path)?.filter_map(|entry| entry.ok()).collect();
	let theme_names: Vec<_> = entries
		.iter()
		.filter_map(|entry| entry.file_name().into_string().ok())
		.collect();

	enable_raw_mode()?;
	let mut stdout = io::stdout();
	execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;

	let mut selected_index = 0;
	let mut selected_path: Option<PathBuf> = None;
	let mut view_offset = 0;
	let mut current_preview_index = usize::MAX;

	const PAGE_JUMP: usize = 10;

	loop {
		if !running.load(Ordering::SeqCst) {
			break;
		}

		if selected_index != current_preview_index {
			current_preview_index = selected_index;
			let path = entries[selected_index].path();
			if let Ok(theme) = load_theme(&path) {
				let merged = merge_colors(default_theme, &theme);
				update_alacritty_config(config_path, merged)?;
			}
		}

		terminal.draw(|f| {
			let terminal_size = f.size();

			let main_chunks = Layout::default()
				.direction(Direction::Vertical)
				.constraints([Constraint::Min(5), Constraint::Length(3)])
				.split(terminal_size);

			let content_chunks = Layout::default()
				.direction(Direction::Horizontal)
				.constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
				.split(main_chunks[0]);

			let max_view_items = (content_chunks[0].height as usize).saturating_sub(2);

			if selected_index < view_offset {
				view_offset = selected_index;
			} else if selected_index >= view_offset + max_view_items {
				view_offset = selected_index + 1 - max_view_items;
			}

			let items: Vec<ListItem> = theme_names
				.iter()
				.enumerate()
				.skip(view_offset)
				.take(max_view_items)
				.map(|(i, name)| {
					let style = if i == selected_index {
						Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
					} else {
						Style::default()
					};
					ListItem::new(Span::styled(name.clone(), style))
				})
				.collect();

			let theme_list = List::new(items)
				.block(Block::default().borders(Borders::ALL).title("Themes"))
				.highlight_style(Style::default().bg(Color::Blue));

			let example = return_example();
			let preview = example.block(Block::default().borders(Borders::ALL).title("Preview"));

			f.render_widget(theme_list, content_chunks[0]);
			f.render_widget(preview, content_chunks[1]);

			let keybinds = Paragraph::new(Spans::from(vec![
				Span::raw("Exit: "),
				Span::styled("<Esc>", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
				Span::raw(" Select: "),
				Span::styled(
					"<Enter>",
					Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
				),
				Span::raw(" Move: "),
				Span::styled("↑↓", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
				Span::raw(" Page: "),
				Span::styled(
					"PgUp/PgDn",
					Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
				),
			]))
			.block(Block::default().borders(Borders::ALL).title("Keymap"));

			f.render_widget(keybinds, main_chunks[1]);
		})?;

		if !running.load(Ordering::SeqCst) {
			break;
		}

		if event::poll(Duration::from_millis(100))? {
			if let Event::Key(KeyEvent { code, .. }) = event::read()? {
				match code {
					KeyCode::Down => {
						if selected_index < theme_names.len() - 1 {
							selected_index += 1;
						}
					},
					KeyCode::Up => {
						if selected_index > 0 {
							selected_index -= 1;
						}
					},
					KeyCode::PageDown => {
						selected_index = (selected_index + PAGE_JUMP).min(theme_names.len().saturating_sub(1));
					},
					KeyCode::PageUp => {
						selected_index = selected_index.saturating_sub(PAGE_JUMP);
					},
					KeyCode::Enter => {
						selected_path = Some(entries[selected_index].path());
						break;
					},
					KeyCode::Esc => break,
					_ => {},
				}
			}
		}

		if !running.load(Ordering::SeqCst) {
			break;
		}
	}

	disable_raw_mode()?;
	execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
	terminal.show_cursor()?;

	Ok(selected_path)
}

fn extract_colors_from_config(config_content: &str) -> Result<Colors> {
	let config: Value = toml::from_str(config_content).unwrap_or_else(|_| toml::Value::Table(toml::map::Map::new()));

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
	let mut config: Value = toml::from_str(&content).unwrap_or_else(|_| toml::Value::Table(toml::map::Map::new()));

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
