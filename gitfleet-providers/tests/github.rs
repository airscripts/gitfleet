use gitfleet_core::provider::{GitProvider, ProviderCapability};
use gitfleet_core::types::MilestoneState;
use gitfleet_providers::GitHubProvider;
use serial_test::serial;
use wiremock::matchers::{body_json, body_string_contains, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn setup_token() {
    std::env::set_var("GITFLEET_GITHUB_TOKEN", "testtoken");
    std::env::set_var("GITFLEET_PROFILE", "__gitfleet_integration_test__");
}

fn teardown_token() {
    std::env::remove_var("GITFLEET_GITHUB_TOKEN");
    std::env::remove_var("GITFLEET_PROFILE");
}

fn repo_json() -> serde_json::Value {
    serde_json::json!([
        {
            "id": 12345,
            "name": "my-repo",
            "fork": false,
            "private": true,
            "archived": false,
            "full_name": "testorg/my-repo",
            "html_url": "https://github.com/testorg/my-repo",
            "clone_url": "https://github.com/testorg/my-repo.git",
            "visibility": "private",
            "default_branch": "main",
            "pushed_at": "2024-06-01T00:00:00Z",
            "homepage": null,
            "owner": { "login": "testorg" },
            "open_issues_count": 5,
            "stargazers_count": 10,
            "description": "A test repository",
            "parent": null
        }
    ])
}

fn single_repo_json() -> serde_json::Value {
    serde_json::json!({
        "id": 99999,
        "name": "new-repo",
        "fork": false,
        "private": false,
        "archived": false,
        "full_name": "testuser/new-repo",
        "html_url": "https://github.com/testuser/new-repo",
        "clone_url": "https://github.com/testuser/new-repo.git",
        "visibility": "public",
        "default_branch": "main",
        "pushed_at": "2024-07-01T00:00:00Z",
        "homepage": null,
        "owner": { "login": "testuser" },
        "open_issues_count": 0,
        "stargazers_count": 0,
        "description": "New repo",
        "parent": null
    })
}

fn pr_json() -> serde_json::Value {
    serde_json::json!([
        {
            "title": "Fix bug in auth",
            "state": "open",
            "number": 42,
            "merged": false,
            "draft": false,
            "html_url": "https://github.com/testorg/my-repo/pull/42",
            "created_at": "2024-06-01T00:00:00Z",
            "updated_at": "2024-06-02T00:00:00Z",
            "body": "This fixes the auth bug",
            "mergeable_state": "clean",
            "merged_at": null,
            "mergeable": true,
            "user": { "login": "contributor" },
            "maintainer_can_modify": true,
            "merge_commit_sha": null,
            "labels": [],
            "requested_reviewers": [],
            "head": {
                "ref": "fix-auth",
                "sha": "abc123def456",
                "repo": {
                    "full_name": "contributor/my-repo",
                    "html_url": "https://github.com/contributor/my-repo"
                }
            },
            "base": {
                "ref": "main",
                "repo": {
                    "full_name": "testorg/my-repo",
                    "html_url": "https://github.com/testorg/my-repo"
                }
            }
        }
    ])
}

fn issue_search_json() -> serde_json::Value {
    serde_json::json!({
        "total_count": 1,
        "incomplete_results": false,
        "items": [
            {
                "id": 100,
                "title": "Bug in login",
                "state": "open",
                "number": 7,
                "html_url": "https://github.com/testorg/my-repo/issues/7",
                "is_pull_request": false,
                "repository_url": "https://api.github.com/repos/testorg/my-repo",
                "user": { "login": "reporter" },
                "labels": [{ "name": "bug", "color": "ff0000" }],
                "score": 1.0,
                "body": "Login fails on mobile",
                "created_at": "2024-06-01T00:00:00Z",
                "updated_at": "2024-06-02T00:00:00Z",
                "comments": 3,
                "assignees": []
            }
        ]
    })
}

fn label_json() -> serde_json::Value {
    serde_json::json!([
        {
            "id": 1,
            "node_id": "MDU6TGFiZWwx",
            "url": "https://api.github.com/repos/testorg/my-repo/labels/bug",
            "name": "bug",
            "color": "ff0000",
            "default": true,
            "description": "Something isn't working"
        },
        {
            "id": 2,
            "node_id": "MDU6TGFiZWwy",
            "url": "https://api.github.com/repos/testorg/my-repo/labels/enhancement",
            "name": "enhancement",
            "color": "84b6eb",
            "default": false,
            "description": "New feature or request"
        }
    ])
}

fn notification_json() -> serde_json::Value {
    serde_json::json!([
        {
            "id": "1234567",
            "repository": {
                "id": 12345,
                "full_name": "testorg/my-repo",
                "html_url": "https://github.com/testorg/my-repo"
            },
            "subject": {
                "title": "Fix bug in auth",
                "type": "PullRequest",
                "url": "https://api.github.com/repos/testorg/my-repo/pulls/42"
            },
            "reason": "subscribed",
            "unread": true,
            "updated_at": "2024-06-15T12:00:00Z"
        }
    ])
}

#[tokio::test]
#[serial]
async fn test_list_org_repos() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/orgs/testorg/repos"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(repo_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().expect("repo ops");

    let repos = ops.list_org_repos("testorg").await.expect("list org repos");

    teardown_token();

    assert_eq!(repos.len(), 1);

    assert_eq!(repos[0].name, "my-repo");
    assert_eq!(repos[0].full_name, "testorg/my-repo");

    assert!(!repos[0].fork);
    assert!(repos[0].private);

    assert!(!repos[0].archived);
    assert_eq!(repos[0].default_branch, "main");

    assert_eq!(repos[0].id, 12345);
}

#[tokio::test]
#[serial]
async fn test_list_user_repos() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/user/repos"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(repo_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().expect("repo ops");

    let repos = ops.list_user_repos().await.expect("list user repos");

    teardown_token();

    assert_eq!(repos.len(), 1);

    assert_eq!(repos[0].name, "my-repo");
    assert_eq!(repos[0].id, 12345);
}

#[tokio::test]
#[serial]
async fn test_list_user_named_repos() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/users/octocat/repos"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(repo_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().expect("repo ops");

    let repos = ops
        .list_user_named_repos("octocat")
        .await
        .expect("list user named repos");
    teardown_token();

    assert_eq!(repos.len(), 1);

    assert_eq!(repos[0].full_name, "testorg/my-repo");
}

