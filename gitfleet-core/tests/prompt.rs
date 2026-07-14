use gitfleet_core::errors::GitfleetError;
use gitfleet_core::output_state::OutputMode;
use gitfleet_core::prompt::{confirm_destructive, guard_non_interactive};

#[test]
#[serial_test::serial]
fn test_guard_non_interactive_json_mode() {
    let result = guard_non_interactive("Operation not allowed in JSON mode", OutputMode::Json);

    assert!(result.is_err());

    let err = result.unwrap_err();

    assert_eq!(err.to_string(), "Operation not allowed in JSON mode");
}

#[test]
#[serial_test::serial]
fn test_guard_non_interactive_silent_mode() {
    let result = guard_non_interactive("Not allowed", OutputMode::Silent);

    assert!(result.is_err());
}

#[test]
#[serial_test::serial]
fn test_guard_non_interactive_human_mode_without_ci() {
    std::env::remove_var("GITFLEET_CI");

    let result = guard_non_interactive("test", OutputMode::Human);

    assert!(result.is_ok());
}

#[test]
#[serial_test::serial]
fn test_guard_non_interactive_human_mode_with_ci() {
    std::env::set_var("GITFLEET_CI", "true");

    let result = guard_non_interactive("test", OutputMode::Human);

    assert!(result.is_err());

    std::env::remove_var("GITFLEET_CI");
}

#[test]
#[serial_test::serial]
fn test_confirm_destructive_yes_flag() {
    let result = confirm_destructive("Delete everything?", OutputMode::Human, true);

    assert!(result.is_ok());
}

#[test]
#[serial_test::serial]
fn test_confirm_destructive_json_mode_requires_yes() {
    let result = confirm_destructive("Delete?", OutputMode::Json, false);

    assert!(result.is_err());
}

#[test]
#[serial_test::serial]
fn test_confirm_destructive_silent_mode_requires_yes() {
    let result = confirm_destructive("Delete?", OutputMode::Silent, false);

    assert!(result.is_err());
}

#[test]
#[serial_test::serial]
fn test_confirm_destructive_json_mode_with_yes() {
    let result = confirm_destructive("Delete?", OutputMode::Json, true);

    assert!(result.is_ok());
}

#[test]
#[serial_test::serial]
fn test_confirm_destructive_silent_mode_with_yes() {
    let result = confirm_destructive("Delete?", OutputMode::Silent, true);

    assert!(result.is_ok());
}

#[test]
#[serial_test::serial]
fn test_confirm_destructive_ci_env_fails() {
    std::env::set_var("GITFLEET_CI", "true");

    let result = confirm_destructive("Delete?", OutputMode::Human, false);

    assert!(result.is_err());

    std::env::remove_var("GITFLEET_CI");
}

#[test]
#[serial_test::serial]
fn test_prompt_if_missing_returns_value() {
    std::env::remove_var("GITFLEET_CI");

    let result =
        gitfleet_core::prompt::prompt_if_missing(Some("hello"), "Enter value:", OutputMode::Json);
    assert_eq!(result.unwrap(), "hello");
}

#[test]
#[serial_test::serial]
fn test_prompt_if_missing_errors_json() {
    let result =
        gitfleet_core::prompt::prompt_if_missing(None::<&str>, "Enter value:", OutputMode::Json);
    assert!(result.is_err());
}

#[test]
#[serial_test::serial]
fn test_prompt_error_message_format() {
    let err = GitfleetError::new("Required option not provided: repo");

    assert!(err
        .to_string()
        .contains("Required option not provided: repo"));
}
