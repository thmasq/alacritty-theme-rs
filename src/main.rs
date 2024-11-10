use crossterm::event::{
	self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, MouseEvent, MouseEventKind,
};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use dirs::config_dir;
use include_dir::{include_dir, Dir};
use std::fs;
use std::fs::create_dir_all;
use std::io::{self, Write};
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
		restore_config(&config_path, &original_colors)?;
	}

	Ok(())
}

fn select_theme_with_preview(
	themes_path: &Path,
	config_path: &Path,
	default_theme: &Colors,
	running: &Arc<AtomicBool>,
) -> Result<Option<PathBuf>> {
	let entries = read_theme_entries(themes_path)?;
	let theme_names = extract_theme_names(&entries);

	setup_terminal()?;
	let result = run_event_loop(&entries, &theme_names, config_path, default_theme, running);
	cleanup_terminal()?;

	result
}

fn read_theme_entries(themes_path: &Path) -> Result<Vec<std::fs::DirEntry>> {
	Ok(std::fs::read_dir(themes_path)?
		.filter_map(std::result::Result::ok)
		.collect())
}

fn extract_theme_names(entries: &[std::fs::DirEntry]) -> Vec<String> {
	entries
		.iter()
		.filter_map(|entry| entry.file_name().into_string().ok())
		.collect()
}

fn setup_terminal() -> Result<()> {
	enable_raw_mode()?;
	let mut stdout = io::stdout();
	execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
	Ok(())
}

fn cleanup_terminal() -> Result<()> {
	disable_raw_mode()?;
	let mut stdout = io::stdout();
	execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
	stdout.flush()?;
	Ok(())
}

fn run_event_loop(
	entries: &[std::fs::DirEntry],
	theme_names: &[String],
	config_path: &Path,
	default_theme: &Colors,
	running: &Arc<AtomicBool>,
) -> Result<Option<PathBuf>> {
	let backend = CrosstermBackend::new(io::stdout());
	let mut terminal = Terminal::new(backend)?;

	let mut selected_index = 0;
	let mut selected_path: Option<PathBuf> = None;
	let mut view_offset = 0;
	let mut current_preview_index = usize::MAX;

	while running.load(Ordering::SeqCst) {
		if selected_index != current_preview_index {
			update_theme_preview(&entries[selected_index], config_path, default_theme)?;
			current_preview_index = selected_index;
		}

		draw_ui(&mut terminal, theme_names, selected_index, view_offset)?;

		if !event::poll(Duration::from_millis(100))? {
			continue;
		}

		match event::read()? {
			Event::Key(key_event) => {
				if !handle_key_event(
					key_event,
					&mut selected_index,
					theme_names.len(),
					&mut terminal,
					&mut selected_path,
					entries,
				)? {
					break;
				}
			},
			Event::Mouse(mouse_event) => {
				handle_mouse_event(
					mouse_event,
					&mut selected_index,
					&mut view_offset,
					theme_names.len(),
					&mut terminal,
				)?;
			},
			_ => {},
		}

		adjust_view_offset(&mut terminal, &mut view_offset, selected_index)?;
	}

	Ok(selected_path)
}

fn handle_mouse_event(
	mouse_event: MouseEvent,
	selected_index: &mut usize,
	view_offset: &mut usize,
	theme_count: usize,
	terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<()> {
	let terminal_height = terminal.size()?.height as usize;
	let visible_items = terminal_height.saturating_sub(5);
	let list_area_start = 1;
	let list_area_end = visible_items + 1;

	let overlap = 3.min(visible_items / 4);
	let page_jump = visible_items.saturating_sub(overlap);

	match mouse_event.kind {
		MouseEventKind::ScrollDown => {
			if *selected_index < theme_count.saturating_sub(1) {
				*selected_index = (*selected_index + page_jump).min(theme_count.saturating_sub(1));
			}
		},
		MouseEventKind::ScrollUp => {
			if *selected_index > 0 {
				*selected_index = selected_index.saturating_sub(page_jump);
			}
		},
		MouseEventKind::Down(_) => {
			let mouse_y = mouse_event.row as usize;
			let mouse_x = mouse_event.column as usize;
			if mouse_x <= (terminal.size()?.width as usize) / 2 && mouse_y >= list_area_start && mouse_y < list_area_end
			{
				let clicked_index = *view_offset + (mouse_y - list_area_start);
				if clicked_index < theme_count {
					*selected_index = clicked_index;
				}
			}
		},
		_ => {},
	}

	Ok(())
}

fn update_theme_preview(entry: &std::fs::DirEntry, config_path: &Path, default_theme: &Colors) -> Result<()> {
	let path = entry.path();
	if let Ok(theme) = load_theme(&path) {
		let merged = merge_colors(default_theme, &theme);
		update_alacritty_config(config_path, &merged)?;
	}
	Ok(())
}

fn draw_ui(
	terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
	theme_names: &[String],
	selected_index: usize,
	view_offset: usize,
) -> Result<()> {
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

		let visible_height = (content_chunks[0].height as usize).saturating_sub(2);

		let items: Vec<ListItem> = theme_names
			.iter()
			.skip(view_offset)
			.take(visible_height)
			.enumerate()
			.map(|(i, name)| {
				let is_selected = i + view_offset == selected_index;
				let mut style = if is_selected {
					Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
				} else {
					Style::default()
				};

				if (i == 0 && view_offset > 0)
					|| (i == visible_height - 1 && view_offset + visible_height < theme_names.len())
				{
					style = style.add_modifier(Modifier::DIM);
				}

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
			Span::raw(" Preview: "),
			Span::styled("Click", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
			Span::raw(" Navigate: "),
			Span::styled("↑↓", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
			Span::raw(" Page: "),
			Span::styled(
				"Scroll/PgUp/PgDn",
				Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
			),
		]))
		.block(Block::default().borders(Borders::ALL).title("Keymap"));

		f.render_widget(keybinds, main_chunks[1]);
	})?;
	Ok(())
}