#[tokio::test]
#[serial]
async fn test_list_changes() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/my-repo/pulls"))
        .and(query_param("state", "open"))
        .and(query_param("per_page", "100"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(pr_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.change_ops().expect("change ops");

    let pulls = ops
        .list_changes("testorg/my-repo", "open", 100, None, None)
        .await
        .expect("list changes");
    teardown_token();

    assert_eq!(pulls.len(), 1);

    assert_eq!(pulls[0].title, "Fix bug in auth");
    assert_eq!(pulls[0].state, "open");

    assert_eq!(pulls[0].number, 42);
    assert!(!pulls[0].merged);

    assert_eq!(pulls[0].head.r#ref, "fix-auth");
    assert_eq!(pulls[0].base.r#ref, "main");
}

#[tokio::test]
#[serial]
async fn test_list_issues() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/search/issues"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(issue_search_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.issue_ops().expect("issue ops");

    let result = ops
        .list_issues("testorg/my-repo", "open", 30, &[], &[])
        .await
        .expect("list issues");
    teardown_token();

    let total = result
        .get("total_count")
        .and_then(|v| v.as_u64())
        .expect("total_count");
    assert_eq!(total, 1);

    let items = result
        .get("items")
        .and_then(|v| v.as_array())
        .expect("items");
    assert_eq!(items.len(), 1);

    assert_eq!(
        items[0].get("title").and_then(|v| v.as_str()),
        Some("Bug in login")
    );
}

#[tokio::test]
#[serial]
async fn test_list_labels() {
    setup_token();

    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/my-repo/labels"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(label_json()))
        .mount(&server)
        .await;

    let provider = GitHubProvider::with_base_url(&server.uri());

    let ops = provider.label_ops().expect("label ops");
    let labels = ops
        .list_labels("testorg/my-repo")
        .await
        .expect("list labels");
    teardown_token();

    assert_eq!(labels.len(), 2);

    assert_eq!(labels[0].name, "bug");
    assert_eq!(labels[0].color, "ff0000");

    assert_eq!(labels[0].description, "Something isn't working");
    assert_eq!(labels[1].name, "enhancement");

    assert_eq!(labels[1].color, "84b6eb");
    assert_eq!(labels[1].description, "New feature or request");
}

#[tokio::test]
#[serial]
async fn test_list_notifications() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/notifications"))
        .and(query_param("per_page", "100"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(notification_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.notification_ops().expect("notification ops");

    let notifications = ops
        .list_notifications(true, false, None)
        .await
        .expect("list notifications");
    teardown_token();

    assert_eq!(notifications.len(), 1);

    assert_eq!(notifications[0].id, "1234567");
    assert_eq!(notifications[0].repository, "testorg/my-repo");

    assert_eq!(notifications[0].subject_title, "Fix bug in auth");
    assert_eq!(notifications[0].subject_type, "PullRequest");

    assert_eq!(notifications[0].reason, "subscribed");
    assert!(notifications[0].unread);

    assert_eq!(notifications[0].updated_at, "2024-06-15T12:00:00Z");
}

#[tokio::test]
#[serial]
async fn test_repo_not_found_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/orgs/nonexistent/repos"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().expect("repo ops");

    let result = ops.list_org_repos("nonexistent").await;

    teardown_token();

    assert!(result.is_err());

    let err = result.unwrap_err();

    assert!(err.to_string().contains("not found"));
}

#[tokio::test]
#[serial]
async fn test_create_repo_user() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/user/repos"))
        .and(header("authorization", "Bearer testtoken"))
        .and(body_json(serde_json::json!({
            "name": "new-repo",
            "visibility": "public",
            "description": "New repo",
            "auto_init": false
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(single_repo_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().expect("repo ops");

    let repo = ops
        .create_repo("new-repo", "public", None, None, Some("New repo"), false)
        .await
        .expect("create repo");
    teardown_token();

    let name = repo.get("name").and_then(|v| v.as_str()).expect("name");

    assert_eq!(name, "new-repo");
}

#[tokio::test]
#[serial]
async fn test_create_repo_org() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/orgs/testorg/repos"))
        .and(header("authorization", "Bearer testtoken"))
        .and(body_json(serde_json::json!({
            "name": "new-repo",
            "visibility": "public",
            "description": "New repo",
            "auto_init": true
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(single_repo_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().expect("repo ops");

    let repo = ops
        .create_repo(
            "new-repo",
            "public",
            Some("testorg"),
            Some("org"),
            Some("New repo"),
            true,
        )
        .await
        .expect("create org repo");
    teardown_token();

    let name = repo.get("name").and_then(|v| v.as_str()).expect("name");

    assert_eq!(name, "new-repo");
}

#[tokio::test]
#[serial]
async fn test_star_repo() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/user/starred/testorg/my-repo"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().expect("repo ops");

    let result = ops.star_repo("testorg/my-repo").await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_unstar_repo() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/user/starred/testorg/my-repo"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().expect("repo ops");

    let result = ops.unstar_repo("testorg/my-repo").await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_list_changes_with_base_and_head() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/my-repo/pulls"))
        .and(query_param("state", "open"))
        .and(query_param("per_page", "50"))
        .and(query_param("base", "main"))
        .and(query_param("head", "testorg:feature"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(pr_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.change_ops().expect("change ops");

    let pulls = ops
        .list_changes(
            "testorg/my-repo",
            "open",
            50,
            Some("main"),
            Some("testorg:feature"),
        )
        .await
        .expect("list changes with filters");
    teardown_token();

    assert_eq!(pulls.len(), 1);
}

#[tokio::test]
#[serial]
async fn test_list_notifications_with_repo() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/my-repo/notifications"))
        .and(query_param("per_page", "100"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(notification_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.notification_ops().expect("notification ops");

    let notifications = ops
        .list_notifications(false, false, Some("testorg/my-repo"))
        .await
        .expect("list repo notifications");
    teardown_token();

    assert_eq!(notifications.len(), 1);
}

#[tokio::test]
#[serial]
async fn test_list_org_repos_empty() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/orgs/emptyorg/repos"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().expect("repo ops");

    let repos = ops
        .list_org_repos("emptyorg")
        .await
        .expect("list empty org repos");
    teardown_token();

    assert!(repos.is_empty());
}

#[tokio::test]
#[serial]
async fn test_list_labels_empty() {
    setup_token();

    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/my-repo/labels"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;

    let provider = GitHubProvider::with_base_url(&server.uri());

    let ops = provider.label_ops().expect("label ops");
    let labels = ops
        .list_labels("testorg/my-repo")
        .await
        .expect("list empty labels");

    assert!(labels.is_empty());

    teardown_token();
}

#[tokio::test]
#[serial]
async fn test_list_notifications_empty() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/notifications"))
        .and(query_param("per_page", "100"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.notification_ops().expect("notification ops");

    let notifications = ops
        .list_notifications(true, false, None)
        .await
        .expect("list empty notifications");
    teardown_token();

    assert!(notifications.is_empty());
}

#[tokio::test]
#[serial]
async fn test_repo_unauthorized_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/orgs/privateorg/repos"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().expect("repo ops");

    let result = ops.list_org_repos("privateorg").await;

    teardown_token();

    assert!(result.is_err());
}

#[tokio::test]
#[serial]
async fn test_list_changes_empty() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/my-repo/pulls"))
        .and(query_param("state", "open"))
        .and(query_param("per_page", "100"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.change_ops().expect("change ops");

    let pulls = ops
        .list_changes("testorg/my-repo", "open", 100, None, None)
        .await
        .expect("list empty changes");
    teardown_token();

    assert!(pulls.is_empty());
}

#[tokio::test]
#[serial]
async fn test_list_user_named_repos_empty() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/users/norepouser/repos"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().expect("repo ops");

    let repos = ops
        .list_user_named_repos("norepouser")
        .await
        .expect("list empty user repos");
    teardown_token();

    assert!(repos.is_empty());
}

fn auth_user_json() -> serde_json::Value {
    serde_json::json!({
        "login": "octocat",
        "html_url": "https://github.com/octocat",
        "avatar_url": "https://github.com/images/octocat.png",
        "name": "The Octocat"
    })
}

fn webhook_json() -> serde_json::Value {
    serde_json::json!([
        {
            "id": 1,
            "name": "web",
            "url": "https://api.github.com/repos/o/r/hooks/1",
            "config": {"url": "https://example.com/webhook"},
            "events": ["push"],
            "active": true,
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        }
    ])
}

fn release_json() -> serde_json::Value {
    serde_json::json!([
        {
            "id": 1,
            "tag_name": "v1.0.0",
            "name": "Release 1.0.0",
            "body": "First release",
            "draft": false,
            "prerelease": false,
            "created_at": "2024-01-01T00:00:00Z",
            "published_at": "2024-01-01T00:00:00Z",
            "html_url": "https://github.com/o/r/releases/tag/v1.0.0"
        }
    ])
}

fn workflow_json() -> serde_json::Value {
    serde_json::json!({
        "total_count": 1,
        "workflows": [
            {
                "id": 123,
                "name": "CI",
                "path": ".github/workflows/ci.yml",
                "state": "active",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-02T00:00:00Z",
                "html_url": "https://github.com/o/r/actions/workflows/ci.yml"
            }
        ]
    })
}

fn environment_json() -> serde_json::Value {
    serde_json::json!({
        "total_count": 1,
        "environments": [
            {
                "id": 1,
                "name": "production",
                "url": "https://example.com",
                "html_url": "https://github.com/o/r/settings/environments/1",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            }
        ]
    })
}

fn deployment_json() -> serde_json::Value {
    serde_json::json!([
        {
            "id": 1,
            "ref": "main",
            "environment": "production",
            "task": "deploy",
            "description": "Deploy main",
            "creator": { "login": "octocat" },
            "created_at": "2024-01-01T00:00:00Z",
            "production_environment": true
        }
    ])
}

fn runner_json() -> serde_json::Value {
    serde_json::json!({
        "total_count": 1,
        "runners": [
            {
                "id": 1,
                "name": "runner-1",
                "os": "linux",
                "status": "online",
                "busy": false,
                "labels": [
                    { "name": "self-hosted", "type": "read-only" }
                ]
            }
        ]
    })
}

#[tokio::test]
#[serial]
async fn test_labels_create() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/repos/testorg/my-repo/labels"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": 3,
            "name": "enhancement",
            "color": "a2eeef",
            "description": "New feature"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.label_ops().expect("label ops");

    let label = gitfleet_core::types::Label {
        name: "enhancement".to_string(),
        color: "a2eeef".to_string(),
        description: "New feature".to_string(),
        new_name: None,
    };

    let result = ops.create_label(&label, "testorg/my-repo").await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_labels_delete() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/repos/testorg/my-repo/labels/bug"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.label_ops().expect("label ops");

    let result = ops.delete_label("bug", "testorg/my-repo").await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_releases_list() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/my-repo/releases"))
        .and(query_param("per_page", "10"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(release_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.release_ops().expect("release ops");

    let result = ops.list_releases("testorg/my-repo", 10).await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_workflows_list() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/my-repo/actions/workflows"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(workflow_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.pipeline_ops().expect("pipeline ops");

    let result = ops.list_workflows("testorg/my-repo", 30, None).await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_webhooks_list() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/my-repo/hooks"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(webhook_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.webhook_ops().expect("webhook ops");

    let result = ops.list_webhooks("testorg/my-repo").await;

    teardown_token();

    assert!(result.is_ok());

    let hooks = result.unwrap();

    assert_eq!(hooks.len(), 1);

    assert_eq!(hooks[0].id, 1);
    assert_eq!(hooks[0].name, "web");
    assert_eq!(hooks[0].url, "https://example.com/webhook");

    assert!(hooks[0].active);
}

#[tokio::test]
#[serial]
async fn test_auth_get_authenticated_user() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/user"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(auth_user_json())
                .insert_header("x-oauth-scopes", "repo,read:org"),
        )
        .mount(&server)
        .await;

    setup_token();

    let _provider = GitHubProvider::with_base_url(&server.uri());
    let client = gitfleet_providers::github::ProviderClient::with_base_url(&server.uri());

    let result =
        gitfleet_providers::github::api::AuthApi::fetch_authenticated_user(&client, None, None)
            .await;

    teardown_token();

    assert!(result.is_ok());

    let auth = result.unwrap();

    assert_eq!(auth.user.login, "octocat");

    assert_eq!(auth.user.name, Some("The Octocat".to_string()));
    assert_eq!(auth.scopes, vec!["repo", "read:org"]);
}

#[tokio::test]
#[serial]
async fn test_environments_list() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/my-repo/environments"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(environment_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.environment_ops().expect("environment ops");

    let result = ops.list_environments("testorg", "my-repo").await;

    teardown_token();

    assert!(result.is_ok());

    let envs = result.unwrap();

    assert_eq!(envs.total_count, 1);

    assert_eq!(envs.environments.len(), 1);
    assert_eq!(envs.environments[0].name, "production");
}

#[tokio::test]
#[serial]
async fn test_deployments_list() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/my-repo/deployments"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(deployment_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.deploy_ops().expect("deploy ops");

    let result = ops.list_deployments("testorg/my-repo", None, 30).await;

    teardown_token();

    assert!(result.is_ok());

    let deploys = result.unwrap();

    assert_eq!(deploys.len(), 1);

    assert_eq!(deploys[0].id, 1);
    assert_eq!(deploys[0].r#ref, "main");

    assert_eq!(deploys[0].environment, "production");
    assert!(deploys[0].production);
}

#[tokio::test]
#[serial]
async fn test_runners_list() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/my-repo/actions/runners"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(runner_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.runner_ops().expect("runner ops");

    let result = ops.list_runners("testorg/my-repo").await;

    teardown_token();

    assert!(result.is_ok());

    let runners = result.unwrap();

    assert_eq!(runners.len(), 1);

    assert_eq!(runners[0].id, 1);
    assert_eq!(runners[0].name, "runner-1");

    assert_eq!(runners[0].os, "linux");
    assert!(!runners[0].busy);
}

#[tokio::test]
#[serial]
async fn test_workflows_list_runs() {
    let runs_json = serde_json::json!({
        "total_count": 2,
        "workflow_runs": [
            {
                "id": 101,
                "name": "CI",
                "status": "completed",
                "conclusion": "success",
                "head_branch": "main",
                "head_sha": "abc123",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T01:00:00Z",
                "html_url": "https://github.com/o/r/actions/runs/101"
            },
            {
                "id": 102,
                "name": "Deploy",
                "status": "in_progress",
                "conclusion": null,
                "head_branch": "feature",
                "head_sha": "def456",
                "created_at": "2024-01-02T00:00:00Z",
                "updated_at": "2024-01-02T00:30:00Z",
                "html_url": "https://github.com/o/r/actions/runs/102"
            }
        ]
    });

    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/my-repo/actions/runs"))
        .and(query_param("per_page", "10"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(runs_json))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.pipeline_ops().expect("pipeline ops");

    let result = ops.list_runs("testorg/my-repo", "per_page=10", 10).await;

    teardown_token();

    assert!(result.is_ok());

    let data = result.unwrap();

    let total = data
        .get("total_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    assert_eq!(total, 2);
}

#[tokio::test]
#[serial]
async fn test_workflows_dispatch() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path(
            "/repos/testorg/my-repo/actions/workflows/ci.yml/dispatches",
        ))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.pipeline_ops().expect("pipeline ops");

    let result = ops
        .dispatch_pipeline("testorg/my-repo", Some("ci.yml"), "main", None)
        .await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_workflows_dispatch_requires_definition_id() {
    setup_token();

    let provider = GitHubProvider::with_base_url("http://127.0.0.1:1");
    let ops = provider.pipeline_ops().expect("pipeline ops");

    let result = ops
        .dispatch_pipeline("testorg/my-repo", None, "main", None)
        .await;

    teardown_token();

    let error = result.expect_err("GitHub must require a pipeline definition ID");

    assert!(error.to_string().contains("definition ID"));
}

#[tokio::test]
#[serial]
async fn test_releases_create() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/repos/testorg/my-repo/releases"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": 2,
            "tag_name": "v2.0.0",
            "name": "Release 2.0.0",
            "body": "Second release",
            "draft": false,
            "prerelease": false,
            "created_at": "2024-02-01T00:00:00Z",
            "published_at": "2024-02-01T00:00:00Z",
            "html_url": "https://github.com/o/r/releases/tag/v2.0.0"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.release_ops().expect("release ops");

    let result = ops
        .create_release(
            "testorg/my-repo",
            serde_json::json!({
                "tag_name": "v2.0.0",
                "name": "Release 2.0.0",
                "body": "Second release"
            }),
        )
        .await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_environments_create() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/repos/testorg/my-repo/environments/staging"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 2,
            "name": "staging",
            "url": "https://example.com",
            "html_url": "https://github.com/o/r/settings/environments/2",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.environment_ops().expect("environment ops");

    let result = ops
        .create_environment("testorg", "my-repo", "staging", None)
        .await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_deployments_create() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/repos/testorg/my-repo/deployments"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": 2,
            "ref": "main",
            "environment": "staging",
            "task": "deploy",
            "description": "Deploy to staging",
            "creator": { "login": "octocat" },
            "created_at": "2024-01-01T00:00:00Z",
            "production_environment": false
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.deploy_ops().expect("deploy ops");

    let result = ops
        .create_deployment(
            "testorg/my-repo",
            serde_json::json!({
                "ref": "main",
                "environment": "staging"
            }),
        )
        .await;

    teardown_token();

    assert!(result.is_ok());

    let deploy = result.unwrap();

    assert_eq!(deploy.id, 2);

    assert_eq!(deploy.r#ref, "main");
    assert_eq!(deploy.environment, "staging");
}

#[tokio::test]
#[serial]
async fn test_runners_remove() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/repos/testorg/my-repo/actions/runners/1"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.runner_ops().expect("runner ops");

    let result = ops.remove_runner("testorg/my-repo", 1).await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_webhooks_create() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/repos/testorg/my-repo/hooks"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": 2,
            "name": "web",
            "url": "https://api.github.com/repos/testorg/my-repo/hooks/2",
            "config": {"url": "https://example.com/webhook"},
            "events": ["push", "pull_request"],
            "active": true,
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.webhook_ops().expect("webhook ops");

    let result = ops
        .create_webhook(
            "testorg/my-repo",
            serde_json::json!({
                "name": "web",
                "active": true,
                "events": ["push", "pull_request"],
                "config": { "url": "https://example.com/webhook" }
            }),
        )
        .await;

    teardown_token();

    assert!(result.is_ok());

    let hook = result.unwrap();

    assert_eq!(hook.id, 2);

    assert_eq!(hook.events, vec!["push", "pull_request"]);
}

#[tokio::test]
#[serial]
async fn test_webhooks_delete() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/repos/testorg/my-repo/hooks/1"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.webhook_ops().expect("webhook ops");

    let result = ops.remove_webhook("testorg/my-repo", 1).await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_discussions_list() {
    let discussion_response = serde_json::json!({
        "data": {
            "repository": {
                "discussions": {
                    "nodes": [
                        {
                            "id": "DI_1",
                            "title": "How to contribute?",
                            "number": 1,
                            "url": "https://github.com/org/repo/discussions/1",
                            "body": "Discussion body",
                            "author": { "login": "octocat" },
                            "closed": false,
                            "category": { "name": "Q&A" },
                            "createdAt": "2024-01-01T00:00:00Z",
                            "updatedAt": "2024-01-02T00:00:00Z",
                            "comments": { "totalCount": 5 }
                        }
                    ]
                }
            }
        }
    });

    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(discussion_response))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.discussion_ops().expect("discussion ops");

    let result = ops.list_discussions("org", "repo", None, 10).await;

    teardown_token();

    assert!(result.is_ok());

    let discussions = result.unwrap();

    assert_eq!(discussions.len(), 1);

    assert_eq!(discussions[0].title, "How to contribute?");
    assert_eq!(discussions[0].author, "octocat");

    assert_eq!(discussions[0].comments_count, 5);
}

#[tokio::test]
#[serial]
async fn test_identity_list_ssh_keys() {
    let ssh_keys_json = serde_json::json!([
        {
            "id": 1,
            "title": "My Key",
            "key": "ssh-rsa AAAAB3...",
            "created_at": "2024-01-01T00:00:00Z"
        }
    ]);

    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/user/keys"))
        .and(query_param("per_page", "100"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ssh_keys_json))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.identity_ops().expect("identity ops");

    let result = ops.list_ssh_keys().await;

    teardown_token();

    assert!(result.is_ok());

    let keys = result.unwrap();

    assert_eq!(keys.len(), 1);

    assert_eq!(keys[0].id, 1);
    assert_eq!(keys[0].title, "My Key");
}

#[tokio::test]
#[serial]
async fn test_governance_list_rulesets() {
    let rulesets_json = serde_json::json!([
        {
            "id": 1,
            "name": "main-protection",
            "target": "branch",
            "enforcement": "active"
        }
    ]);

    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/my-repo/rulesets"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(rulesets_json))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.governance_ops().expect("governance ops");

    let result = ops.list_rulesets("testorg/my-repo").await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_search_issues() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/search/issues"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(issue_search_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.search_ops().expect("search ops");

    let result = ops
        .search_issues("repo:testorg/my-repo is:issue", None, None, 30)
        .await;

    teardown_token();

    assert!(result.is_ok());

    let search = result.unwrap();

    assert_eq!(search.total_count, 1);

    assert_eq!(search.items.len(), 1);
}

#[tokio::test]
#[serial]
async fn test_search_repos() {
    let repos_search_json = serde_json::json!({
        "total_count": 1,
        "incomplete_results": false,
        "items": [
            {
                "id": 12345,
                "name": "my-repo",
                "full_name": "testorg/my-repo",
                "html_url": "https://github.com/testorg/my-repo",
                "description": "A test repository",
                "stargazers_count": 10,
                "fork": false,
                "private": false
            }
        ]
    });

    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/search/repositories"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(repos_search_json))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.search_ops().expect("search ops");

    let result = ops.search_repos("my-repo", None, None, 30).await;

    teardown_token();

    assert!(result.is_ok());

    let search = result.unwrap();

    assert_eq!(search.total_count, 1);
}

#[tokio::test]
#[serial]
async fn test_search_code() {
    let code_search_json = serde_json::json!({
        "total_count": 1,
        "incomplete_results": false,
        "items": [
            {
                "name": "main.rs",
                "path": "src/main.rs",
                "repository": { "full_name": "org/repo" },
                "html_url": "https://github.com/org/repo/blob/main/src/main.rs"
            }
        ]
    });

    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/search/code"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(code_search_json))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.search_ops().expect("search ops");

    let result = ops.search_code("fn main", 30).await;

    teardown_token();

    assert!(result.is_ok());

    let search = result.unwrap();

    assert_eq!(search.total_count, 1);
}

#[tokio::test]
#[serial]
async fn test_secrets_list() {
    let secrets_json = serde_json::json!({
        "total_count": 1,
        "secrets": [
            {
                "name": "MY_SECRET",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            }
        ]
    });

    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/my-repo/actions/secrets"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(secrets_json))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.secret_ops().expect("secret ops");

    let result = ops.list_repo_secrets("testorg", "my-repo").await;

    teardown_token();

    assert!(result.is_ok());

    let secrets = result.unwrap();

    assert_eq!(secrets.total_count, 1);

    assert_eq!(secrets.secrets.len(), 1);
    assert_eq!(secrets.secrets[0].name, "MY_SECRET");
}

#[tokio::test]
#[serial]
async fn test_secrets_get_public_key() {
    let pubkey_json = serde_json::json!({
        "key_id": "0123456789",
        "key": "2Sg8iYjAxxmI2LvQBp5QgQ9gK5M0"
    });

    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/my-repo/actions/secrets/public-key"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(pubkey_json))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.secret_ops().expect("secret ops");

    let result = ops.get_repo_public_key("testorg", "my-repo").await;

    teardown_token();

    assert!(result.is_ok());

    let key = result.unwrap();

    assert_eq!(key.key_id, "0123456789");

    assert_eq!(key.key, "2Sg8iYjAxxmI2LvQBp5QgQ9gK5M0");
}

#[tokio::test]
#[serial]
async fn test_secrets_set() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/repos/testorg/my-repo/actions/secrets/MY_SECRET"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.secret_ops().expect("secret ops");

    let result = ops
        .set_repo_secret("testorg", "my-repo", "MY_SECRET", "encrypted_val", "key_id")
        .await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_secrets_delete() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/repos/testorg/my-repo/actions/secrets/MY_SECRET"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.secret_ops().expect("secret ops");

    let result = ops
        .delete_repo_secret("testorg", "my-repo", "MY_SECRET")
        .await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_variables_list() {
    let variables_json = serde_json::json!({
        "total_count": 1,
        "variables": [
            {
                "name": "MY_VAR",
                "value": "hello",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            }
        ]
    });

    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/my-repo/actions/variables"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(variables_json))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.variable_ops().expect("variable ops");

    let result = ops.list_repo_variables("testorg", "my-repo").await;

    teardown_token();

    assert!(result.is_ok());

    let vars = result.unwrap();

    assert_eq!(vars.total_count, 1);

    assert_eq!(vars.variables.len(), 1);
    assert_eq!(vars.variables[0].name, "MY_VAR");
}

#[tokio::test]
#[serial]
async fn test_variables_set() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/repos/testorg/my-repo/actions/variables"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(201))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.variable_ops().expect("variable ops");

    let result = ops
        .set_repo_variable("testorg", "my-repo", "MY_VAR", "hello")
        .await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_variables_delete() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/repos/testorg/my-repo/actions/variables/MY_VAR"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.variable_ops().expect("variable ops");

    let result = ops
        .delete_repo_variable("testorg", "my-repo", "MY_VAR")
        .await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_access_invite_collaborator() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/repos/testorg/my-repo/collaborators/newuser"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(201))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.access_ops().expect("access ops");

    let result = ops
        .invite_collaborator("testorg", "my-repo", "newuser", "admin")
        .await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_access_invite_org_member() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/orgs/testorg/memberships/newuser"))
        .and(header("authorization", "Bearer testtoken"))
        .and(body_json(serde_json::json!({"role": "admin"})))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.access_ops().expect("access ops");

    let result = ops.invite_org_member("testorg", "newuser", "admin").await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_access_list_org_members() {
    let members_json = serde_json::json!([
        {
            "login": "member1",
            "id": 1,
            "avatar_url": "https://github.com/images/member1.png"
        }
    ]);

    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/orgs/testorg/members"))
        .and(query_param("per_page", "100"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(members_json))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.access_ops().expect("access ops");

    let result = ops.list_org_members("testorg").await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_analytics_traffic_views() {
    let views_json = serde_json::json!({
        "count": 100,
        "uniques": 50,
        "views": [
            {
                "timestamp": "2024-01-01T00:00:00Z",
                "count": 10,
                "uniques": 5
            }
        ]
    });

    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/my-repo/traffic/views"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(views_json))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.analytics_ops().expect("analytics ops");

    let result = ops.get_traffic_views("testorg/my-repo").await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_analytics_traffic_clones() {
    let clones_json = serde_json::json!({
        "count": 50,
        "uniques": 25,
        "clones": [
            {
                "timestamp": "2024-01-01T00:00:00Z",
                "count": 5,
                "uniques": 3
            }
        ]
    });

    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/my-repo/traffic/clones"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(clones_json))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.analytics_ops().expect("analytics ops");

    let result = ops.get_traffic_clones("testorg/my-repo").await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_code_file_contents() {
    let file_json = serde_json::json!({
        "name": "README.md",
        "path": "README.md",
        "sha": "abc123",
        "content": "SGVsbG8gV29ybGQ=",
        "encoding": "base64",
        "size": 11
    });

    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/my-repo/contents/README.md"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(file_json))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.code_ops().expect("code ops");

    let result = ops
        .get_file_contents("testorg/my-repo", "README.md", None)
        .await
        .unwrap();

    teardown_token();

    assert_eq!(result["content"], "Hello World");
    assert_eq!(result["encoding"], "utf-8");
}

#[tokio::test]
#[serial]
async fn test_governance_delete_ruleset() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/repos/testorg/my-repo/rulesets/42"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.governance_ops().expect("governance ops");

    let result = ops.delete_ruleset("testorg/my-repo", 42).await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_identity_add_ssh_key() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/user/keys"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": 42,
            "title": "New Key",
            "key": "ssh-rsa AAAAB3...",
            "created_at": "2024-01-01T00:00:00Z"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.identity_ops().expect("identity ops");

    let result = ops.add_ssh_key("New Key", "ssh-rsa AAAAB3...").await;

    teardown_token();

    assert!(result.is_ok());

    let key = result.unwrap();

    assert_eq!(key.id, 42);

    assert_eq!(key.title, "New Key");
}

#[tokio::test]
#[serial]
async fn test_identity_delete_ssh_key() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/user/keys/42"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.identity_ops().expect("identity ops");

    let result = ops.delete_ssh_key(42).await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_releases_fetch_by_tag() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/my-repo/releases/tags/v1.0.0"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "tag_name": "v1.0.0",
            "name": "Release 1.0.0",
            "body": "First release",
            "draft": false,
            "prerelease": false,
            "html_url": "https://github.com/o/r/releases/tag/v1.0.0"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.release_ops().expect("release ops");

    let result = ops.fetch_release_by_tag("testorg/my-repo", "v1.0.0").await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_environments_delete() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/repos/testorg/my-repo/environments/staging"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let _provider = GitHubProvider::with_base_url(&server.uri());
    let client = gitfleet_providers::github::ProviderClient::with_base_url(&server.uri());

    let result = gitfleet_providers::github::api::EnvironmentsApi::delete(
        &client, "testorg", "my-repo", "staging",
    )
    .await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_webhook_test() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/repos/testorg/my-repo/hooks/1/tests"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.webhook_ops().expect("webhook ops");

    let result = ops.test_webhook("testorg/my-repo", 1).await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_pipeline_cancel_run() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/repos/testorg/my-repo/actions/runs/101/cancel"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.pipeline_ops().expect("pipeline ops");

    let result = ops.cancel_run("testorg/my-repo", 101).await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_pipeline_rerun() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/repos/testorg/my-repo/actions/runs/101/rerun"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(201))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.pipeline_ops().expect("pipeline ops");

    let result = ops.rerun("testorg/my-repo", 101).await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_governance_create_ruleset() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/repos/testorg/my-repo/rulesets"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": 1,
            "name": "main-protection",
            "target": "branch",
            "enforcement": "active"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.governance_ops().expect("governance ops");

    let input = gitfleet_core::types::RulesetInput {
        name: "main-protection".to_string(),
        target: Some("branch".to_string()),
        rules: None,
        enforcement: Some("active".to_string()),
        conditions: None,
    };

    let result = ops.create_ruleset("testorg/my-repo", &input).await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_identity_list_gpg_keys() {
    let gpg_json = serde_json::json!([
        {
            "id": 1,
            "name": "My GPG Key",
            "key_id": "ABC123",
            "created_at": "2024-01-01T00:00:00Z"
        }
    ]);

    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/user/gpg_keys"))
        .and(query_param("per_page", "100"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(gpg_json))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.identity_ops().expect("identity ops");

    let result = ops.list_gpg_keys().await;

    teardown_token();

    assert!(result.is_ok());

    let keys = result.unwrap();

    assert_eq!(keys.len(), 1);

    assert_eq!(keys[0].name, "My GPG Key");
}

#[tokio::test]
#[serial]
async fn test_get_change() {
    let single_pr_json = serde_json::json!({
        "title": "Fix auth bug",
        "state": "open",
        "number": 42,
        "merged": false,
        "draft": false,
        "html_url": "https://github.com/testorg/my-repo/pull/42",
        "created_at": "2024-06-01T00:00:00Z",
        "updated_at": "2024-06-02T00:00:00Z",
        "body": "This fixes auth",
        "mergeable_state": "clean",
        "merged_at": null,
        "mergeable": true,
        "user": { "login": "contributor" },
        "maintainer_can_modify": true,
        "merge_commit_sha": null,
        "labels": [],
        "requested_reviewers": [],
        "head": {
            "ref": "fix-auth",
            "sha": "abc123",
            "repo": {
                "full_name": "contributor/my-repo",
                "html_url": "https://github.com/contributor/my-repo"
            }
        },
        "base": {
            "ref": "main",
            "repo": {
                "full_name": "testorg/my-repo",
                "html_url": "https://github.com/testorg/my-repo"
            }
        }
    });

    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/my-repo/pulls/42"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(single_pr_json))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.change_ops().expect("change ops");

    let result = ops.get_change("testorg/my-repo", 42).await;

    teardown_token();

    assert!(result.is_ok());

    let pr = result.unwrap();

    assert_eq!(pr.number, 42);

    assert_eq!(pr.title, "Fix auth bug");
    assert_eq!(pr.state, "open");

    assert!(!pr.merged);
}

#[tokio::test]
#[serial]
async fn test_create_change() {
    let created_pr_json = serde_json::json!({
        "title": "New feature",
        "state": "open",
        "number": 99,
        "merged": false,
        "draft": true,
        "html_url": "https://github.com/testorg/my-repo/pull/99",
        "created_at": "2024-07-01T00:00:00Z",
        "updated_at": "2024-07-01T00:00:00Z",
        "body": "Adds new feature",
        "mergeable_state": "unknown",
        "merged_at": null,
        "mergeable": null,
        "user": { "login": "dev" },
        "maintainer_can_modify": false,
        "merge_commit_sha": null,
        "labels": [],
        "requested_reviewers": [],
        "head": {
            "ref": "feature-branch",
            "sha": "def456",
            "repo": {
                "full_name": "dev/my-repo",
                "html_url": "https://github.com/dev/my-repo"
            }
        },
        "base": {
            "ref": "main",
            "repo": {
                "full_name": "testorg/my-repo",
                "html_url": "https://github.com/testorg/my-repo"
            }
        }
    });

    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/repos/testorg/my-repo/pulls"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(created_pr_json))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.change_ops().expect("change ops");

    let result = ops
        .create_change(
            "testorg/my-repo",
            "New feature",
            "feature-branch",
            "main",
            Some("Adds new feature"),
            true,
        )
        .await;

    teardown_token();

    assert!(result.is_ok());

    let pr = result.unwrap();

    assert_eq!(pr.number, 99);

    assert_eq!(pr.title, "New feature");
    assert_eq!(pr.draft, Some(true));
}

#[tokio::test]
#[serial]
async fn test_update_change() {
    let updated_pr_json = serde_json::json!({
        "title": "Updated title",
        "state": "open",
        "number": 42,
        "merged": false,
        "draft": false,
        "html_url": "https://github.com/testorg/my-repo/pull/42",
        "created_at": "2024-06-01T00:00:00Z",
        "updated_at": "2024-06-03T00:00:00Z",
        "body": "Updated body",
        "mergeable_state": "clean",
        "merged_at": null,
        "mergeable": true,
        "user": { "login": "contributor" },
        "maintainer_can_modify": true,
        "merge_commit_sha": null,
        "labels": [],
        "requested_reviewers": [],
        "head": {
            "ref": "fix-auth",
            "sha": "abc123",
            "repo": {
                "full_name": "contributor/my-repo",
                "html_url": "https://github.com/contributor/my-repo"
            }
        },
        "base": {
            "ref": "main",
            "repo": {
                "full_name": "testorg/my-repo",
                "html_url": "https://github.com/testorg/my-repo"
            }
        }
    });

    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/repos/testorg/my-repo/pulls/42"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(updated_pr_json))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.change_ops().expect("change ops");

    let result = ops
        .update_change(
            "testorg/my-repo",
            42,
            serde_json::json!({"title": "Updated title"}),
        )
        .await;

    teardown_token();

    assert!(result.is_ok());

    let pr = result.unwrap();

    assert_eq!(pr.title, "Updated title");
}

#[tokio::test]
#[serial]
async fn test_merge_change() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/repos/testorg/my-repo/pulls/42/merge"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"sha": "abc123", "merged": true})),
        )
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.change_ops().expect("change ops");

    let result = ops.merge_change("testorg/my-repo", 42, "squash").await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_comment_on_change() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/repos/testorg/my-repo/issues/42/comments"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(serde_json::json!({"id": 999, "body": "Nice work!"})),
        )
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.change_ops().expect("change ops");

    let result = ops
        .comment_on_change("testorg/my-repo", 42, "Nice work!")
        .await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_lock_change() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/repos/testorg/my-repo/issues/42/lock"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.change_ops().expect("change ops");

    let result = ops.lock_change("testorg/my-repo", 42).await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_unlock_change() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/repos/testorg/my-repo/issues/42/lock"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.change_ops().expect("change ops");

    let result = ops.unlock_change("testorg/my-repo", 42).await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_get_workflow() {
    let workflow_json = serde_json::json!({
        "id": 1234,
        "name": "CI",
        "path": ".github/workflows/ci.yml",
        "state": "active",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-06-01T00:00:00Z"
    });

    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/my-repo/actions/workflows/ci.yml"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(workflow_json))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.pipeline_ops().expect("pipeline ops");

    let result = ops.get_workflow("testorg/my-repo", "ci.yml").await;

    teardown_token();

    assert!(result.is_ok());

    let data = result.unwrap();

    assert_eq!(data["id"], 1234);

    assert_eq!(data["name"], "CI");
}

#[tokio::test]
#[serial]
async fn test_list_workflows() {
    let workflows_json = serde_json::json!({
        "total_count": 1,
        "workflows": [
            {
                "id": 1234,
                "name": "CI",
                "path": ".github/workflows/ci.yml",
                "state": "active"
            }
        ]
    });

    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/my-repo/actions/workflows"))
        .and(query_param("per_page", "10"))
        .and(query_param("page", "1"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(workflows_json))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.pipeline_ops().expect("pipeline ops");

    let result = ops.list_workflows("testorg/my-repo", 10, None).await;

    teardown_token();

    assert!(result.is_ok());

    let data = result.unwrap();

    assert_eq!(data["total_count"], 1);
}

#[tokio::test]
#[serial]
async fn test_get_run() {
    let run_json = serde_json::json!({
        "id": 101,
        "name": "CI",
        "status": "completed",
        "conclusion": "success",
        "head_branch": "main",
        "head_sha": "abc123"
    });

    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/my-repo/actions/runs/101"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(run_json))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.pipeline_ops().expect("pipeline ops");

    let result = ops.get_run("testorg/my-repo", 101).await;

    teardown_token();

    assert!(result.is_ok());

    let data = result.unwrap();

    assert_eq!(data["id"], 101);
}

#[tokio::test]
#[serial]
async fn test_delete_run() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/repos/testorg/my-repo/actions/runs/101"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.pipeline_ops().expect("pipeline ops");

    let result = ops.delete_run("testorg/my-repo", 101).await;

    teardown_token();

    assert!(result.is_ok());
}
fn gist_json() -> serde_json::Value {
    serde_json::json!({
        "id": "abc123",
        "description": "Test gist",
        "public": true,
        "html_url": "https://gist.github.com/abc123",
        "git_pull_url": "https://gist.github.com/abc123.git",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-02T00:00:00Z",
        "owner": { "login": "octocat" },
        "files": {
            "main.rs": {
                "filename": "main.rs",
                "type": "text/plain",
                "language": "Rust",
                "raw_url": "https://gist.github.com/raw/abc123/main.rs",
                "size": 42,
                "content": "fn main() {}"
            }
        }
    })
}

fn codespace_json() -> serde_json::Value {
    serde_json::json!({
        "codespaces": [
            {
                "id": 99,
                "name": "my-codespace",
                "state": "Available",
                "owner": { "login": "octocat" },
                "repository": { "full_name": "org/repo" },
                "branch": "main",
                "created_at": "2024-01-01T00:00:00Z",
                "idle_timeout_minutes": 60,
                "machine": { "name": "standardLinux" }
            }
        ]
    })
}

fn milestone_json() -> serde_json::Value {
    serde_json::json!([
        {
            "id": 1,
            "number": 1,
            "title": "v1.0",
            "state": "open",
            "description": "First release",
            "html_url": "https://github.com/testorg/repo/milestone/1",
            "due_on": "2024-12-31T00:00:00Z",
            "closed_at": null
        }
    ])
}

fn package_json() -> serde_json::Value {
    serde_json::json!([
        {
            "id": 123,
            "name": "my-package",
            "package_type": "npm",
            "version": "1.0.0",
            "html_url": "https://github.com/testorg/repo/packages/123",
            "owner": { "login": "testorg" },
            "repository": { "full_name": "testorg/repo" },
            "visibility": "public",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-02T00:00:00Z"
        }
    ])
}

fn reaction_json() -> serde_json::Value {
    serde_json::json!([
        {
            "id": 1,
            "node_id": "abc",
            "user": { "login": "octocat" },
            "content": "heart",
            "created_at": "2024-01-01T00:00:00Z"
        }
    ])
}

fn comment_json() -> serde_json::Value {
    serde_json::json!([
        {
            "id": 100,
            "body": "Great PR!",
            "user": { "login": "reviewer" },
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-02T00:00:00Z",
            "html_url": "https://github.com/testorg/repo/issues/1#issuecomment-100"
        }
    ])
}

// ===== RepoOps =====

#[tokio::test]
#[serial]
async fn test_get_repo() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/repo"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "name": "repo",
            "full_name": "testorg/repo",
            "private": false,
            "archived": false,
            "fork": false,
            "html_url": "https://github.com/testorg/repo",
            "clone_url": "https://github.com/testorg/repo.git",
            "default_branch": "main",
            "owner": { "login": "testorg" },
            "open_issues_count": 5,
            "stargazers_count": 10,
            "description": "Test repo",
            "pushed_at": "2024-01-01T00:00:00Z",
            "visibility": "public",
            "homepage": null,
            "parent": null
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().unwrap();

    let repo = ops.get_repo("testorg/repo").await.unwrap();

    teardown_token();

    assert_eq!(repo["name"], "repo");

    assert_eq!(repo["full_name"], "testorg/repo");
}

#[tokio::test]
#[serial]
async fn test_update_repo() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/repos/testorg/repo"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1, "name": "repo", "full_name": "testorg/repo", "private": true,
            "archived": false, "fork": false, "html_url": "https://github.com/testorg/repo",
            "clone_url": "https://github.com/testorg/repo.git", "default_branch": "main",
            "owner": {"login": "testorg"}, "open_issues_count": 5, "stargazers_count": 10,
            "description": "Updated", "pushed_at": "2024-01-01T00:00:00Z", "visibility": "private",
            "homepage": null, "parent": null
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().unwrap();

    let result = ops
        .update_repo(
            "testorg/repo",
            serde_json::json!({"description": "Updated"}),
        )
        .await
        .unwrap();

    teardown_token();

    assert_eq!(result["description"], "Updated");
}

#[tokio::test]
#[serial]
async fn test_delete_repo() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/repos/testorg/repo"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().unwrap();

    ops.delete_repo("testorg/repo").await.unwrap();

    teardown_token();
}

#[tokio::test]
#[serial]
async fn test_list_forks() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/repo/forks"))
        .and(query_param("per_page", "100"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!([{
                "id": 2,
                "name": "repo",
                "fork": true,
                "private": false,
                "archived": false,
                "full_name": "myuser/repo",
                "default_branch": "main"
            }])),
        )
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().unwrap();

    let forks = ops.list_forks("testorg/repo").await.unwrap();

    teardown_token();

    assert_eq!(forks.len(), 1);
    assert_eq!(forks[0].full_name, "myuser/repo");
    assert!(forks[0].fork);
}

#[tokio::test]
#[serial]
async fn test_fork_repo() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/repos/testorg/repo/forks"))
        .and(header("authorization", "Bearer testtoken"))
        .and(body_json(serde_json::json!({
            "organization": "destination-org"
        })))
        .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
            "id": 2, "name": "repo", "full_name": "myuser/repo", "private": false,
            "archived": false, "fork": true, "html_url": "https://github.com/myuser/repo",
            "clone_url": "https://github.com/myuser/repo.git", "default_branch": "main",
            "owner": {"login": "myuser"}, "open_issues_count": 0, "stargazers_count": 0,
            "description": "Fork", "pushed_at": "2024-01-01T00:00:00Z", "visibility": "public",
            "homepage": null, "parent": {"full_name": "testorg/repo"}
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().unwrap();

    let result = ops
        .fork_repo("testorg/repo", Some("destination-org"))
        .await
        .unwrap();

    teardown_token();

    assert_eq!(result["full_name"], "myuser/repo");
}

#[tokio::test]
#[serial]
async fn test_archive_repo() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/repos/testorg/repo"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
            "id": 1, "name": "repo", "full_name": "testorg/repo", "private": false,
            "archived": true, "fork": false, "html_url": "https://github.com/testorg/repo",
            "clone_url": "https://github.com/testorg/repo.git", "default_branch": "main",
            "owner": {"login": "testorg"}, "open_issues_count": 0, "stargazers_count": 0,
            "description": "", "pushed_at": "2024-01-01T00:00:00Z", "visibility": "public",
            "homepage": null, "parent": null
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().unwrap();

    ops.archive_repo("testorg/repo").await.unwrap();

    teardown_token();
}

#[tokio::test]
#[serial]
async fn test_unarchive_repo() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/repos/testorg/repo"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
            "id": 1, "name": "repo", "full_name": "testorg/repo", "private": false,
            "archived": false, "fork": false, "html_url": "https://github.com/testorg/repo",
            "clone_url": "https://github.com/testorg/repo.git", "default_branch": "main",
            "owner": {"login": "testorg"}, "open_issues_count": 0, "stargazers_count": 0,
            "description": "", "pushed_at": "2024-01-01T00:00:00Z", "visibility": "public",
            "homepage": null, "parent": null
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().unwrap();

    ops.unarchive_repo("testorg/repo").await.unwrap();

    teardown_token();
}

// ===== IssueOps =====

#[tokio::test]
#[serial]
async fn test_get_issue() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/repo/issues/1"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1, "number": 1, "title": "Bug", "state": "open",
            "body": "Fix needed", "html_url": "https://github.com/testorg/repo/issues/1",
            "user": {"login": "reporter"}, "labels": [], "assignees": [],
            "created_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-02T00:00:00Z"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.issue_ops().unwrap();

    let issue = ops.get_issue("testorg/repo", 1).await.unwrap();

    teardown_token();

    assert_eq!(issue["number"], 1);

    assert_eq!(issue["title"], "Bug");
}

#[tokio::test]
#[serial]
async fn test_create_issue() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/repos/testorg/repo/issues"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": 10, "number": 10, "title": "New issue", "state": "open",
            "body": "Description", "html_url": "https://github.com/testorg/repo/issues/10",
            "user": {"login": "testuser"}, "labels": [], "assignees": [],
            "created_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.issue_ops().unwrap();

    let issue = ops
        .create_issue("testorg/repo", "New issue", Some("Description"), &[], &[])
        .await
        .unwrap();

    teardown_token();

    assert_eq!(issue["number"], 10);
}

#[tokio::test]
#[serial]
async fn test_update_issue() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/repos/testorg/repo/issues/1"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1, "number": 1, "title": "Updated", "state": "closed",
            "body": "Fixed", "html_url": "https://github.com/testorg/repo/issues/1",
            "user": {"login": "reporter"}, "labels": [], "assignees": [],
            "created_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-02T00:00:00Z"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.issue_ops().unwrap();

    let issue = ops
        .update_issue("testorg/repo", 1, serde_json::json!({"state": "closed"}))
        .await
        .unwrap();

    teardown_token();

    assert_eq!(issue["state"], "closed");
}

#[tokio::test]
#[serial]
async fn test_comment_on_issue() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/repos/testorg/repo/issues/1/comments"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": 200, "body": "Nice!", "user": {"login": "testuser"},
            "created_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z",
            "html_url": "https://github.com/testorg/repo/issues/1#issuecomment-200"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.issue_ops().unwrap();

    let result = ops
        .comment_on_issue("testorg/repo", 1, "Nice!")
        .await
        .unwrap();

    teardown_token();

    assert_eq!(result["body"], "Nice!");
}

#[tokio::test]
#[serial]
async fn test_list_issue_comments() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/repo/issues/1/comments"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(comment_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.issue_ops().unwrap();

    let comments = ops.list_issue_comments("testorg/repo", 1).await.unwrap();

    teardown_token();

    assert!(comments.is_array());

    assert_eq!(comments.as_array().unwrap().len(), 1);
}

// ===== ChangeOps (list_change_comments) =====

#[tokio::test]
#[serial]
async fn test_list_change_comments() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/repo/pulls/42/comments"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(comment_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.change_ops().unwrap();

    let comments = ops.list_change_comments("testorg/repo", 42).await.unwrap();

    teardown_token();

    assert!(comments.is_array());
}

// ===== ReviewOps =====

#[tokio::test]
#[serial]
async fn test_list_reactions_for_issue() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/repo/issues/1/reactions"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(reaction_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.review_ops().unwrap();
    let reactions = ops
        .list_reactions_for_issue("testorg/repo", 1)
        .await
        .unwrap();

    teardown_token();

    assert_eq!(reactions.len(), 1);

    assert_eq!(reactions[0].content, "heart");
}

