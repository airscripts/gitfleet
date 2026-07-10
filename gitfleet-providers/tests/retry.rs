use reqwest::Method;
use wiremock::matchers::{method, path};
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
