use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Colors {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub bright: Option<ColorScheme>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub cursor: Option<CursorColors>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub normal: Option<ColorScheme>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub primary: Option<PrimaryColors>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CursorColors {
	#[serde(skip_serializing_if = "Option::is_none")]
	cursor: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	text: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct ColorScheme {
	#[serde(skip_serializing_if = "Option::is_none")]
	black: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	blue: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	cyan: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	green: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	magenta: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	red: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	white: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	yellow: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PrimaryColors {
	#[serde(skip_serializing_if = "Option::is_none")]
	background: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	foreground: Option<String>,
}

pub fn merge_colors(default: &Colors, custom: &Colors) -> Colors {
	Colors {
		bright: match (&custom.bright, &default.bright) {
			(Some(c), Some(d)) => Some(merge_color_scheme(c, d)),
			(Some(c), None) => Some(c.clone()),
			(None, Some(d)) => Some(d.clone()),
			(None, None) => None,
		},
		cursor: match (&custom.cursor, &default.cursor) {
			(Some(c), Some(d)) => Some(CursorColors {
				cursor: c.cursor.clone().or_else(|| d.cursor.clone()),
				text: c.text.clone().or_else(|| d.text.clone()),
			}),
			(Some(c), None) => Some(c.clone()),
			(None, Some(d)) => Some(d.clone()),
			(None, None) => None,
		},
		normal: match (&custom.normal, &default.normal) {
			(Some(c), Some(d)) => Some(merge_color_scheme(c, d)),
			(Some(c), None) => Some(c.clone()),
			(None, Some(d)) => Some(d.clone()),
			(None, None) => None,
		},
		primary: match (&custom.primary, &default.primary) {
			(Some(c), Some(d)) => Some(PrimaryColors {
				background: c.background.clone().or_else(|| d.background.clone()),
				foreground: c.foreground.clone().or_else(|| d.foreground.clone()),
			}),
			(Some(c), None) => Some(c.clone()),
			(None, Some(d)) => Some(d.clone()),
			(None, None) => None,
		},
	}
}

fn merge_color_scheme(custom: &ColorScheme, default: &ColorScheme) -> ColorScheme {
	ColorScheme {
		black: custom.black.clone().or_else(|| default.black.clone()),
		blue: custom.blue.clone().or_else(|| default.blue.clone()),
		cyan: custom.cyan.clone().or_else(|| default.cyan.clone()),
		green: custom.green.clone().or_else(|| default.green.clone()),
		magenta: custom.magenta.clone().or_else(|| default.magenta.clone()),
		red: custom.red.clone().or_else(|| default.red.clone()),
		white: custom.white.clone().or_else(|| default.white.clone()),
		yellow: custom.yellow.clone().or_else(|| default.yellow.clone()),
	}
}