#[tokio::test]
#[serial]
async fn test_create_reaction_for_issue() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/repos/testorg/repo/issues/1/reactions"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": 5, "node_id": "abc", "user": {"login": "octocat"},
            "content": "+1", "created_at": "2024-01-01T00:00:00Z"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.review_ops().unwrap();

    let reaction = ops
        .create_reaction_for_issue("testorg/repo", 1, "+1")
        .await
        .unwrap();

    teardown_token();

    assert_eq!(reaction.content, "+1");
}

#[tokio::test]
#[serial]
async fn test_delete_reaction_for_issue() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/repos/testorg/repo/issues/1/reactions/5"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.review_ops().unwrap();

    ops.delete_reaction_for_issue("testorg/repo", 1, 5)
        .await
        .unwrap();

    teardown_token();
}

// ===== PlanningOps (Milestones) =====

#[tokio::test]
#[serial]
async fn test_list_milestones() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/repo/milestones"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(milestone_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.planning_ops().unwrap();

    let milestones = ops
        .list_milestones("testorg/repo", Some("open"), 10)
        .await
        .unwrap();

    teardown_token();

    assert_eq!(milestones.len(), 1);

    assert_eq!(milestones[0].title, "v1.0");
}

#[tokio::test]
#[serial]
async fn test_create_milestone() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/repos/testorg/repo/milestones"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "id": 2, "number": 2, "title": "v2.0", "state": "open",
                "description": "Second release", "html_url": "https://github.com/testorg/repo/milestone/2",
                "due_on": null, "closed_at": null
            })),
        )
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.planning_ops().unwrap();

    let ms = ops
        .create_milestone("testorg/repo", "v2.0", Some("Second release"))
        .await
        .unwrap();

    teardown_token();

    assert_eq!(ms.title, "v2.0");
}

