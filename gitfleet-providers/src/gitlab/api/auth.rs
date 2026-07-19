use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::{AuthStatus, AuthUser};

use crate::gitlab::client::ProviderClient;

pub struct AuthApi;

impl AuthApi {
    pub async fn fetch_authenticated_user(
        client: &ProviderClient,
    ) -> Result<AuthStatus, GitfleetError> {
        let response = client
            .request_token_required(reqwest::Method::GET, "/user", None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to parse authenticated user: {e}")))?;

        Ok(normalize_user(&data))
    }
}

fn normalize_user(data: &serde_json::Value) -> AuthStatus {
    let user = AuthUser {
        login: data
            .get("username")
            .or_else(|| data.get("login"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        html_url: data
            .get("web_url")
            .or_else(|| data.get("html_url"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        avatar_url: data
            .get("avatar_url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        name: data
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    };

    AuthStatus {
        user,
        scopes: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_user_full() {
        let json = serde_json::json!({
            "username": "tanuki",
            "name": "The Tanuki",
            "web_url": "https://gitlab.com/tanuki",
            "avatar_url": "https://gitlab.com/uploads/avatar.png"
        });

        let result = normalize_user(&json);

        assert_eq!(result.user.login, "tanuki");
        assert_eq!(result.user.name.as_deref(), Some("The Tanuki"));
        assert_eq!(result.user.html_url, "https://gitlab.com/tanuki");
        assert_eq!(
            result.user.avatar_url,
            "https://gitlab.com/uploads/avatar.png"
        );
        assert!(result.scopes.is_empty());
    }

    #[test]
    fn test_normalize_user_minimal() {
        let json = serde_json::json!({ "username": "bot" });

        let result = normalize_user(&json);

        assert_eq!(result.user.login, "bot");
        assert_eq!(result.user.html_url, "");
        assert!(result.user.name.is_none());
    }
}
