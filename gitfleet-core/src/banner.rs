use crate::theme::Palette;

pub const BANNER_LINES: &[&str] = &[
    " ██████╗ ██╗████████╗███████╗██╗     ███████╗███████╗████████╗",
    "██╔════╝ ██║╚══██╔══╝██╔════╝██║     ██╔════╝██╔════╝╚══██╔══╝",
    "██║  ███╗██║   ██║   █████╗  ██║     █████╗  █████╗     ██║   ",
    "██║   ██║██║   ██║   ██╔══╝  ██║     ██╔══╝  ██╔══╝     ██║   ",
    "╚██████╔╝██║   ██║   ██║     ███████╗███████╗███████╗   ██║   ",
    " ╚═════╝ ╚═╝   ╚═╝   ╚═╝     ╚══════╝╚══════╝╚══════╝   ╚═╝   ",
];

pub const TAGLINE: &str = "Command every repository as one fleet.";

pub fn banner() -> String {
    BANNER_LINES.join("\n")
}

pub fn colored_banner(palette: &Palette) -> String {
    BANNER_LINES
        .iter()
        .map(|line| palette.accent(line))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn help_header(palette: &Palette) -> String {
    format!("{}\n{}\n", colored_banner(palette), palette.muted(TAGLINE),)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Palette;

    #[test]
    fn test_banner_lines_not_empty() {
        assert!(!BANNER_LINES.is_empty());

        assert_eq!(BANNER_LINES.len(), 6);
    }

    #[test]
    fn test_banner_joined() {
        let b = banner();

        assert!(b.contains("████"));

        assert!(b.contains('\n'));
    }

    #[test]
    fn test_colored_banner_not_empty() {
        let p = Palette::new();

        let b = colored_banner(&p);

        assert!(!b.is_empty());

        assert!(b.contains('\n'));
    }

    #[test]
    fn test_help_header_contains_tagline() {
        let p = Palette::new();

        let h = help_header(&p);

        assert!(h.contains(TAGLINE));
    }

    #[test]
    fn test_tagline_not_empty() {
        assert!(!TAGLINE.is_empty());
    }
}
