use gitfleet_core::provider::GitProvider;
use gitfleet_providers::GitHubProvider;
use serial_test::serial;
use wiremock::matchers::{header, method, path, query_param};
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
        .respond_with(ResponseTemplate::new(201).set_body_json(single_repo_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().expect("repo ops");

    let repo = ops
        .create_repo("new-repo", "public", None, None, Some("New repo"))
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
        .dispatch_workflow("testorg/my-repo", "ci.yml", "main", None)
        .await;

    teardown_token();

    assert!(result.is_ok());
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
            "url": "https://example.com/webhook",
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
        .await;

    teardown_token();

    assert!(result.is_ok());
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
