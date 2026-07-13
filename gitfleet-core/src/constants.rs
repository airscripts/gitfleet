use crate::provider::ProviderId;

pub const GITFLEET_FOLDER: &str = ".config/gitfleet";
pub const CREDENTIALS_FILE: &str = "credentials.toml";
pub const METADATA_FILE: &str = "labels.json";
pub const GITFLEET_RC_FILE: &str = ".gitfleetrc";
pub const DEFAULT_PROFILE_NAME: &str = "default";

pub const GITHUB_API_VERSION: &str = "2026-03-10";
pub const GITHUB_API_BASE_URL: &str = "https://api.github.com";
pub const GITHUB_API_ACCEPT: &str = "application/vnd.github+json";

pub const STATUS_OK_MIN: u16 = 200;
pub const STATUS_OK_MAX: u16 = 299;
pub const STATUS_UNAUTHORIZED: u16 = 401;
pub const STATUS_FORBIDDEN: u16 = 403;
pub const STATUS_NOT_FOUND: u16 = 404;
pub const STATUS_UNPROCESSABLE: u16 = 422;
pub const STATUS_RATE_LIMITED: u16 = 429;

pub const RATE_LIMIT_UNAUTHENTICATED: u32 = 60;
pub const RATE_LIMIT_AUTHENTICATED: u32 = 5000;

pub const ERROR_UNAUTHORIZED: &str = "Unauthorized.";
pub const ERROR_NOT_FOUND: &str = "Resource not found.";
pub const ERROR_UNPROCESSABLE: &str = "Content is unprocessable.";
pub const ERROR_UNEXPECTED: &str = "Unexpected status code.";
pub const ERROR_INVALID_CREDENTIALS: &str = "Invalid credentials file.";
pub const ERROR_INVALID_PROFILE_RC: &str = "Invalid profile config file.";
pub const ERROR_PROFILE_NOT_FOUND: &str = "Profile not found.";
pub const ERROR_AUTH_NO_TOKEN: &str = "No token found. Run: gitfleet auth login";
pub const ERROR_AUTH_FAILED: &str = "Authentication failed. Check your token.";
pub const ERROR_NO_GIT_ROOT: &str = "Git repository root not found.";
pub const ERROR_NO_REMOTE_URL: &str = "Unable to detect repository remote.";
pub const ERROR_NO_REPO: &str = "No repository specified. Use --repo namespace/repository or run inside a git repository with a supported remote.";
pub const ERROR_NO_TOKEN: &str = "Token not configured. Run: gitfleet auth login.";
pub const ERROR_RATE_LIMIT_UNAUTHENTICATED: &str =
    "Rate limit reached (60/hour). Run: gitfleet auth login for a higher authenticated limit.";
pub const ERROR_RATE_LIMIT_AUTHENTICATED: &str = "Rate limit reached.";
pub const ERROR_TOKEN_REQUIRED: &str = "This operation requires a token.";
pub const ERROR_UNSUPPORTED_KEY: &str = "Trying to set unsupported key.";
pub const ERROR_NO_METADATA: &str = "No metadata file found.";
pub const ERROR_NO_REPO_TARGET: &str = "No repository target provided.";
pub const ERROR_MUTATION_REQUIRES_YES: &str =
    "This operation changes repositories. Re-run with --yes to apply.";