#[tokio::test]
#[serial]
async fn test_get_milestone() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/repo/milestones/1"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": 1, "number": 1, "title": "v1.0", "state": "open",
                "description": "First release", "html_url": "https://github.com/testorg/repo/milestone/1",
                "due_on": "2024-12-31T00:00:00Z", "closed_at": null
            })),
        )
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.planning_ops().unwrap();

    let ms = ops.get_milestone("testorg/repo", 1).await.unwrap();

    teardown_token();

    assert_eq!(ms.title, "v1.0");

    assert_eq!(ms.number, 1);
}

#[tokio::test]
#[serial]
async fn test_update_milestone() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/repos/testorg/repo/milestones/1"))
        .and(header("authorization", "Bearer testtoken"))
        .and(body_json(serde_json::json!({"state": "closed"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1, "number": 1, "title": "v1.0-updated", "state": "closed",
            "description": "Released", "html_url": "https://github.com/testorg/repo/milestone/1",
            "due_on": null, "closed_at": "2024-06-01T00:00:00Z"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.planning_ops().unwrap();

    let ms = ops
        .update_milestone("testorg/repo", 1, serde_json::json!({"state": "closed"}))
        .await
        .unwrap();

    teardown_token();

    assert!(matches!(ms.state, MilestoneState::Closed));
}

#[tokio::test]
#[serial]
async fn test_delete_milestone() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/repos/testorg/repo/milestones/1"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.planning_ops().unwrap();

    ops.delete_milestone("testorg/repo", 1).await.unwrap();

    teardown_token();
}

// ===== PlanningOps (Projects - GraphQL) =====

#[test]
fn test_github_wiki_is_not_advertised() {
    let provider = GitHubProvider::new();

    assert!(provider.wiki_ops().is_none());
    assert!(!provider.capabilities().contains(&ProviderCapability::Wiki));
}

#[tokio::test]
#[serial]
async fn test_list_projects() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "repositoryOwner": {
                        "projectsV2": {
                            "nodes": [
                                {"id": "P_1", "number": 1, "title": "Project 1", "shortDescription": null,
                                 "closed": false, "url": "https://github.com/orgs/testorg/projects/1",
                                 "updatedAt": "2024-01-01T00:00:00Z"}
                            ]
                        }
                    }
                }
            })),
        )
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.planning_ops().unwrap();

    let projects = ops.list_projects("testorg", 10).await.unwrap();

    teardown_token();

    assert_eq!(projects.len(), 1);

    assert_eq!(projects[0].title, "Project 1");
}

