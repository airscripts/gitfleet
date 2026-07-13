use gitfleet_core::constants::*;
use gitfleet_core::provider::ProviderId;

#[test]
fn test_constants_are_valid() {
    assert!(!GITFLEET_FOLDER.is_empty());

    assert!(!CREDENTIALS_FILE.is_empty());
    assert!(!METADATA_FILE.is_empty());

    assert!(!GITFLEET_RC_FILE.is_empty());
    assert!(!DEFAULT_PROFILE_NAME.is_empty());

    assert!(!GITHUB_API_VERSION.is_empty());
    assert!(!GITHUB_API_BASE_URL.is_empty());

    assert!(!GITHUB_API_ACCEPT.is_empty());
    assert!(!GITLAB_API_BASE_URL.is_empty());

    assert!(!GITFLEET_PROFILE_ENV.is_empty());
}

#[test]
fn test_status_code_ranges() {
    assert!((200..=299).contains(&STATUS_OK_MIN));

    assert!((200..=299).contains(&STATUS_OK_MAX));
    assert_eq!(STATUS_UNAUTHORIZED, 401);

    assert_eq!(STATUS_FORBIDDEN, 403);
    assert_eq!(STATUS_NOT_FOUND, 404);

    assert_eq!(STATUS_UNPROCESSABLE, 422);
    assert_eq!(STATUS_RATE_LIMITED, 429);
}

#[test]
fn test_rate_limits() {
    const _: () = assert!(RATE_LIMIT_AUTHENTICATED > RATE_LIMIT_UNAUTHENTICATED);
    const _: () = assert!(RATE_LIMIT_UNAUTHENTICATED > 0);
    const _: () = assert!(RATE_LIMIT_AUTHENTICATED > 0);
}

#[test]
fn test_error_messages_not_empty() {
    assert!(!ERROR_UNAUTHORIZED.is_empty());

    assert!(!ERROR_NOT_FOUND.is_empty());
    assert!(!ERROR_UNPROCESSABLE.is_empty());

    assert!(!ERROR_UNEXPECTED.is_empty());
    assert!(!ERROR_INVALID_CREDENTIALS.is_empty());

    assert!(!ERROR_NO_TOKEN.is_empty());
    assert!(!ERROR_NO_REPO.is_empty());

    assert!(!ERROR_AUTH_NO_TOKEN.is_empty());
    assert!(!ERROR_AUTH_FAILED.is_empty());
}

#[test]
fn test_default_per_page() {
    const _: () = assert!(DEFAULT_PER_PAGE > 0);
}

#[test]
fn test_search_limits() {
    const _: () = assert!(SEARCH_DEFAULT_LIMIT > 0);
    const _: () = assert!(SEARCH_MAX_PER_PAGE >= SEARCH_DEFAULT_LIMIT);
}

#[test]
fn test_host_providers() {
    assert!(HOST_PROVIDERS
        .iter()
        .any(|(h, p)| *h == "github.com" && *p == ProviderId::GitHub));
    assert!(HOST_PROVIDERS
        .iter()
        .any(|(h, p)| *h == "gitlab.com" && *p == ProviderId::GitLab));
}

#[test]
fn test_supported_config_keys() {
    assert!(!SUPPORTED_CONFIG_KEYS.is_empty());

    assert!(SUPPORTED_CONFIG_KEYS.contains(&"editor"));
    assert!(SUPPORTED_CONFIG_KEYS.contains(&"pager"));
}

#[test]
fn test_dependabot_dismiss_reasons() {
    assert!(!DEPENDABOT_DISMISS_REASONS.is_empty());

    assert!(DEPENDABOT_DISMISS_REASONS.contains(&"fix_started"));
    assert!(DEPENDABOT_DISMISS_REASONS.contains(&"not_used"));
}
