use gitfleet_core::theme::{get_effective_theme, Theme};

#[test]
fn test_theme_dark() {
    assert_eq!(get_effective_theme(Theme::Dark), Theme::Dark);
}

#[test]
fn test_theme_light() {
    assert_eq!(get_effective_theme(Theme::Light), Theme::Light);
}

#[test]
fn test_theme_auto_returns_valid_theme() {
    let theme = get_effective_theme(Theme::Auto);

    assert!(theme == Theme::Dark || theme == Theme::Light);
}

#[test]
fn test_theme_equality() {
    assert_eq!(Theme::Dark, Theme::Dark);

    assert_eq!(Theme::Light, Theme::Light);
    assert_eq!(Theme::Auto, Theme::Auto);

    assert_ne!(Theme::Dark, Theme::Light);
}

#[test]
fn test_theme_debug_format() {
    assert_eq!(format!("{:?}", Theme::Dark), "Dark");

    assert_eq!(format!("{:?}", Theme::Light), "Light");
    assert_eq!(format!("{:?}", Theme::Auto), "Auto");
}

#[test]
fn test_theme_color_functions() {
    let info = gitfleet_core::theme::info("info");

    assert!(!info.is_empty());

    let error = gitfleet_core::theme::error("error");

    assert!(!error.is_empty());

    let muted = gitfleet_core::theme::muted("muted");

    assert!(!muted.is_empty());

    let primary = gitfleet_core::theme::primary("primary");

    assert!(!primary.is_empty());

    let success = gitfleet_core::theme::success("success");

    assert!(!success.is_empty());

    let warning = gitfleet_core::theme::warning("warning");

    assert!(!warning.is_empty());
}