#[tokio::test]
#[serial]
async fn test_get_project() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "node": {
                    "id": "P_1", "number": 1, "title": "Project 1", "shortDescription": null,
                    "closed": false, "url": "https://github.com/orgs/testorg/projects/1",
                    "updatedAt": "2024-01-01T00:00:00Z"
                }
            }
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.planning_ops().unwrap();

    let project = ops.get_project("P_1").await.unwrap();

    teardown_token();

    assert_eq!(project["data"]["node"]["title"], "Project 1");
}

#[tokio::test]
#[serial]
async fn test_create_project() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .and(header("authorization", "Bearer testtoken"))
        .and(body_string_contains("ProjectOwner"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"repositoryOwner": {"id": "O_1"}}
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .and(header("authorization", "Bearer testtoken"))
        .and(body_string_contains("CreateProject"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "createProjectV2": {
                    "projectV2": {
                        "id": "P_2", "number": 2, "title": "New Project", "shortDescription": null,
                        "closed": false, "url": "https://github.com/orgs/testorg/projects/2",
                        "updatedAt": "2024-01-01T00:00:00Z"
                    }
                }
            }
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.planning_ops().unwrap();

    let project = ops
        .create_project("testorg", "New Project", None)
        .await
        .unwrap();

    teardown_token();

    assert_eq!(project.title, "New Project");
}

#[tokio::test]
#[serial]
async fn test_delete_project() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"deleteProjectV2": {"clientMutationId": null}}
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.planning_ops().unwrap();

    ops.delete_project("P_1").await.unwrap();

    teardown_token();
}