pub const ERROR_RULESET_REQUIRED: &str = "Ruleset file is required.";
pub const ERROR_AUDIT_TARGET_REQUIRED: &str = "Either --org or --enterprise must be provided.";
pub const ERROR_DEPENDABOT_ALERT_REQUIRED: &str = "Dependabot alert number is required.";
pub const ERROR_DEPENDABOT_DISMISS_REASON_REQUIRED: &str = "Dependabot dismiss reason is required.";
pub const ERROR_INVALID_DEPENDABOT_DISMISS_REASON: &str = "Invalid Dependabot dismiss reason.";
pub const ERROR_LABEL_SOURCE_REQUIRED: &str = "Either --template or --metadata must be provided.";
pub const ERROR_SEARCH_QUERY_REQUIRED: &str = "Search query is required.";
pub const ERROR_CHANGE_NUMBER_REQUIRED: &str = "Change number is required.";
pub const ERROR_REVIEW_FILE_REQUIRED: &str = "File path is required for review comments.";
pub const ERROR_REVIEW_LINE_REQUIRED: &str = "Line number is required for review comments.";
pub const ERROR_REVIEW_BODY_REQUIRED: &str = "Comment body is required.";
pub const ERROR_SECRET_ENCRYPTION_FAILED: &str = "Failed to encrypt secret value.";
pub const ERROR_SECRET_NAME_REQUIRED: &str = "Secret name is required.";
pub const ERROR_SECRET_VALUE_REQUIRED: &str = "Secret value is required.";
pub const ERROR_VARIABLE_NAME_REQUIRED: &str = "Variable name is required.";
pub const ERROR_VARIABLE_VALUE_REQUIRED: &str = "Variable value is required.";
pub const ERROR_ENVIRONMENT_NAME_REQUIRED: &str = "Environment name is required.";
pub const ERROR_WORKFLOW_NOT_FOUND: &str = "No workflow files were found in .github/workflows.";
pub const ERROR_WORKFLOW_INVALID_YAML: &str = "Workflow file contains invalid YAML.";
pub const ERROR_RUN_ID_REQUIRED: &str = "Run id is required.";
pub const ERROR_CACHE_KEY_REQUIRED: &str = "Cache key is required.";
pub const ERROR_ALIAS_NOT_FOUND: &str = "Alias not found.";
pub const ERROR_ALIAS_EXISTS: &str = "Alias already exists. Use --force to overwrite.";
pub const ERROR_ALIAS_NAME_REQUIRED: &str = "Alias name is required.";
pub const ERROR_ALIAS_EXPANSION_REQUIRED: &str = "Alias expansion is required.";

pub const DEFAULT_PER_PAGE: u32 = 100;
pub const DEFAULT_OUTPUT_DIR: &str = ".gitfleet/pipelines";
pub const WORKFLOW_DEFAULT_DIR: &str = ".github/workflows";
pub const SEARCH_DEFAULT_LIMIT: u32 = 30;
pub const SEARCH_MAX_PER_PAGE: u32 = 100;

pub const GITFLEET_PROFILE_ENV: &str = "GITFLEET_PROFILE";
pub const GITFLEET_TRUST_REPO_CONFIG_ENV: &str = "GITFLEET_TRUST_REPO_CONFIG";
pub const GITFLEET_CREDENTIAL_STORE_ENV: &str = "GITFLEET_CREDENTIAL_STORE";
pub const GITFLEET_TEST_CREDENTIAL_STORE_ENV: &str = "GITFLEET_TEST_CREDENTIAL_STORE";
pub const KEYRING_SERVICE: &str = "gitfleet";

pub const MAX_PAGINATION_PAGES: u32 = 100;
pub const MAX_PAGINATION_ITEMS: usize = 10_000;
pub const HTTP_TIMEOUT_SECONDS: u64 = 30;
pub const MAX_HTTP_RESPONSE_BYTES: usize = 16 * 1024 * 1024;

pub const SUPPORTED_CONFIG_KEYS: &[&str] = &[
    "editor",
    "pager",
    "prefer_editor",
    "prompt",
    "git_protocol",
    "browser",
];

pub const DEPENDABOT_DISMISS_REASONS: &[&str] = &[
    "fix_started",
    "inaccurate",
    "no_bandwidth",
    "not_used",
    "tolerable_risk",
];

pub const GITLAB_API_BASE_URL: &str = "https://gitlab.com/api/v4";

