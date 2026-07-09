use gitfleet_core::errors::*;
use gitfleet_core::provider::{ProviderCapability, ProviderId};
use gitfleet_core::repository::RepositoryRef;

#[test]
fn test_gitfleet_error_variants_match_inner_messages() {
    let auth = GitfleetError::Auth(AuthError::new("auth fail"));

    assert_eq!(auth.to_string(), "auth fail");

    let config = GitfleetError::Config(ConfigError::new("config fail"));

    assert_eq!(config.to_string(), "config fail");

    let not_found = GitfleetError::NotFound(NotFoundError::new("missing"));

    assert_eq!(not_found.to_string(), "missing");

    let unprocessable = GitfleetError::Unprocessable(UnprocessableError::new("bad data"));

    assert_eq!(unprocessable.to_string(), "bad data");

    let rate_limit = GitfleetError::RateLimit(RateLimitError::new(
        "limited",
        time::OffsetDateTime::UNIX_EPOCH,
        0,
        5000,
    ));
    assert_eq!(rate_limit.to_string(), "limited");

    let token_required =
        GitfleetError::TokenRequired(TokenRequiredError::new("need token", vec!["repo".into()]));
    assert_eq!(token_required.to_string(), "need token");

    let secret = GitfleetError::SecretEncryption(SecretEncryptionError::new("encrypt fail"));

    assert_eq!(secret.to_string(), "encrypt fail");
}

#[test]
fn test_unsupported_capability_error_message() {
    let err = UnsupportedCapabilityError::new(ProviderId::GitHub, ProviderCapability::Wiki);

    let msg = err.to_string();

    assert!(msg.contains("github"));

    assert!(msg.contains("wiki"));

    let wrapped: GitfleetError = err.into();

    assert!(wrapped.to_string().contains("github"));
}

#[test]
fn test_rate_limit_error_fields() {
    let dt = time::OffsetDateTime::UNIX_EPOCH;
    let err = RateLimitError::new("rate limited", dt, 42, 5000);

    assert_eq!(err.remaining, 42);

    assert_eq!(err.limit, 5000);
    assert_eq!(err.reset_at, dt);
}

#[test]
fn test_token_required_error_scopes() {
    let err = TokenRequiredError::new("need scopes", vec!["repo".into(), "read:org".into()]);

    assert_eq!(err.scopes, vec!["repo", "read:org"]);

    assert_eq!(err.scopes.len(), 2);
}

#[test]
fn test_error_other_variant() {
    let err = GitfleetError::new("generic error");

    assert_eq!(err.to_string(), "generic error");
}

#[test]
fn test_provider_id_display() {
    assert_eq!(format!("{}", ProviderId::GitHub), "github");

    assert_eq!(format!("{}", ProviderId::GitLab), "gitlab");
}

#[test]
fn test_provider_capability_display_all() {
    let caps = [
        ProviderCapability::Repositories,
        ProviderCapability::Changes,
        ProviderCapability::Issues,
        ProviderCapability::Pipelines,
        ProviderCapability::Wiki,
        ProviderCapability::RawApi,
    ];

    for cap in &caps {
        let s = format!("{}", cap);

        assert!(!s.is_empty());
    }
}

#[test]
fn test_repository_ref_full_name() {
    let repo = RepositoryRef {
        provider: ProviderId::GitHub,
        host: "github.com".to_string(),
        namespace: "airscripts".to_string(),
        name: "gitfleet".to_string(),
    };

    assert_eq!(repo.full_name(), "airscripts/gitfleet");
}

#[test]
fn test_repository_ref_qualified() {
    let repo = RepositoryRef {
        provider: ProviderId::GitHub,
        host: "github.com".to_string(),
        namespace: "airscripts".to_string(),
        name: "gitfleet".to_string(),
    };

    assert_eq!(repo.qualified(), "github@github.com:airscripts/gitfleet");
}

#[test]
fn test_repository_ref_serialization() {
    let repo = RepositoryRef {
        provider: ProviderId::GitHub,
        host: "github.com".to_string(),
        namespace: "airscripts".to_string(),
        name: "gitfleet".to_string(),
    };

    let json = serde_json::to_string(&repo).unwrap();

    let deserialized: RepositoryRef = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.namespace, "airscripts");

    assert_eq!(deserialized.name, "gitfleet");
    assert_eq!(deserialized.host, "github.com");
}

#[test]
fn test_account_ref_serialization() {
    use gitfleet_core::repository::AccountRef;
    let account = AccountRef {
        provider: ProviderId::GitLab,
        host: "gitlab.com".to_string(),
        profile: "work".to_string(),
    };

    let json = serde_json::to_string(&account).unwrap();

    let deserialized: AccountRef = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.profile, "work");

    assert_eq!(deserialized.host, "gitlab.com");
}

#[test]
fn test_repository_ref_from_remote_https() {
    let result = gitfleet_core::repository::repository_ref_from_remote(
        "https://github.com/airscripts/gitfleet.git",
    );

    assert!(result.is_ok());

    let repo = result.unwrap();

    assert_eq!(repo.provider, ProviderId::GitHub);

    assert_eq!(repo.host, "github.com");
    assert_eq!(repo.namespace, "airscripts");

    assert_eq!(repo.name, "gitfleet");
}

#[test]
fn test_repository_ref_from_remote_scp() {
    let result =
        gitfleet_core::repository::repository_ref_from_remote("git@github.com:airscripts/gitfleet");
    assert!(result.is_ok());

    let repo = result.unwrap();

    assert_eq!(repo.provider, ProviderId::GitHub);

    assert_eq!(repo.namespace, "airscripts");
    assert_eq!(repo.name, "gitfleet");
}

#[test]
fn test_repository_ref_from_remote_invalid() {
    let result = gitfleet_core::repository::repository_ref_from_remote("invalid");

    assert!(result.is_err());
}

#[test]
fn test_repository_ref_from_remote_nested_namespace() {
    let result = gitfleet_core::repository::repository_ref_from_remote(
        "https://github.com/org/subgroup/repo.git",
    );

    assert!(result.is_ok());

    let repo = result.unwrap();

    assert_eq!(repo.namespace, "org/subgroup");

    assert_eq!(repo.name, "repo");
}