#[tokio::test]
#[serial]
async fn test_delete_project_rejects_graphql_errors() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"deleteProjectV2": null},
            "errors": [{"message": "Resource not accessible"}]
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.planning_ops().unwrap();
    let result = ops.delete_project("P_1").await;

    teardown_token();

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Resource not accessible"));
}

// ===== ReleaseOps (update/delete) =====

#[tokio::test]
#[serial]
async fn test_update_release() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/repos/testorg/repo/releases/1"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": 1, "tag_name": "v1.1", "name": "v1.1", "body": "Updated",
                "draft": false, "prerelease": false, "html_url": "https://github.com/testorg/repo/releases/tag/v1.1",
                "created_at": "2024-01-01T00:00:00Z", "published_at": "2024-01-01T00:00:00Z",
                "author": {"login": "testuser"}, "assets": []
            })),
        )
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.release_ops().unwrap();

    let release = ops
        .update_release("testorg/repo", "1", serde_json::json!({"body": "Updated"}))
        .await
        .unwrap();

    teardown_token();

    assert_eq!(release["tag_name"], "v1.1");
}

#[tokio::test]
#[serial]
async fn test_delete_release() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/repos/testorg/repo/releases/1"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.release_ops().unwrap();

    ops.delete_release("testorg/repo", "1").await.unwrap();

    teardown_token();
}

#[tokio::test]
#[serial]
async fn test_delete_release_resolves_tag() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/repo/releases/tags/v1.0.0"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 17,
            "tag_name": "v1.0.0"
        })))
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/repos/testorg/repo/releases/17"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.release_ops().unwrap();

    ops.delete_release("testorg/repo", "v1.0.0").await.unwrap();

    teardown_token();
}

// ===== PolicyOps =====

#[tokio::test]
#[serial]
async fn test_get_branch_protection() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/repo/branches/main/protection"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "required_status_checks": {"strict": true, "contexts": ["ci"]},
            "enforce_admins": {"url": "", "enabled": true},
            "required_pull_request_reviews": {"required_approving_review_count": 1},
            "restrictions": null
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.policy_ops().unwrap();

    let result = ops
        .get_branch_protection("testorg/repo", "main")
        .await
        .unwrap();

    teardown_token();

    assert!(result["enforce_admins"]["enabled"].as_bool().unwrap());
}

#[tokio::test]
#[serial]
async fn test_protect_branch() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/repos/testorg/repo/branches/main/protection"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "required_status_checks": {"strict": true, "contexts": []},
            "enforce_admins": {"enabled": true},
            "required_pull_request_reviews": null,
            "restrictions": null
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.policy_ops().unwrap();

    let result = ops
        .protect_branch("testorg/repo", "main", serde_json::json!({}))
        .await
        .unwrap();

    teardown_token();

    assert!(result.is_object());
}

#[tokio::test]
#[serial]
async fn test_unprotect_branch() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/repos/testorg/repo/branches/main/protection"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.policy_ops().unwrap();

    ops.unprotect_branch("testorg/repo", "main").await.unwrap();

    teardown_token();
}

#[tokio::test]
#[serial]
async fn test_list_tag_protection() {
    let provider = GitHubProvider::new();
    let ops = provider.policy_ops().unwrap();

    let result = ops.list_tag_protection("testorg/repo").await;

    assert!(matches!(
        result,
        Err(gitfleet_core::errors::GitfleetError::UnsupportedCapability(
            _
        ))
    ));
}

#[tokio::test]
#[serial]
async fn test_create_tag_protection() {
    let provider = GitHubProvider::new();
    let ops = provider.policy_ops().unwrap();

    let result = ops.create_tag_protection("testorg/repo", "v*").await;

    assert!(matches!(
        result,
        Err(gitfleet_core::errors::GitfleetError::UnsupportedCapability(
            _
        ))
    ));
}

