use crate::provider::{ProviderCapability, ProviderId};

#[derive(Debug, thiserror::Error)]
pub enum GitfleetError {
    #[error("{0}")]
    Auth(#[from] AuthError),
    #[error("{0}")]
    Config(#[from] ConfigError),
    #[error("{0}")]
    NotFound(#[from] NotFoundError),
    #[error("{0}")]
    Unprocessable(#[from] UnprocessableError),
    #[error("{0}")]
    RateLimit(#[from] RateLimitError),
    #[error("{0}")]
    TokenRequired(#[from] TokenRequiredError),
    #[error("{0}")]
    SecretEncryption(#[from] SecretEncryptionError),
    #[error("{0}")]
    UnsupportedCapability(#[from] UnsupportedCapabilityError),
    #[error("{0}")]
    Other(String),
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct AuthError {
    message: String,
}

impl AuthError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct ConfigError {
    message: String,
}

impl ConfigError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct NotFoundError {
    message: String,
}

impl NotFoundError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct UnprocessableError {
    message: String,
}

impl UnprocessableError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct RateLimitError {
    message: String,
    pub reset_at: time::OffsetDateTime,
    pub remaining: u32,
    pub limit: u32,
}

impl RateLimitError {
    pub fn new(
        message: impl Into<String>,
        reset_at: time::OffsetDateTime,
        remaining: u32,
        limit: u32,
    ) -> Self {
        Self {
            message: message.into(),
            reset_at,
            remaining,
            limit,
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct TokenRequiredError {
    message: String,
    pub scopes: Vec<String>,
}

impl TokenRequiredError {
    pub fn new(message: impl Into<String>, scopes: Vec<String>) -> Self {
        Self {
            message: message.into(),
            scopes,
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct SecretEncryptionError {
    message: String,
}

impl SecretEncryptionError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Provider \"{provider}\" does not support {capability}.")]
pub struct UnsupportedCapabilityError {
    provider: ProviderId,
    capability: ProviderCapability,
}

impl UnsupportedCapabilityError {
    pub fn new(provider: ProviderId, capability: ProviderCapability) -> Self {
        Self {
            provider,
            capability,
        }
    }
}

impl GitfleetError {
    pub fn new(message: impl Into<String>) -> Self {
        Self::Other(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::ProviderCapability;

    #[test]
    fn test_gitfleet_error_new() {
        let err = GitfleetError::new("something went wrong");

        assert_eq!(err.to_string(), "something went wrong");
    }

    #[test]
    fn test_auth_error_new() {
        let err = AuthError::new("bad token");

        assert_eq!(err.to_string(), "bad token");
    }

    #[test]
    fn test_config_error_new() {
        let err = ConfigError::new("bad config");

        assert_eq!(err.to_string(), "bad config");
    }

    #[test]
    fn test_not_found_error_new() {
        let err = NotFoundError::new("missing resource");

        assert_eq!(err.to_string(), "missing resource");
    }

    #[test]
    fn test_unprocessable_error_new() {
        let err = UnprocessableError::new("unprocessable content");

        assert_eq!(err.to_string(), "unprocessable content");
    }

    #[test]
    fn test_rate_limit_error_new() {
        let dt = time::OffsetDateTime::UNIX_EPOCH;
        let err = RateLimitError::new("rate limited", dt, 0, 5000);

        assert_eq!(err.to_string(), "rate limited");

        assert_eq!(err.remaining, 0);
        assert_eq!(err.limit, 5000);
    }

    #[test]
    fn test_token_required_error_new() {
        let err = TokenRequiredError::new("token needed", vec!["repo".into(), "read:org".into()]);

        assert_eq!(err.to_string(), "token needed");

        assert_eq!(err.scopes, vec!["repo", "read:org"]);
    }

    #[test]
    fn test_secret_encryption_error_new() {
        let err = SecretEncryptionError::new("encryption failed");

        assert_eq!(err.to_string(), "encryption failed");
    }

    #[test]
    fn test_unsupported_capability_error_new() {
        let err = UnsupportedCapabilityError::new(ProviderId::GitHub, ProviderCapability::Wiki);

        assert!(err.to_string().contains("github"));

        assert!(err.to_string().contains("wiki"));
    }

    #[test]
    fn test_auth_error_into_gitfleet_error() {
        let err: GitfleetError = AuthError::new("auth fail").into();

        assert_eq!(err.to_string(), "auth fail");
    }

    #[test]
    fn test_config_error_into_gitfleet_error() {
        let err: GitfleetError = ConfigError::new("config fail").into();

        assert_eq!(err.to_string(), "config fail");
    }

    #[test]
    fn test_not_found_error_into_gitfleet_error() {
        let err: GitfleetError = NotFoundError::new("not here").into();

        assert_eq!(err.to_string(), "not here");
    }

    #[test]
    fn test_unprocessable_error_into_gitfleet_error() {
        let err: GitfleetError = UnprocessableError::new("unprocessable").into();

        assert_eq!(err.to_string(), "unprocessable");
    }

    #[test]
    fn test_rate_limit_error_into_gitfleet_error() {
        let dt = time::OffsetDateTime::UNIX_EPOCH;
        let err: GitfleetError = RateLimitError::new("rate limited", dt, 10, 5000).into();

        assert_eq!(err.to_string(), "rate limited");
    }

    #[test]
    fn test_token_required_error_into_gitfleet_error() {
        let err: GitfleetError = TokenRequiredError::new("need token", vec![]).into();

        assert_eq!(err.to_string(), "need token");
    }

    #[test]
    fn test_secret_encryption_error_into_gitfleet_error() {
        let err: GitfleetError = SecretEncryptionError::new("encrypt fail").into();

        assert_eq!(err.to_string(), "encrypt fail");
    }

    #[test]
    fn test_unsupported_capability_error_into_gitfleet_error() {
        let err: GitfleetError =
            UnsupportedCapabilityError::new(ProviderId::GitHub, ProviderCapability::Wiki).into();

        let msg = err.to_string();

        assert!(!msg.is_empty());
    }

    #[test]
    fn test_error_display_preserves_message() {
        let msg = "detailed error info: code=500";
        let err = GitfleetError::new(msg);

        assert_eq!(format!("{err}"), msg);
    }

    #[test]
    fn test_rate_limit_error_fields() {
        let dt = time::OffsetDateTime::UNIX_EPOCH;
        let err = RateLimitError::new("limited", dt, 42, 100);

        assert_eq!(err.remaining, 42);

        assert_eq!(err.limit, 100);
        assert_eq!(err.reset_at, dt);
    }

    #[test]
    fn test_token_required_error_empty_scopes() {
        let err = TokenRequiredError::new("no scopes", vec![]);

        assert!(err.scopes.is_empty());
    }

    #[test]
    fn test_token_required_error_multiple_scopes() {
        let scopes = vec!["a".into(), "b".into(), "c".into()];
        let err = TokenRequiredError::new("scopes", scopes.clone());

        assert_eq!(err.scopes, scopes);
    }

    #[test]
    fn test_gitfleet_error_variant_auth() {
        let err = GitfleetError::Auth(AuthError::new("auth"));

        match err {
            GitfleetError::Auth(e) => assert_eq!(e.to_string(), "auth"),
            _ => panic!("expected Auth variant"),
        }
    }

    #[test]
    fn test_gitfleet_error_variant_config() {
        let err = GitfleetError::Config(ConfigError::new("config"));

        match err {
            GitfleetError::Config(e) => assert_eq!(e.to_string(), "config"),
            _ => panic!("expected Config variant"),
        }
    }

    #[test]
    fn test_gitfleet_error_variant_not_found() {
        let err = GitfleetError::NotFound(NotFoundError::new("not found"));

        match err {
            GitfleetError::NotFound(e) => assert_eq!(e.to_string(), "not found"),
            _ => panic!("expected NotFound variant"),
        }
    }

    #[test]
    fn test_gitfleet_error_variant_other() {
        let err = GitfleetError::new("generic");

        match err {
            GitfleetError::Other(msg) => assert_eq!(msg, "generic"),
            _ => panic!("expected Other variant"),
        }
    }
}
