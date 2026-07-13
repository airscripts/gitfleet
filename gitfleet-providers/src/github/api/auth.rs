use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::AuthStatus;

use crate::github::client::ProviderClient;

pub struct AuthApi;

impl AuthApi {
    pub async fn fetch_authenticated_user(
        client: &ProviderClient,
        token: Option<&str>,
        host: Option<&str>,
    ) -> Result<AuthStatus, GitfleetError> {
        let response = client
            .request_token_required(reqwest::Method::GET, "/user", None, token, host)
            .await?;

        let scopes_header = response
            .headers()
            .get("x-oauth-scopes")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let scopes: Vec<String> = if scopes_header.is_empty() {
            vec![]
        } else {
            scopes_header
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        };

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to parse authenticated user: {e}")))?;

        let user = gitfleet_core::types::AuthUser {
            login: data
                .get("login")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            html_url: data
                .get("html_url")
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

        Ok(AuthStatus { user, scopes })
    }
}

#[cfg(test)]
mod tests {
    use gitfleet_core::types::{AuthStatus, AuthUser};

    fn normalize_user(data: &serde_json::Value, scopes_header: &str) -> AuthStatus {
        let scopes: Vec<String> = if scopes_header.is_empty() {
            vec![]
        } else {
            scopes_header
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        };

        let user = AuthUser {
            login: data
                .get("login")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            html_url: data
                .get("html_url")
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

        AuthStatus { user, scopes }
    }

    #[test]
    fn test_normalize_user_full() {
        let json = serde_json::json!({
            "login": "octocat",
            "html_url": "https://github.com/octocat",
            "avatar_url": "https://github.com/images/octocat.png",
            "name": "The Octocat"
        });

        let result = normalize_user(&json, "repo,read:org");

        assert_eq!(result.user.login, "octocat");

        assert_eq!(result.user.html_url, "https://github.com/octocat");
        assert_eq!(
            result.user.avatar_url,
            "https://github.com/images/octocat.png"
        );

        assert_eq!(result.user.name, Some("The Octocat".to_string()));

        assert_eq!(result.scopes, vec!["repo", "read:org"]);
    }

    #[test]
    fn test_normalize_user_minimal() {
        let json = serde_json::json!({
            "login": "bot"
        });

        let result = normalize_user(&json, "");

        assert_eq!(result.user.login, "bot");

        assert_eq!(result.user.html_url, "");
        assert!(result.user.name.is_none());

        assert!(result.scopes.is_empty());
    }

    #[test]
    fn test_normalize_user_scopes_parsing() {
        let json = serde_json::json!({ "login": "u", "html_url": "", "avatar_url": "" });

        let result = normalize_user(&json, "  repo ,  read:org  , gist  ");

        assert_eq!(result.scopes, vec!["repo", "read:org", "gist"]);
    }
}