#[tokio::test]
#[serial]
async fn test_delete_tag_protection() {
    let provider = GitHubProvider::new();
    let ops = provider.policy_ops().unwrap();

    let result = ops.delete_tag_protection("testorg/repo", "42").await;

    assert!(matches!(
        result,
        Err(gitfleet_core::errors::GitfleetError::UnsupportedCapability(
            _
        ))
    ));
}

// ===== SiteOps =====

#[tokio::test]
#[serial]
async fn test_get_pages() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/repo/pages"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "url": "https://api.github.com/repos/testorg/repo/pages",
            "status": "built",
            "source": {"branch": "main", "path": "/docs"},
            "html_url": "https://testorg.github.io/repo/"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.site_ops().unwrap();

    let pages = ops.get_pages("testorg/repo").await.unwrap();

    teardown_token();

    assert_eq!(pages["status"], "built");
}

#[tokio::test]
#[serial]
async fn test_create_pages() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/repos/testorg/repo/pages"))
        .and(header("authorization", "Bearer testtoken"))
        .and(body_json(serde_json::json!({
            "source": {"branch": "main", "path": "/docs"}
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "url": "https://api.github.com/repos/testorg/repo/pages",
            "status": "queued",
            "source": {"branch": "main", "path": "/"},
            "html_url": "https://testorg.github.io/repo/"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.site_ops().unwrap();

    let pages = ops
        .create_pages("testorg/repo", "main/docs", None)
        .await
        .unwrap();

    teardown_token();

    assert_eq!(pages["status"], "queued");
}

#[tokio::test]
#[serial]
async fn test_remove_pages() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/repos/testorg/repo/pages"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.site_ops().unwrap();

    ops.remove_pages("testorg/repo").await.unwrap();

    teardown_token();
}

// ===== SnippetOps =====

#[tokio::test]
#[serial]
async fn test_list_snippets() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/users/testuser/gists"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([gist_json()])))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.snippet_ops().unwrap();

    let gists = ops.list_snippets("testuser").await.unwrap();

    teardown_token();

    assert!(!gists.is_empty());

    assert_eq!(gists[0].id, "abc123");
}

#[tokio::test]
#[serial]
async fn test_get_snippet() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/gists/abc123"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(gist_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.snippet_ops().unwrap();

    let gist = ops.get_snippet("abc123").await.unwrap();

    teardown_token();

    assert_eq!(gist["id"], "abc123");
}

#[tokio::test]
#[serial]
async fn test_create_snippet() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/gists"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(gist_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.snippet_ops().unwrap();

    let gist = ops
        .create_snippet(
            "Test gist",
            true,
            serde_json::json!({"main.rs": {"content": "fn main() {}"}}),
        )
        .await
        .unwrap();

    teardown_token();

    assert_eq!(gist.id, "abc123");
}

#[tokio::test]
#[serial]
async fn test_delete_snippet() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/gists/abc123"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.snippet_ops().unwrap();

    ops.delete_snippet("abc123").await.unwrap();

    teardown_token();
}

// ===== DevEnvOps (Codespaces) =====

#[tokio::test]
#[serial]
async fn test_list_codespaces() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/repo/codespaces"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(codespace_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.dev_env_ops().unwrap();

    let codespaces = ops.list_codespaces("testorg/repo").await.unwrap();

    teardown_token();

    assert_eq!(codespaces.len(), 1);

    assert_eq!(codespaces[0].name, "my-codespace");
}

#[tokio::test]
#[serial]
async fn test_create_codespace() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/repos/testorg/repo/codespaces"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": 100, "name": "new-cs", "state": "Available",
            "owner": {"login": "octocat"},
            "repository": {"full_name": "testorg/repo"},
            "branch": "main", "created_at": "2024-01-01T00:00:00Z",
            "idle_timeout_minutes": 30, "machine": {"name": "standardLinux"}
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.dev_env_ops().unwrap();

    let cs = ops
        .create_codespace("testorg/repo", Some("main"))
        .await
        .unwrap();

    teardown_token();

    assert_eq!(cs.name, "new-cs");

    assert_eq!(cs.state, "Available");
}

#[tokio::test]
#[serial]
async fn test_delete_codespace() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/user/codespaces/my-cs"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.dev_env_ops().unwrap();

    ops.delete_codespace("testorg/repo", "my-cs").await.unwrap();

    teardown_token();
}

// ===== RegistryOps (Packages) =====

#[tokio::test]
#[serial]
async fn test_list_packages() {
    let server = MockServer::start().await;

    for package_type in ["npm", "maven", "rubygems", "docker", "nuget", "container"] {
        let response = if package_type == "npm" {
            package_json()
        } else {
            serde_json::json!([])
        };

        Mock::given(method("GET"))
            .and(path("/orgs/testorg/packages"))
            .and(query_param("per_page", "10"))
            .and(query_param("package_type", package_type))
            .and(header("authorization", "Bearer testtoken"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&server)
            .await;
    }

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.registry_ops().unwrap();

    let packages = ops.list_packages("testorg", None, 10).await.unwrap();

    teardown_token();

    assert_eq!(packages.len(), 1);

    assert_eq!(packages[0].name, "my-package");
}

#[tokio::test]
#[serial]
async fn test_list_user_packages_falls_back_from_org_endpoint() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/orgs/testuser/packages"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;
    for package_type in ["npm", "maven", "rubygems", "docker", "nuget", "container"] {
        let response = if package_type == "npm" {
            package_json()
        } else {
            serde_json::json!([])
        };

        Mock::given(method("GET"))
            .and(path("/users/testuser/packages"))
            .and(query_param("per_page", "10"))
            .and(query_param("package_type", package_type))
            .and(header("authorization", "Bearer testtoken"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&server)
            .await;
    }

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let packages = provider
        .registry_ops()
        .unwrap()
        .list_packages("testuser", None, 10)
        .await
        .unwrap();

    teardown_token();

    assert_eq!(packages[0].name, "my-package");
}

#[tokio::test]
#[serial]
async fn test_get_package() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/orgs/testorg/packages/npm/my-package"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 123, "name": "my-package", "package_type": "npm", "version": "1.0.0",
            "html_url": "https://github.com/testorg/repo/packages/123",
            "owner": {"login": "testorg"},
            "repository": {"full_name": "testorg/repo"},
            "visibility": "public",
            "created_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-02T00:00:00Z"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.registry_ops().unwrap();

    let pkg = ops
        .get_package("testorg", "npm", "my-package")
        .await
        .unwrap();

    teardown_token();

    assert_eq!(pkg["name"], "my-package");
}

#[tokio::test]
#[serial]
async fn test_get_user_package_falls_back_from_org_endpoint() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/orgs/testuser/packages/npm/my-package"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/users/testuser/packages/npm/my-package"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 123,
            "name": "my-package",
            "package_type": "npm",
            "visibility": "public"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let package = provider
        .registry_ops()
        .unwrap()
        .get_package("testuser", "npm", "my-package")
        .await
        .unwrap();

    teardown_token();

    assert_eq!(package["name"], "my-package");
}

// ===== LicenseOps =====

#[tokio::test]
#[serial]
async fn test_list_licenses() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/licenses"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {"key": "mit", "name": "MIT License", "spdx_id": "MIT", "url": "https://api.github.com/licenses/mit", "html_url": "https://github.com/licenses/mit"}
            ])),
        )
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.license_ops().unwrap();

    let licenses = ops.list_licenses().await.unwrap();

    teardown_token();

    assert_eq!(licenses.len(), 1);

    assert_eq!(licenses[0].key, "mit");
}

#[tokio::test]
#[serial]
async fn test_get_license() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/licenses/mit"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "key": "mit", "name": "MIT License", "spdx_id": "MIT",
            "url": "https://api.github.com/licenses/mit",
            "html_url": "https://github.com/licenses/mit",
            "description": "A permissive license...",
            "implementation": "Create a text file LICENSE containing the license text.",
            "body": "MIT License\n\nCopyright (c) ...",
            "permissions": ["commercial-use", "modifications"],
            "conditions": ["include-copyright"],
            "limitations": ["liability"],
            "featured": true
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.license_ops().unwrap();

    let license = ops.get_license("mit").await.unwrap();

    teardown_token();

    assert_eq!(license.key, "mit");

    assert_eq!(license.name, "MIT License");
}

#[tokio::test]
#[serial]
async fn test_repo_license() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/repo/license"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "license": {"key": "mit", "name": "MIT License", "spdx_id": "MIT"}
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.license_ops().unwrap();

    let license = ops.repo_license("testorg/repo").await.unwrap();

    teardown_token();

    assert_eq!(license["license"]["key"], "mit");
}

// ===== DependencyOps =====

#[tokio::test]
#[serial]
async fn test_sbom() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/repo/dependency-graph/sbom"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "sbom": {"packages": [], "relationships": []}
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.dependency_ops().unwrap();

    let sbom = ops.sbom("testorg/repo").await.unwrap();

    teardown_token();

    assert!(sbom["sbom"].is_object());
}

#[tokio::test]
#[serial]
async fn test_review_dependencies() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(
            "/repos/testorg/repo/dependency-graph/compare/main...feature",
        ))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "change_type": "added",
                "name": "serde",
                "ecosystem": "cargo",
                "version": "1.0",
                "vulnerabilities": [{"severity": "high"}]
            }
        ])))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.dependency_ops().unwrap();

    let changes = ops
        .review_dependencies("testorg/repo", "main", "feature")
        .await
        .unwrap();

    teardown_token();

    assert_eq!(changes.len(), 1);

    assert_eq!(changes[0].package, "serde");
    assert_eq!(changes[0].change_type, "added");
    assert_eq!(changes[0].severity, "high");
    assert_eq!(changes[0].vulnerabilities, 1);
}

// ===== AdvisoryOps =====

#[tokio::test]
#[serial]
async fn test_list_dependabot_alerts() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/repo/dependabot/alerts"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {"number": 1, "state": "open", "severity": "high", "dependency": {"package": {"name": "lodash"}}}
            ])),
        )
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.advisory_ops().unwrap();

    let alerts = ops
        .list_dependabot_alerts("testorg/repo", None)
        .await
        .unwrap();

    teardown_token();

    assert!(alerts.is_array());
}

#[tokio::test]
#[serial]
async fn test_list_codeql_alerts() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/repo/code-scanning/alerts"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"number": 1, "state": "open", "rule": {"id": "sql-injection"}}
        ])))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.advisory_ops().unwrap();

    let alerts = ops.list_codeql_alerts("testorg/repo", None).await.unwrap();

    teardown_token();

    assert!(alerts.is_array());
}

#[tokio::test]
#[serial]
async fn test_list_secret_scanning_alerts() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/repo/secret-scanning/alerts"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"number": 1, "state": "open", "secret_type": "github_pat"}
        ])))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.advisory_ops().unwrap();

    let alerts = ops
        .list_secret_scanning_alerts("testorg/repo", None)
        .await
        .unwrap();

    teardown_token();

    assert!(alerts.is_array());
}

