use reqwest::Method;
use wiremock::matchers::{method, path, query_param_is_missing};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn github_retries_transient_reads() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/retry"))
        .respond_with(ResponseTemplate::new(503))
        .up_to_n_times(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/retry"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok": true})))
        .expect(1)
        .mount(&server)
        .await;

    let client = gitfleet_providers::github::ProviderClient::with_base_url(&server.uri());
    let response = client
        .request_url(
            Method::GET,
            &format!("{}/retry", server.uri()),
            None,
            Some("token"),
            None,
            None,
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn gitlab_retries_rate_limited_reads_using_retry_after() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/retry"))
        .respond_with(ResponseTemplate::new(429).insert_header("retry-after", "0"))
        .up_to_n_times(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/retry"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;

    let client = gitfleet_providers::gitlab::ProviderClient::with_base_url(&server.uri());
    let response = client
        .request_url(
            Method::GET,
            &format!("{}/retry", server.uri()),
            None,
            Some("token"),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn github_does_not_retry_mutations() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/mutate"))
        .respond_with(ResponseTemplate::new(503))
        .expect(1)
        .mount(&server)
        .await;

    let client = gitfleet_providers::github::ProviderClient::with_base_url(&server.uri());
    let result = client
        .request_url(
            Method::POST,
            &format!("{}/mutate", server.uri()),
            Some(serde_json::json!({"value": true})),
            Some("token"),
            None,
            None,
        )
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn github_aggregates_pages_and_preserves_page_size() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/items"))
        .and(query_param_is_missing("page"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header(
                    "link",
                    format!("<{}/items?page=2&per_page=2>; rel=\"next\"", server.uri()),
                )
                .set_body_json(vec![serde_json::json!({"id": 1})]),
        )
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/items"))
        .and(wiremock::matchers::query_param("page", "2"))
        .and(wiremock::matchers::query_param("per_page", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vec![serde_json::json!({"id": 2})]))
        .expect(1)
        .mount(&server)
        .await;

    let client = gitfleet_providers::github::ProviderClient::with_base_url(&server.uri());
    let items: Vec<serde_json::Value> = client
        .get_paginated("/items?per_page=2", Some("token"), None)
        .await
        .unwrap();

    assert_eq!(items.len(), 2);
}

#[tokio::test]
async fn gitlab_aggregates_pages_and_preserves_page_size() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/items"))
        .and(query_param_is_missing("page"))
        .and(wiremock::matchers::query_param("per_page", "2"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("x-next-page", "2")
                .set_body_json(vec![serde_json::json!({"id": 1})]),
        )
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/items"))
        .and(wiremock::matchers::query_param("page", "2"))
        .and(wiremock::matchers::query_param("per_page", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vec![serde_json::json!({"id": 2})]))
        .expect(1)
        .mount(&server)
        .await;

    let client = gitfleet_providers::gitlab::ProviderClient::with_base_url(&server.uri());
    let items: Vec<serde_json::Value> = client
        .get_paginated("/items?per_page=2", Some("token"), None)
        .await
        .unwrap();

    assert_eq!(items.len(), 2);
}

#[tokio::test]
async fn gitlab_rejects_malformed_pagination_metadata() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/items"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("x-next-page", "not-a-page")
                .set_body_json(vec![serde_json::json!({"id": 1})]),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = gitfleet_providers::gitlab::ProviderClient::with_base_url(&server.uri());
    let result: Result<Vec<serde_json::Value>, _> = client
        .get_paginated("/items?per_page=2", Some("token"), None)
        .await;

    let error = result.unwrap_err().to_string();

    assert!(error.contains("Malformed GitLab pagination"));
}

#[tokio::test]
async fn gitlab_unstar_accepts_already_unstarred_response() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/unstar"))
        .respond_with(ResponseTemplate::new(304))
        .expect(1)
        .mount(&server)
        .await;

    let client = gitfleet_providers::gitlab::ProviderClient::with_base_url(&server.uri());
    let response = client
        .request_url(
            Method::POST,
            &format!("{}/unstar", server.uri()),
            None,
            Some("token"),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 304);
}

#[tokio::test]
async fn gitlab_does_not_treat_not_modified_reads_as_success() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repo"))
        .respond_with(ResponseTemplate::new(304))
        .expect(1)
        .mount(&server)
        .await;

    let client = gitfleet_providers::gitlab::ProviderClient::with_base_url(&server.uri());
    let result = client
        .request_url(
            Method::GET,
            &format!("{}/repo", server.uri()),
            None,
            Some("token"),
        )
        .await;

    assert!(result.is_err());
}
