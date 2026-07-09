use owo_colors::OwoColorize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Dark,
    Light,
    Auto,
}

impl Theme {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "light" => Theme::Light,
            "dark" => Theme::Dark,
            _ => Theme::Auto,
        }
    }

    pub fn resolve(self) -> ResolvedTheme {
        match self {
            Theme::Auto => {
                if detect_terminal_background() == Theme::Light {
                    ResolvedTheme::Light
                } else {
                    ResolvedTheme::Dark
                }
            }

            Theme::Dark => ResolvedTheme::Dark,
            Theme::Light => ResolvedTheme::Light,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedTheme {
    Dark,
    Light,
}

impl ResolvedTheme {
    pub fn is_dark(self) -> bool {
        matches!(self, ResolvedTheme::Dark)
    }
}

pub fn get_effective_theme(theme: Theme) -> Theme {
    match theme {
        Theme::Auto => detect_terminal_background(),
        other => other,
    }
}

fn detect_terminal_background() -> Theme {
    if let Ok(val) = std::env::var("COLORFGBG") {
        let parts: Vec<&str> = val.split(';').collect();

        if parts.len() >= 2 {
            if let Ok(bg) = parts[1].parse::<u8>() {
                if (7..=15).contains(&bg) {
                    return Theme::Light;
                }
            }
        }
    }

    if let Ok(ct) = std::env::var("COLORTERM") {
        if ct.contains("light") {
            return Theme::Light;
        }
    }

    if let Ok(term) = std::env::var("TERM") {
        if term.contains("light") {
            return Theme::Light;
        }
    }

    Theme::Dark
}

pub fn init_colors(theme: Theme) -> ResolvedTheme {
    use std::io::IsTerminal;

    if !std::io::stdout().is_terminal() || !std::io::stderr().is_terminal() {
        owo_colors::set_override(false);
    }

    if std::env::var("NO_COLOR").is_ok() {
        owo_colors::set_override(false);
    }

    theme.resolve()
}

pub struct Palette;

impl Default for Palette {
    fn default() -> Self {
        Self::new()
    }
}

impl Palette {
    pub fn new() -> Self {
        Self
    }

    pub fn primary(&self, text: &str) -> String {
        text.bold().to_string()
    }

    pub fn primary_bold(&self, text: &str) -> String {
        text.bold().to_string()
    }

    pub fn success(&self, text: &str) -> String {
        text.bold().to_string()
    }

    pub fn success_bold(&self, text: &str) -> String {
        text.bold().to_string()
    }

    pub fn error(&self, text: &str) -> String {
        text.bold().to_string()
    }

    pub fn error_bold(&self, text: &str) -> String {
        text.bold().to_string()
    }

    pub fn warning(&self, text: &str) -> String {
        text.bold().to_string()
    }

    pub fn warning_bold(&self, text: &str) -> String {
        text.bold().to_string()
    }

    pub fn info(&self, text: &str) -> String {
        text.bold().to_string()
    }

    pub fn muted(&self, text: &str) -> String {
        text.dimmed().to_string()
    }

    pub fn accent(&self, text: &str) -> String {
        text.bold().to_string()
    }

    pub fn accent_bold(&self, text: &str) -> String {
        text.bold().to_string()
    }

    pub fn dim(&self, text: &str) -> String {
        text.dimmed().to_string()
    }

    pub fn bold(&self, text: &str) -> String {
        text.bold().to_string()
    }

    pub fn underline(&self, text: &str) -> String {
        text.underline().to_string()
    }
}

pub fn info(text: &str) -> String {
    text.bold().to_string()
}

pub fn error(text: &str) -> String {
    text.bold().to_string()
}

pub fn muted(text: &str) -> String {
    text.dimmed().to_string()
}

pub fn primary(text: &str) -> String {
    text.bold().to_string()
}

pub fn success(text: &str) -> String {
    text.bold().to_string()
}

pub fn warning(text: &str) -> String {
    text.bold().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_effective_theme_dark() {
        let effective = get_effective_theme(Theme::Dark);

        assert_eq!(effective, Theme::Dark);
    }

    #[test]
    fn test_get_effective_theme_light() {
        let effective = get_effective_theme(Theme::Light);

        assert_eq!(effective, Theme::Light);
    }

    #[test]
    fn test_theme_parse_dark() {
        assert_eq!(Theme::parse("dark"), Theme::Dark);
    }

    #[test]
    fn test_theme_parse_light() {
        assert_eq!(Theme::parse("light"), Theme::Light);
    }

    #[test]
    fn test_theme_parse_auto() {
        assert_eq!(Theme::parse("auto"), Theme::Auto);
    }

    #[test]
    fn test_theme_parse_case_insensitive() {
        assert_eq!(Theme::parse("Dark"), Theme::Dark);

        assert_eq!(Theme::parse("LIGHT"), Theme::Light);
        assert_eq!(Theme::parse("Auto"), Theme::Auto);
    }

    #[test]
    fn test_theme_parse_unknown_defaults_auto() {
        assert_eq!(Theme::parse("unknown"), Theme::Auto);

        assert_eq!(Theme::parse(""), Theme::Auto);
    }

    #[test]
    fn test_theme_resolve_dark() {
        assert_eq!(Theme::Dark.resolve(), ResolvedTheme::Dark);
    }

    #[test]
    fn test_theme_resolve_light() {
        assert_eq!(Theme::Light.resolve(), ResolvedTheme::Light);
    }

    #[test]
    fn test_resolved_theme_is_dark() {
        assert!(ResolvedTheme::Dark.is_dark());

        assert!(!ResolvedTheme::Light.is_dark());
    }

    #[test]
    fn test_palette_dark_primary() {
        let p = Palette::new();

        assert!(!p.primary("test").is_empty());
    }

    #[test]
    fn test_palette_light_primary() {
        let p = Palette::new();

        assert!(!p.primary("test").is_empty());
    }

    #[test]
    fn test_palette_success() {
        let p = Palette::new();

        assert!(!p.success("test").is_empty());
    }

    #[test]
    fn test_palette_error() {
        let p = Palette::new();

        assert!(!p.error("test").is_empty());
    }

    #[test]
    fn test_palette_warning() {
        let p = Palette::new();

        assert!(!p.warning("test").is_empty());
    }

    #[test]
    fn test_palette_muted() {
        let p = Palette::new();

        assert!(!p.muted("test").is_empty());
    }

    #[test]
    fn test_palette_accent() {
        let p = Palette::new();

        assert!(!p.accent("test").is_empty());
    }
}
