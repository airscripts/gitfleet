use gitfleet_core::errors::{GitfleetError, UnprocessableError};
use gitfleet_core::types::CodeSearchResult;

use crate::gitlab::client::ProviderClient;

fn encode_path(project: &str) -> String {
    urlencoding::encode(project).to_string()
}

pub struct CodeApi;

impl CodeApi {
    pub async fn file_contents(
        client: &ProviderClient,
        project: &str,
        path: &str,
        r#ref: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let mut endpoint = format!(
            "/projects/{encoded}/repository/files/{}",
            urlencoding::encode(path)
        );

        let reference = r#ref.unwrap_or("HEAD");

        endpoint.push_str(&format!(
            "?raw=false&ref={}",
            urlencoding::encode(reference)
        ));

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get file contents: {e}")))?;

        crate::decode_file_content(data)
    }

    pub async fn search(
        client: &ProviderClient,
        query: &str,
        repo: Option<&str>,
        language: Option<&str>,
        limit: u32,
    ) -> Result<Vec<CodeSearchResult>, GitfleetError> {
        let project = repo.ok_or_else(|| {
            GitfleetError::from(UnprocessableError::new(
                "GitLab code search requires a repository.",
            ))
        })?;
        let encoded = encode_path(project);
        let query = match language {
            Some(language) => format!("{query} extension:{}", language_extension(language)),
            None => query.to_string(),
        };
        let endpoint = format!(
            "/projects/{encoded}/search?scope=blobs&search={}&per_page={limit}",
            urlencoding::encode(&query)
        );

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let raw: Vec<serde_json::Value> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to search code: {e}")))?;

        Ok(raw
            .iter()
            .map(|item| normalize_search_result(item, project))
            .collect())
    }
}

fn normalize_search_result(item: &serde_json::Value, project: &str) -> CodeSearchResult {
    CodeSearchResult {
        file: item
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        repo: project.to_string(),
        url: String::new(),
    }
}

fn language_extension(language: &str) -> &str {
    match language.to_ascii_lowercase().as_str() {
        "c++" | "cpp" => "cpp",
        "c#" | "csharp" => "cs",
        "javascript" => "js",
        "kotlin" => "kt",
        "python" => "py",
        "ruby" => "rb",
        "rust" => "rs",
        "shell" | "bash" => "sh",
        "typescript" => "ts",
        _ => language,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_gitlab_code_search_full() {
        let json = serde_json::json!({
            "filename": "main.rs",
            "path": "src/main.rs",
            "ref": "main"
        });

        let result = normalize_search_result(&json, "org/repo");

        assert_eq!(result.file, "src/main.rs");

        assert_eq!(result.repo, "org/repo");
        assert_eq!(result.url, "");
    }

    #[test]
    fn test_normalize_gitlab_code_search_minimal() {
        let json = serde_json::json!({});

        let result = normalize_search_result(&json, "org/repo");

        assert_eq!(result.file, "");

        assert_eq!(result.repo, "org/repo");
        assert_eq!(result.url, "");
    }

    #[test]
    fn test_language_extension_maps_common_languages() {
        assert_eq!(language_extension("rust"), "rs");
        assert_eq!(language_extension("Python"), "py");
        assert_eq!(language_extension("go"), "go");
    }
}