fn handle_key_event(
	key_event: KeyEvent,
	selected_index: &mut usize,
	theme_count: usize,
	terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
	selected_path: &mut Option<PathBuf>,
	entries: &[std::fs::DirEntry],
) -> Result<bool> {
	let terminal_height = terminal.size()?.height as usize;
	let visible_items = terminal_height.saturating_sub(5);
	let overlap = 3.min(visible_items / 4);
	let page_jump = visible_items.saturating_sub(overlap);

	match key_event.code {
		KeyCode::Down => {
			if *selected_index < theme_count.saturating_sub(1) {
				*selected_index += 1;
			}
		},
		KeyCode::Up => {
			if *selected_index > 0 {
				*selected_index = selected_index.saturating_sub(1);
			}
		},
		KeyCode::PageDown => {
			if *selected_index + page_jump >= theme_count {
				*selected_index = theme_count.saturating_sub(1);
			} else {
				*selected_index += page_jump;
			}
		},
		KeyCode::PageUp => {
			if *selected_index < page_jump {
				*selected_index = 0;
			} else {
				*selected_index -= page_jump;
			}
		},
		KeyCode::Enter => {
			*selected_path = Some(entries[*selected_index].path());
			return Ok(false);
		},
		KeyCode::Esc => return Ok(false),
		_ => {},
	}

	Ok(true)
}

fn adjust_view_offset(
	terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
	view_offset: &mut usize,
	selected_index: usize,
) -> Result<()> {
	let terminal_height = terminal.size()?.height as usize;
	let visible_items = terminal_height.saturating_sub(5);
	let buffer_zone = 3.min(visible_items / 4);

	if selected_index < *view_offset + buffer_zone {
		*view_offset = selected_index.saturating_sub(buffer_zone);
	} else if selected_index >= *view_offset + visible_items.saturating_sub(buffer_zone) {
		*view_offset = selected_index.saturating_sub(visible_items.saturating_sub(buffer_zone + 1));
	}

	Ok(())
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

fn restore_config(config_path: &Path, original_colors: &Colors) -> Result<()> {
	update_alacritty_config(config_path, original_colors)
}

fn update_alacritty_config(config_path: &Path, colors: &Colors) -> Result<()> {
	let content = fs::read_to_string(config_path).unwrap_or_default();
	let mut config: Value = toml::from_str(&content).unwrap_or_else(|_| toml::Value::Table(toml::map::Map::new()));

	let Some(table) = config.as_table_mut() else {
		return Ok(());
	};

	let Ok(colors_value) = toml::Value::try_from(colors) else {
		return Ok(());
	};

	let colors_table = match colors_value.as_table() {
		Some(table) if !table.is_empty() => table,
		_ => return Ok(()),
	};

	table.insert("colors".to_string(), Value::Table(colors_table.clone()));

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
	let Some(config_home) = config_dir() else { return Ok(()) };

	let themes_path = config_home.join("alacritty_themes");
	if themes_path.exists() {
		return Ok(());
	}

	create_dir_all(&themes_path)?;
	for file in THEMES_DIR.files() {
		fs::write(themes_path.join(file.path()), file.contents())?;
	}

	Ok(())
}