pub const HOST_PROVIDERS: &[(&str, ProviderId)] = &[
    ("github.com", ProviderId::GitHub),
    ("gitlab.com", ProviderId::GitLab),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gitfleet_folder_non_empty() {
        assert!(!GITFLEET_FOLDER.is_empty());
    }

    #[test]
    fn test_credentials_file_non_empty() {
        assert!(!CREDENTIALS_FILE.is_empty());
    }

    #[test]
    fn test_metadata_file_non_empty() {
        assert!(!METADATA_FILE.is_empty());
    }

    #[test]
    fn test_gitfleet_rc_file_non_empty() {
        assert!(!GITFLEET_RC_FILE.is_empty());
    }

    #[test]
    fn test_default_profile_name_non_empty() {
        assert!(!DEFAULT_PROFILE_NAME.is_empty());
    }

    #[test]
    fn test_github_api_version_non_empty() {
        assert!(!GITHUB_API_VERSION.is_empty());
    }

    #[test]
    fn test_github_api_base_url_non_empty() {
        assert!(!GITHUB_API_BASE_URL.is_empty());
    }

    #[test]
    fn test_github_api_accept_non_empty() {
        assert!(!GITHUB_API_ACCEPT.is_empty());
    }

    #[test]
    fn test_status_codes_are_valid_http() {
        assert!((200..=299).contains(&STATUS_OK_MIN));

        assert!((200..=299).contains(&STATUS_OK_MAX));
        assert!((200..=599).contains(&STATUS_UNAUTHORIZED));

        assert!((200..=599).contains(&STATUS_FORBIDDEN));
        assert!((200..=599).contains(&STATUS_NOT_FOUND));

        assert!((200..=599).contains(&STATUS_UNPROCESSABLE));
        assert!((200..=599).contains(&STATUS_RATE_LIMITED));
    }

    #[test]
    fn test_status_ok_range() {
        assert_eq!(STATUS_OK_MIN, 200);

        assert_eq!(STATUS_OK_MAX, 299);
        const _: () = assert!(STATUS_OK_MIN <= STATUS_OK_MAX);
    }

    #[test]
    fn test_status_specific_values() {
        assert_eq!(STATUS_UNAUTHORIZED, 401);

        assert_eq!(STATUS_FORBIDDEN, 403);
        assert_eq!(STATUS_NOT_FOUND, 404);

        assert_eq!(STATUS_UNPROCESSABLE, 422);
        assert_eq!(STATUS_RATE_LIMITED, 429);
    }

    #[test]
    fn test_rate_limits_positive() {
        const _: () = assert!(RATE_LIMIT_UNAUTHENTICATED > 0);
        const _: () = assert!(RATE_LIMIT_AUTHENTICATED > 0);
        const _: () = assert!(RATE_LIMIT_AUTHENTICATED > RATE_LIMIT_UNAUTHENTICATED);
    }

    #[test]
    fn test_error_messages_non_empty() {
        assert!(!ERROR_UNAUTHORIZED.is_empty());

        assert!(!ERROR_NOT_FOUND.is_empty());
        assert!(!ERROR_UNPROCESSABLE.is_empty());

        assert!(!ERROR_UNEXPECTED.is_empty());
        assert!(!ERROR_INVALID_CREDENTIALS.is_empty());

        assert!(!ERROR_INVALID_PROFILE_RC.is_empty());
        assert!(!ERROR_PROFILE_NOT_FOUND.is_empty());

        assert!(!ERROR_AUTH_NO_TOKEN.is_empty());
        assert!(!ERROR_AUTH_FAILED.is_empty());

        assert!(!ERROR_NO_GIT_ROOT.is_empty());
        assert!(!ERROR_NO_REMOTE_URL.is_empty());

        assert!(!ERROR_NO_REPO.is_empty());
        assert!(!ERROR_NO_TOKEN.is_empty());

        assert!(!ERROR_RATE_LIMIT_UNAUTHENTICATED.is_empty());
        assert!(!ERROR_RATE_LIMIT_AUTHENTICATED.is_empty());

        assert!(!ERROR_TOKEN_REQUIRED.is_empty());
        assert!(!ERROR_UNSUPPORTED_KEY.is_empty());
    }

    #[test]
    fn test_default_per_page_positive() {
        const _: () = assert!(DEFAULT_PER_PAGE > 0);
    }

    #[test]
    fn test_search_limits() {
        const _: () = assert!(SEARCH_DEFAULT_LIMIT > 0);
        const _: () = assert!(SEARCH_MAX_PER_PAGE >= SEARCH_DEFAULT_LIMIT);
    }

    #[test]
    fn test_gitfleet_profile_env_non_empty() {
        assert!(!GITFLEET_PROFILE_ENV.is_empty());
    }

    #[test]
    fn test_supported_config_keys_non_empty() {
        assert!(!SUPPORTED_CONFIG_KEYS.is_empty());
    }

    #[test]
    fn test_dependabot_dismiss_reasons_non_empty() {
        assert!(!DEPENDABOT_DISMISS_REASONS.is_empty());
    }

    #[test]
    fn test_host_providers_contains_github() {
        assert!(HOST_PROVIDERS.iter().any(|(h, _)| *h == "github.com"));
    }

    #[test]
    fn test_host_providers_contains_gitlab() {
        assert!(HOST_PROVIDERS.iter().any(|(h, _)| *h == "gitlab.com"));
    }

    #[test]
    fn test_gitlab_api_base_url_non_empty() {
        assert!(!GITLAB_API_BASE_URL.is_empty());
    }
}