#[tokio::test]
#[serial]
async fn test_get_dependabot_alert() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/repo/dependabot/alerts/1"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "number": 1, "state": "open", "severity": "high"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.advisory_ops().unwrap();

    let alert = ops.get_dependabot_alert("testorg/repo", 1).await.unwrap();

    teardown_token();

    assert_eq!(alert["number"], 1);
}

// ===== AttestationOps =====

#[tokio::test]
#[serial]
async fn test_list_attestations() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/repo/attestations/sha256:abc"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"attestations": []})),
        )
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.attestation_ops().unwrap();

    let result = ops
        .list_attestations("testorg/repo", "sha256:abc")
        .await
        .unwrap();

    teardown_token();

    assert!(result["attestations"].is_array());
}

// ===== BrowseOps =====

#[tokio::test]
#[serial]
async fn test_list_contents() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/repo/contents"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"name": "README.md", "path": "README.md", "type": "file", "sha": "abc", "size": 100}
        ])))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.browse_ops().unwrap();

    let contents = ops.list_contents("testorg/repo", None).await.unwrap();

    teardown_token();

    assert!(contents.is_array());
}

#[tokio::test]
#[serial]
async fn test_list_contents_with_path() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/repo/contents/src"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"name": "main.rs", "path": "src/main.rs", "type": "file", "sha": "def", "size": 200}
        ])))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.browse_ops().unwrap();

    let contents = ops
        .list_contents("testorg/repo", Some("src"))
        .await
        .unwrap();

    teardown_token();

    assert!(contents.is_array());
}

#[tokio::test]
#[serial]
async fn test_file_contents() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/repo/contents/src/main.rs"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "name": "main.rs", "path": "src/main.rs", "type": "file",
            "sha": "abc", "size": 100, "content": "fn main() {}", "encoding": "base64"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.browse_ops().unwrap();

    let contents = ops
        .file_contents("testorg/repo", "src/main.rs", None)
        .await
        .unwrap();

    teardown_token();

    assert_eq!(contents["name"], "main.rs");
}

// ===== RawApiOps =====

#[tokio::test]
#[serial]
async fn test_raw_get() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/some/endpoint"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"result": "ok"})))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.raw_api_ops().unwrap();

    let result = ops.raw_get("/some/endpoint").await.unwrap();

    teardown_token();

    assert_eq!(result["result"], "ok");
}

#[tokio::test]
#[serial]
async fn test_raw_post() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/some/endpoint"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(serde_json::json!({"created": true})),
        )
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.raw_api_ops().unwrap();

    let result = ops
        .raw_post("/some/endpoint", serde_json::json!({"data": "value"}))
        .await
        .unwrap();

    teardown_token();

    assert_eq!(result["created"], true);
}

#[tokio::test]
#[serial]
async fn test_raw_put() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/some/endpoint"))
        .and(header("authorization", "Bearer testtoken"))
        .and(body_json(serde_json::json!({"data": "replacement"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "updated": true
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.raw_api_ops().unwrap();

    let result = ops
        .raw_put("/some/endpoint", serde_json::json!({"data": "replacement"}))
        .await
        .unwrap();

    teardown_token();

    assert_eq!(result["updated"], true);
}

#[tokio::test]
#[serial]
async fn test_raw_patch() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/some/endpoint"))
        .and(header("authorization", "Bearer testtoken"))
        .and(body_json(serde_json::json!({"data": "partial"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "updated": true
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.raw_api_ops().unwrap();

    let result = ops
        .raw_patch("/some/endpoint", serde_json::json!({"data": "partial"}))
        .await
        .unwrap();

    teardown_token();

    assert_eq!(result["updated"], true);
}

#[tokio::test]
#[serial]
async fn test_raw_delete() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/some/endpoint"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"deleted": true})),
        )
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.raw_api_ops().unwrap();

    let result = ops.raw_delete("/some/endpoint").await.unwrap();

    teardown_token();

    assert_eq!(result["deleted"], true);
}

// ===== TemplateOps =====

#[tokio::test]
#[serial]
async fn test_list_issue_templates() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"repository": {"issueTemplates": [
                {"name": "Bug Report", "filename": "bug.md", "body": "## Bug",
                 "about": "Report a bug", "title": "Bug: ",
                 "labels": {"nodes": [{"name": "bug"}]},
                 "assignees": {"nodes": [{"login": "octocat"}]}}
            ]}}
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.template_ops().unwrap();

    let templates = ops.list_issue_templates("testorg/repo").await.unwrap();

    teardown_token();

    assert_eq!(templates.len(), 1);

    assert_eq!(templates[0].name, "Bug Report");
    assert_eq!(templates[0].path, ".github/ISSUE_TEMPLATE/bug.md");
    assert_eq!(templates[0].labels, Some(vec!["bug".to_string()]));
    assert_eq!(templates[0].assignees, Some(vec!["octocat".to_string()]));
}

#[tokio::test]
#[serial]
async fn test_list_issue_templates_empty() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"repository": {"issueTemplates": []}}
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.template_ops().unwrap();

    let templates = ops.list_issue_templates("testorg/repo").await.unwrap();

    teardown_token();

    assert!(templates.is_empty());
}

// ===== NotificationOps (mark_read) =====

#[tokio::test]
#[serial]
async fn test_mark_notifications_read() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/notifications"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(205))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.notification_ops().unwrap();

    ops.mark_notifications_read().await.unwrap();

    teardown_token();
}

// ===== AccessOps (remaining) =====

#[tokio::test]
#[serial]
async fn test_remove_org_member() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/orgs/testorg/memberships/user1"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.access_ops().unwrap();

    ops.remove_org_member("testorg", "user1").await.unwrap();

    teardown_token();
}

#[tokio::test]
#[serial]
async fn test_list_teams() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/orgs/testorg/teams"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "name": "Engineering", "slug": "engineering", "permission": "push"}
        ])))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.access_ops().unwrap();

    let teams = ops.list_teams("testorg").await.unwrap();

    teardown_token();

    assert!(teams.is_array());
}

#[tokio::test]
#[serial]
async fn test_create_team() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/orgs/testorg/teams"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": 2, "name": "New Team", "slug": "new-team", "permission": "pull"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.access_ops().unwrap();

    let team = ops.create_team("testorg", "New Team").await.unwrap();

    teardown_token();

    assert_eq!(team["name"], "New Team");
}

#[tokio::test]
#[serial]
async fn test_list_team_members() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/orgs/testorg/teams/engineering/members"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"login": "user1", "type": "User"}
        ])))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.access_ops().unwrap();

    let members = ops
        .list_team_members("testorg", "engineering")
        .await
        .unwrap();

    teardown_token();

    assert!(members.is_array());
}

// ===== IdentityOps (remaining) =====

#[tokio::test]
#[serial]
async fn test_add_gpg_key() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/user/gpg_keys"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": 1, "key_id": "abc123", "public_key": "-----BEGIN PGP-----",
            "emails": [{"email": "test@example.com", "verified": true}],
            "created_at": "2024-01-01T00:00:00Z"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.identity_ops().unwrap();

    let key = ops.add_gpg_key("-----BEGIN PGP-----").await.unwrap();

    teardown_token();

    assert_eq!(key.key_id, "abc123");
}

#[tokio::test]
#[serial]
async fn test_delete_gpg_key() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/user/gpg_keys/1"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.identity_ops().unwrap();

    ops.delete_gpg_key(1).await.unwrap();

    teardown_token();
}

// ===== DiscussionOps (get/create) =====

#[tokio::test]
#[serial]
async fn test_get_discussion() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "repository": {
                    "discussion": {
                        "number": 42, "title": "Discussion title",
                        "body": "Discussion body", "category": {"name": "General"},
                        "url": "https://github.com/testorg/repo/discussions/42",
                        "createdAt": "2024-01-01T00:00:00Z",
                        "author": {"login": "user1"}
                    }
                }
            }
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.discussion_ops().unwrap();

    let discussion = ops.get_discussion("testorg", "repo", 42).await.unwrap();

    teardown_token();

    assert_eq!(discussion.number, 42);

    assert_eq!(discussion.title, "Discussion title");
}

#[tokio::test]
#[serial]
async fn test_create_discussion() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .and(header("authorization", "Bearer testtoken"))
        .and(body_string_contains("GetRepoId"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": { "repository": { "id": "R_kg123" } }
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .and(header("authorization", "Bearer testtoken"))
        .and(body_string_contains("mutation CreateDiscussion"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "createDiscussion": {
                    "discussion": {
                        "number": 100, "title": "New Discussion",
                        "body": "Body here", "category": {"name": "Ideas"},
                        "url": "https://github.com/testorg/repo/discussions/100",
                        "createdAt": "2024-01-01T00:00:00Z",
                        "author": {"login": "testuser"}
                    }
                }
            }
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.discussion_ops().unwrap();

    let discussion = ops
        .create_discussion(
            "testorg",
            "repo",
            "New Discussion",
            "Body here",
            Some("DIC_kw"),
        )
        .await
        .unwrap();

    teardown_token();

    assert_eq!(discussion.number, 100);

    assert_eq!(discussion.title, "New Discussion");
}

// ===== Insta snapshot tests for normalized wire payloads =====

#[tokio::test]
#[serial]
async fn snapshot_github_repo_normalization() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/org/repo"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1, "name": "repo", "full_name": "org/repo",
            "private": true, "html_url": "https://github.com/org/repo",
            "description": "Test repo", "fork": false, "archived": false,
            "default_branch": "main", "stargazers_count": 42
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().unwrap();

    let repo = ops.get_repo("org/repo").await.unwrap();

    teardown_token();

    insta::assert_json_snapshot!("github_repo_normalized", repo);
}

#[tokio::test]
#[serial]
async fn snapshot_github_issue_normalization() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/org/repo/issues/1"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1, "number": 1, "title": "Bug found", "body": "Something is wrong",
            "state": "open", "user": {"login": "octocat"},
            "labels": [{"name": "bug", "color": "#ff0000"}],
            "created_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-02T00:00:00Z",
            "html_url": "https://github.com/org/repo/issues/1"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.issue_ops().unwrap();

    let issue = ops.get_issue("org/repo", 1).await.unwrap();

    teardown_token();

    insta::assert_json_snapshot!("github_issue_normalized", issue);
}

#[tokio::test]
#[serial]
async fn snapshot_github_milestone_normalization() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/org/repo/milestones/1"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "url": "https://api.github.com/repos/org/repo/milestones/1",
            "html_url": "https://github.com/org/repo/milestone/1",
            "id": 1, "number": 1, "title": "v1.0",
            "state": "open", "open_issues": 3, "closed_issues": 7,
            "due_on": "2024-12-31T00:00:00Z"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.planning_ops().unwrap();

    let milestone = ops.get_milestone("org/repo", 1).await.unwrap();

    teardown_token();

    insta::assert_json_snapshot!("github_milestone_normalized", milestone);
}

// ===== WebhookOps (delete already tested, test create with different pattern) =====
