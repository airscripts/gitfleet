use crate::errors::GitfleetError;
use crate::output_state::OutputMode;

use crate::constants::GITFLEET_CI_ENV;

pub fn guard_non_interactive(message: &str, mode: OutputMode) -> Result<(), GitfleetError> {
    if mode != OutputMode::Human {
        return Err(GitfleetError::new(message));
    }

    if std::env::var(GITFLEET_CI_ENV).is_ok() {
        return Err(GitfleetError::new(message));
    }

    Ok(())
}

pub fn confirm_destructive(
    message: &str,
    mode: OutputMode,
    yes: bool,
) -> Result<(), GitfleetError> {
    if yes {
        return Ok(());
    }

    if mode != OutputMode::Human {
        return Err(GitfleetError::new(format!(
            "{message} (use --yes to confirm)"
        )));
    }

    if std::env::var(GITFLEET_CI_ENV).is_ok() {
        return Err(GitfleetError::new(message));
    }

    use std::io::IsTerminal;

    if !std::io::stdin().is_terminal() {
        return Err(GitfleetError::new(format!(
            "{message} (use --yes to confirm)"
        )));
    }

    if prompt_confirm(message)? {
        Ok(())
    } else {
        Err(GitfleetError::new("Cancelled."))
    }
}

pub fn prompt_text(message: &str) -> Result<String, GitfleetError> {
    inquire::Text::new(message)
        .prompt()
        .map_err(|e| GitfleetError::new(format!("Prompt failed: {e}")))
}

pub fn prompt_password(message: &str) -> Result<String, GitfleetError> {
    inquire::Password::new(message)
        .without_confirmation()
        .prompt()
        .map_err(|e| GitfleetError::new(format!("Prompt failed: {e}")))
}

pub fn prompt_text_with_placeholder(
    message: &str,
    placeholder: &str,
) -> Result<String, GitfleetError> {
    inquire::Text::new(message)
        .with_placeholder(placeholder)
        .prompt()
        .map_err(|e| GitfleetError::new(format!("Prompt failed: {e}")))
}

pub fn prompt_confirm(message: &str) -> Result<bool, GitfleetError> {
    inquire::Confirm::new(message)
        .with_default(false)
        .prompt()
        .map_err(|e| GitfleetError::new(format!("Prompt failed: {e}")))
}

pub fn prompt_select(message: &str, options: &[String]) -> Result<String, GitfleetError> {
    guard_non_interactive("Selection requires interactive mode", OutputMode::Human)?;
    inquire::Select::new(message, options.to_vec())
        .prompt()
        .map_err(|e| GitfleetError::new(format!("Prompt failed: {e}")))
}

pub fn prompt_multi_select(
    message: &str,
    options: &[String],
) -> Result<Vec<String>, GitfleetError> {
    guard_non_interactive("Selection requires interactive mode", OutputMode::Human)?;
    inquire::MultiSelect::new(message, options.to_vec())
        .prompt()
        .map_err(|e| GitfleetError::new(format!("Prompt failed: {e}")))
}

pub fn prompt_if_missing(
    value: Option<&str>,
    message: &str,
    mode: OutputMode,
) -> Result<String, GitfleetError> {
    if let Some(v) = value {
        return Ok(v.to_string());
    }

    guard_non_interactive(&format!("Required option not provided: {message}"), mode)?;
    prompt_text(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[serial_test::serial]
    fn test_guard_non_interactive_json_mode_returns_error() {
        let result = guard_non_interactive("not allowed", OutputMode::Json);

        assert!(result.is_err());

        assert_eq!(result.unwrap_err().to_string(), "not allowed");
    }

    #[test]
    #[serial_test::serial]
    fn test_guard_non_interactive_silent_mode_returns_error() {
        let result = guard_non_interactive("not allowed", OutputMode::Silent);

        assert!(result.is_err());
    }

    #[test]
    #[serial_test::serial]
    fn test_guard_non_interactive_human_mode_ok_without_ci() {
        std::env::remove_var("GITFLEET_CI");

        let result = guard_non_interactive("test", OutputMode::Human);

        assert!(result.is_ok());
    }

    #[test]
    #[serial_test::serial]
    fn test_guard_non_interactive_human_mode_fails_with_ci() {
        std::env::set_var("GITFLEET_CI", "true");

        let result = guard_non_interactive("test", OutputMode::Human);

        assert!(result.is_err());

        std::env::remove_var("GITFLEET_CI");
    }

    #[test]
    #[serial_test::serial]
    fn test_prompt_if_missing_returns_value_when_present() {
        std::env::remove_var("GITFLEET_CI");

        let result = prompt_if_missing(Some("hello"), "prompt", OutputMode::Json);

        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    #[serial_test::serial]
    fn test_prompt_if_missing_returns_error_json_mode() {
        let result = prompt_if_missing(None::<&str>, "prompt", OutputMode::Json);

        assert!(result.is_err());
    }

    #[test]
    #[serial_test::serial]
    fn test_guard_non_interactive_error_message() {
        let err = guard_non_interactive("custom message", OutputMode::Json).unwrap_err();

        assert_eq!(err.to_string(), "custom message");
    }

    #[test]
    #[serial_test::serial]
    fn test_confirm_destructive_yes_flag_skips_prompt() {
        let result = confirm_destructive("Delete?", OutputMode::Human, true);

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
}
