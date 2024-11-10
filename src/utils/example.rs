use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::Paragraph;

#[allow(clippy::too_many_lines)]
pub fn return_example() -> Paragraph<'static> {
	Paragraph::new(vec![
		// First line
		Spans::from(vec![
			Span::styled("use", Style::default().fg(Color::Red)),
			Span::raw(" std::mem::transmute;"),
		]),
		// Empty line
		Spans::from(vec![Span::raw("")]),
		// Function declaration
		Spans::from(vec![
			Span::styled("fn", Style::default().fg(Color::LightBlue)),
			Span::raw(" "),
			Span::styled("q_rsqrt", Style::default().fg(Color::LightGreen)),
			Span::raw("("),
			Span::styled("number", Style::default().fg(Color::Yellow)),
			Span::raw(": "),
			Span::styled("f32", Style::default().fg(Color::LightBlue)),
			Span::raw(") -> "),
			Span::styled("f32", Style::default().fg(Color::LightBlue)),
			Span::raw(" {"),
		]),
		// Variable declarations
		Spans::from(vec![
			Span::raw("    "),
			Span::styled("let", Style::default().fg(Color::LightBlue)),
			Span::raw(" "),
			Span::styled("mut", Style::default().fg(Color::Red)),
			Span::raw(" i: "),
			Span::styled("i32", Style::default().fg(Color::LightBlue)),
			Span::raw(";"),
		]),
		Spans::from(vec![
			Span::raw("    "),
			Span::styled("let", Style::default().fg(Color::LightBlue)),
			Span::raw(" "),
			Span::styled("mut", Style::default().fg(Color::Red)),
			Span::raw(" x2: "),
			Span::styled("f32", Style::default().fg(Color::LightBlue)),
			Span::raw(";"),
		]),
		Spans::from(vec![
			Span::raw("    "),
			Span::styled("let", Style::default().fg(Color::LightBlue)),
			Span::raw(" "),
			Span::styled("mut", Style::default().fg(Color::Red)),
			Span::raw(" y: "),
			Span::styled("f32", Style::default().fg(Color::LightBlue)),
			Span::raw(";"),
		]),
		Spans::from(vec![
			Span::raw("    "),
			Span::styled("const", Style::default().fg(Color::LightBlue)),
			Span::raw(" "),
			Span::styled("THREEHALFS", Style::default().fg(Color::Magenta)),
			Span::raw(": "),
			Span::styled("f32", Style::default().fg(Color::LightBlue)),
			Span::raw(" "),
			Span::styled("=", Style::default().fg(Color::Red)),
			Span::raw(" "),
			Span::styled("1.5", Style::default().fg(Color::Magenta)),
			Span::raw(";"),
		]),
		// Empty line
		Spans::from(vec![Span::raw("")]),
		// Assignments
		Spans::from(vec![
			Span::raw("    x2 "),
			Span::styled("=", Style::default().fg(Color::Red)),
			Span::raw(" number "),
			Span::styled("*", Style::default().fg(Color::Red)),
			Span::raw(" "),
			Span::styled("0.5", Style::default().fg(Color::Magenta)),
			Span::raw(";"),
		]),
		Spans::from(vec![
			Span::raw("    y "),
			Span::styled("=", Style::default().fg(Color::Red)),
			Span::raw(" number;"),
		]),
		// Comment
		Spans::from(vec![
			Span::raw("    "),
			Span::styled(
				"// evil floating point bit level hacking",
				Style::default().fg(Color::DarkGray),
			),
		]),
		// First unsafe block
		Spans::from(vec![
			Span::raw("    "),
			Span::styled("unsafe", Style::default().fg(Color::Red)),
			Span::raw(" {"),
		]),
		Spans::from(vec![
			Span::raw("        i "),
			Span::styled("=", Style::default().fg(Color::Red)),
			Span::raw(" transmute::<"),
			Span::styled("f32", Style::default().fg(Color::LightBlue)),
			Span::raw(", "),
			Span::styled("i32", Style::default().fg(Color::LightBlue)),
			Span::raw(">(y);"),
		]),
		Spans::from(vec![Span::raw("    }")]),
		// Comment
		Spans::from(vec![
			Span::raw("    "),
			Span::styled("// what the fuck?", Style::default().fg(Color::DarkGray)),
		]),
		// Bit manipulation
		Spans::from(vec![
			Span::raw("    i "),
			Span::styled("=", Style::default().fg(Color::Red)),
			Span::raw(" "),
			Span::styled("0x5f3759df", Style::default().fg(Color::Magenta)),
			Span::raw(" "),
			Span::styled("-", Style::default().fg(Color::Red)),
			Span::raw(" (i "),
			Span::styled(">>", Style::default().fg(Color::Red)),
			Span::raw(" "),
			Span::styled("1", Style::default().fg(Color::Magenta)),
			Span::raw(");"),
		]),
		// Second unsafe block
		Spans::from(vec![
			Span::raw("    "),
			Span::styled("unsafe", Style::default().fg(Color::Red)),
			Span::raw(" {"),
		]),
		Spans::from(vec![
			Span::raw("        y "),
			Span::styled("=", Style::default().fg(Color::Red)),
			Span::raw(" transmute::<"),
			Span::styled("i32", Style::default().fg(Color::LightBlue)),
			Span::raw(", "),
			Span::styled("f32", Style::default().fg(Color::LightBlue)),
			Span::raw(">(i);"),
		]),
		Spans::from(vec![Span::raw("    }")]),
		// Comment
		Spans::from(vec![
			Span::raw("    "),
			Span::styled("// 1st iteration", Style::default().fg(Color::DarkGray)),
		]),
		// Final calculation
		Spans::from(vec![
			Span::raw("    y "),
			Span::styled("=", Style::default().fg(Color::Red)),
			Span::raw(" y "),
			Span::styled("*", Style::default().fg(Color::Red)),
			Span::raw(" ("),
			Span::styled("THREEHALFS", Style::default().fg(Color::Magenta)),
			Span::raw(" "),
			Span::styled("-", Style::default().fg(Color::Red)),
			Span::raw(" (x2 "),
			Span::styled("*", Style::default().fg(Color::Red)),
			Span::raw(" y "),
			Span::styled("*", Style::default().fg(Color::Red)),
			Span::raw(" y));"),
		]),
		// Empty line
		Spans::from(vec![Span::raw("")]),
		// Return
		Spans::from(vec![Span::raw("    y")]),
		// Closing brace
		Spans::from(vec![Span::raw("}")]),
	])
}
