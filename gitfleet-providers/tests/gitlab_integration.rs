use gitfleet_core::provider::GitProvider;
use gitfleet_providers::GitLabProvider;
use serial_test::serial;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn setup_token() {
    std::env::set_var("GITFLEET_GITLAB_TOKEN", "testtoken");
}

fn teardown_token() {
    std::env::remove_var("GITFLEET_GITLAB_TOKEN");
}

fn project_json() -> serde_json::Value {
    serde_json::json!([
        {
            "id": 42,
            "name": "my-project",
            "path_with_namespace": "testgroup/my-project",
            "visibility": "private",
            "archived": false,
            "default_branch": "main",
            "last_activity_at": "2024-06-01T00:00:00Z",
            "forked_from_project": null
        }
    ])
}

fn single_project_json() -> serde_json::Value {
    serde_json::json!({
        "id": 99,
        "name": "new-project",
        "path_with_namespace": "testuser/new-project",
        "visibility": "public",
        "archived": false,
        "default_branch": "main",
        "last_activity_at": "2024-07-01T00:00:00Z"
    })
}

fn mr_json() -> serde_json::Value {
    serde_json::json!([
        {
            "title": "Fix login bug",
            "state": "merged",
            "iid": 7,
            "draft": false,
            "web_url": "https://gitlab.com/testgroup/my-project/-/merge_requests/7",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-02T00:00:00Z",
            "description": "This fixes the login bug",
            "detailed_merge_status": "mergeable",
            "merged_at": "2024-01-03T00:00:00Z",
            "author": { "username": "dev1" },
            "merge_commit_sha": "abc123",
            "labels": ["bug", "urgent"],
            "source_branch": "fix-login",
            "target_branch": "main",
            "sha": "def456"
        }
    ])
}

fn gitlab_issue_json() -> serde_json::Value {
    serde_json::json!([
        {
            "id": 100,
            "iid": 5,
            "title": "Bug in search",
            "state": "opened",
            "description": "Search returns wrong results",
            "labels": ["bug"],
            "created_at": "2024-06-01T00:00:00Z",
            "updated_at": "2024-06-02T00:00:00Z",
            "author": { "username": "reporter" }
        }
    ])
}

fn gitlab_label_json() -> serde_json::Value {
    serde_json::json!([
        {
            "id": 1,
            "name": "bug",
            "color": "#ff0000",
            "description": "Something isn't working"
        },
        {
            "id": 2,
            "name": "enhancement",
            "color": "#84b6eb",
            "description": "New feature or request"
        }
    ])
}

fn gitlab_pipeline_json() -> serde_json::Value {
    serde_json::json!([
        {
            "id": 101,
            "sha": "abc123",
            "ref": "main",
            "status": "success",
            "web_url": "https://gitlab.com/testgroup/my-project/-/pipelines/101",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T01:00:00Z"
        }
    ])
}

fn gitlab_webhook_json() -> serde_json::Value {
    serde_json::json!([
        {
            "id": 42,
            "url": "https://example.com/webhook",
            "push_events": true,
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-02T00:00:00Z"
        }
    ])
}

fn gitlab_release_json() -> serde_json::Value {
    serde_json::json!([
        {
            "tag_name": "v1.0.0",
            "name": "Release 1.0.0",
            "description": "First release",
            "created_at": "2024-01-01T00:00:00Z"
        }
    ])
}

fn gitlab_runner_json() -> serde_json::Value {
    serde_json::json!([
        {
            "id": 10,
            "description": "shared-runner-1",
            "platform": "linux",
            "status": "online",
            "is_active": true,
            "tag_list": ["docker", "shell"]
        }
    ])
}

fn gitlab_notification_json() -> serde_json::Value {
    serde_json::json!([
        {
            "id": 99,
            "project": { "path_with_namespace": "testgroup/my-project" },
            "body": "Merge request was approved",
            "target_type": "MergeRequest",
            "action_name": "approved",
            "state": "pending",
            "updated_at": "2024-06-01T00:00:00Z"
        }
    ])
}

fn gitlab_environment_json() -> serde_json::Value {
    serde_json::json!({
        "total_count": 1,
        "environments": [
            {
                "id": 1,
                "name": "production",
                "url": "https://example.com",
                "html_url": "https://gitlab.com/testgroup/my-project/-/environments/1",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            }
        ]
    })
}

fn gitlab_discussion_json() -> serde_json::Value {
    serde_json::json!([
        {
            "id": 1,
            "title": "How to contribute?",
            "iid": 1,
            "author": { "username": "octocat" },
            "notes": [
                {
                    "body": "Discussion body",
                    "created_at": "2024-01-01T00:00:00Z"
                }
            ],
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-02T00:00:00Z"
        }
    ])
}

fn gitlab_ssh_key_json() -> serde_json::Value {
    serde_json::json!([
        {
            "id": 5,
            "title": "My SSH Key",
            "key": "ssh-ed25519 AAAAC3...",
            "created_at": "2024-03-01T00:00:00Z"
        }
    ])
}

fn gitlab_wiki_json() -> serde_json::Value {
    serde_json::json!([
        {
            "title": "Home",
            "slug": "home",
            "format": "markdown"
        }
    ])
}

fn gitlab_wiki_page_json() -> serde_json::Value {
    serde_json::json!({
        "title": "Home",
        "slug": "home",
        "format": "markdown",
        "content": "Welcome to the wiki"
    })
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_group_repos() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/groups/testgroup/projects"))
        .and(query_param("per_page", "100"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(project_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().expect("repo ops");

    let repos = ops
        .list_org_repos("testgroup")
        .await
        .expect("list group repos");
    teardown_token();

    assert_eq!(repos.len(), 1);

    assert_eq!(repos[0].name, "my-project");
    assert_eq!(repos[0].full_name, "testgroup/my-project");

    assert!(repos[0].private);
    assert!(!repos[0].archived);

    assert_eq!(repos[0].default_branch, "main");
    assert_eq!(repos[0].id, 42);
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_user_repos() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects"))
        .and(query_param("membership", "true"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(project_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().expect("repo ops");

    let repos = ops.list_user_repos().await.expect("list user repos");

    teardown_token();

    assert_eq!(repos.len(), 1);

    assert_eq!(repos[0].name, "my-project");
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_user_named_repos() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/users/octocat/projects"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(project_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().expect("repo ops");

    let repos = ops
        .list_user_named_repos("octocat")
        .await
        .expect("list user named repos");
    teardown_token();

    assert_eq!(repos.len(), 1);

    assert_eq!(repos[0].full_name, "testgroup/my-project");
}

#[tokio::test]
#[serial]
async fn test_gitlab_get_repo() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(single_project_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().expect("repo ops");

    let result = ops.get_repo("testgroup/my-project").await;

    teardown_token();

    assert!(result.is_ok(), "get_repo failed: {:?}", result);

    let data = result.unwrap();

    assert_eq!(data["name"], "new-project");
}

#[tokio::test]
#[serial]
async fn test_gitlab_create_repo() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/projects"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(single_project_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().expect("repo ops");

    let result = ops
        .create_repo("new-project", "public", None, None, Some("New project"))
        .await;

    teardown_token();

    assert!(result.is_ok());

    let data = result.unwrap();

    assert_eq!(data["name"], "new-project");
}

#[tokio::test]
#[serial]
async fn test_gitlab_delete_repo() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/projects/testgroup%2Fmy-project"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(202))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().expect("repo ops");

    let result = ops.delete_repo("testgroup/my-project").await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_star_repo() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/projects/testgroup%2Fmy-project/star"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(single_project_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().expect("repo ops");

    let result = ops.star_repo("testgroup/my-project").await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_unstar_repo() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/projects/testgroup%2Fmy-project/unstar"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(single_project_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().expect("repo ops");

    let result = ops.unstar_repo("testgroup/my-project").await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_changes() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/testgroup%2Fmy-project/merge_requests"))
        .and(query_param("state", "opened"))
        .and(query_param("per_page", "100"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mr_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.change_ops().expect("change ops");

    let pulls = ops
        .list_changes("testgroup/my-project", "opened", 100, None, None)
        .await
        .expect("list changes");
    teardown_token();

    assert_eq!(pulls.len(), 1);

    assert_eq!(pulls[0].title, "Fix login bug");
    assert_eq!(pulls[0].state, "merged");

    assert!(pulls[0].merged);
    assert_eq!(pulls[0].number, 7);

    assert_eq!(pulls[0].head.r#ref, "fix-login");
    assert_eq!(pulls[0].base.r#ref, "main");

    assert_eq!(pulls[0].user.as_ref().unwrap().login, "dev1");
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_changes_with_branches() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/testgroup%2Fmy-project/merge_requests"))
        .and(query_param("state", "opened"))
        .and(query_param("per_page", "50"))
        .and(query_param("target_branch", "main"))
        .and(query_param("source_branch", "fix-login"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mr_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.change_ops().expect("change ops");

    let pulls = ops
        .list_changes(
            "testgroup/my-project",
            "opened",
            50,
            Some("main"),
            Some("fix-login"),
        )
        .await
        .expect("list changes with branches");
    teardown_token();

    assert_eq!(pulls.len(), 1);
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_changes_empty() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/testgroup%2Fmy-project/merge_requests"))
        .and(query_param("state", "opened"))
        .and(query_param("per_page", "100"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.change_ops().expect("change ops");

    let pulls = ops
        .list_changes("testgroup/my-project", "opened", 100, None, None)
        .await
        .expect("list empty changes");
    teardown_token();

    assert!(pulls.is_empty());
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_issues() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/testgroup%2Fmy-project/issues"))
        .and(query_param("state", "opened"))
        .and(query_param("per_page", "30"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(gitlab_issue_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.issue_ops().expect("issue ops");

    let result = ops
        .list_issues("testgroup/my-project", "opened", 30, &[], &[])
        .await
        .expect("list issues");
    teardown_token();

    let items = result.as_array().expect("issues array");

    assert_eq!(items.len(), 1);

    assert_eq!(items[0]["title"], "Bug in search");
}

#[tokio::test]
#[serial]
async fn test_gitlab_create_issue() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/projects/testgroup%2Fmy-project/issues"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": 200,
            "iid": 6,
            "title": "New bug",
            "description": "Details here",
            "state": "opened"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.issue_ops().expect("issue ops");

    let result = ops
        .create_issue(
            "testgroup/my-project",
            "New bug",
            Some("Details here"),
            &[],
            &[],
        )
        .await;

    teardown_token();

    assert!(result.is_ok());

    let data = result.unwrap();

    assert_eq!(data["title"], "New bug");
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_labels() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/testgroup%2Fmy-project/labels"))
        .respond_with(ResponseTemplate::new(200).set_body_json(gitlab_label_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.label_ops().expect("label ops");

    let labels = ops
        .list_labels("testgroup/my-project")
        .await
        .expect("list labels");
    teardown_token();

    assert_eq!(labels.len(), 2);

    assert_eq!(labels[0].name, "bug");
    assert_eq!(labels[0].color, "#ff0000");

    assert_eq!(labels[1].name, "enhancement");
}

#[tokio::test]
#[serial]
async fn test_gitlab_create_label() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/projects/testgroup%2Fmy-project/labels"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": 3,
            "name": "enhancement",
            "color": "#a2eeef",
            "description": "New feature"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.label_ops().expect("label ops");

    let label = gitfleet_core::types::Label {
        name: "enhancement".to_string(),
        color: "#a2eeef".to_string(),
        description: "New feature".to_string(),
        new_name: None,
    };

    let result = ops.create_label(&label, "testgroup/my-project").await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_delete_label() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/projects/testgroup%2Fmy-project/labels"))
        .and(query_param("search", "bug"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.label_ops().expect("label ops");

    let result = ops.delete_label("bug", "testgroup/my-project").await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_pipelines() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/testgroup%2Fmy-project/pipelines"))
        .and(query_param("per_page", "30"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(gitlab_pipeline_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.pipeline_ops().expect("pipeline ops");

    let result = ops.list_workflows("testgroup/my-project", 30, None).await;

    teardown_token();

    assert!(result.is_ok());

    let data = result.unwrap();

    let items = data.as_array().expect("pipelines array");

    assert_eq!(items.len(), 1);

    assert_eq!(items[0]["id"], 101);
}

#[tokio::test]
#[serial]
async fn test_gitlab_dispatch_pipeline() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/projects/testgroup%2Fmy-project/pipeline"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": 200,
            "sha": "abc123",
            "ref": "main",
            "status": "pending"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.pipeline_ops().expect("pipeline ops");

    let result = ops
        .dispatch_workflow("testgroup/my-project", "ci.yml", "main", None)
        .await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_releases() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/testgroup%2Fmy-project/releases"))
        .and(query_param("per_page", "10"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(gitlab_release_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.release_ops().expect("release ops");

    let result = ops.list_releases("testgroup/my-project", 10).await;

    teardown_token();

    assert!(result.is_ok());

    let data = result.unwrap();

    let items = data.as_array().expect("releases array");

    assert_eq!(items.len(), 1);

    assert_eq!(items[0]["tag_name"], "v1.0.0");
}

#[tokio::test]
#[serial]
async fn test_gitlab_create_release() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/projects/testgroup%2Fmy-project/releases"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "tag_name": "v2.0.0",
            "name": "Release 2.0.0",
            "description": "Second release"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.release_ops().expect("release ops");

    let result = ops
        .create_release(
            "testgroup/my-project",
            serde_json::json!({
                "tag_name": "v2.0.0",
                "name": "Release 2.0.0",
                "description": "Second release"
            }),
        )
        .await;

    teardown_token();

    assert!(result.is_ok());

    let data = result.unwrap();

    assert_eq!(data["tag_name"], "v2.0.0");
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_webhooks() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/testgroup%2Fmy-project/hooks"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(gitlab_webhook_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.webhook_ops().expect("webhook ops");

    let result = ops.list_webhooks("testgroup/my-project").await;

    teardown_token();

    assert!(result.is_ok());

    let hooks = result.unwrap();

    assert_eq!(hooks.len(), 1);

    assert_eq!(hooks[0].id, 42);
    assert_eq!(hooks[0].url, "https://example.com/webhook");

    assert!(hooks[0].active);
}

#[tokio::test]
#[serial]
async fn test_gitlab_create_webhook() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/projects/testgroup%2Fmy-project/hooks"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": 43,
            "url": "https://example.com/new-hook",
            "push_events": true,
            "created_at": "2024-02-01T00:00:00Z",
            "updated_at": "2024-02-01T00:00:00Z"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.webhook_ops().expect("webhook ops");

    let result = ops
        .create_webhook(
            "testgroup/my-project",
            serde_json::json!({
                "url": "https://example.com/new-hook",
                "push_events": true
            }),
        )
        .await;

    teardown_token();

    assert!(result.is_ok());

    let hook = result.unwrap();

    assert_eq!(hook.id, 43);

    assert_eq!(hook.url, "https://example.com/new-hook");
}

#[tokio::test]
#[serial]
async fn test_gitlab_remove_webhook() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/projects/testgroup%2Fmy-project/hooks/42"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.webhook_ops().expect("webhook ops");

    let result = ops.remove_webhook("testgroup/my-project", 42).await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_runners() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/testgroup%2Fmy-project/runners"))
        .and(query_param("per_page", "100"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(gitlab_runner_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.runner_ops().expect("runner ops");

    let result = ops.list_runners("testgroup/my-project").await;

    teardown_token();

    assert!(result.is_ok());

    let runners = result.unwrap();

    assert_eq!(runners.len(), 1);

    assert_eq!(runners[0].id, 10);
    assert_eq!(runners[0].name, "shared-runner-1");

    assert_eq!(runners[0].os, "linux");
    assert_eq!(runners[0].status, "online");

    assert!(runners[0].busy);
    assert_eq!(runners[0].labels, vec!["docker", "shell"]);
}

#[tokio::test]
#[serial]
async fn test_gitlab_remove_runner() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/projects/testgroup%2Fmy-project/runners/10"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.runner_ops().expect("runner ops");

    let result = ops.remove_runner("testgroup/my-project", 10).await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_notifications() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/todos"))
        .and(query_param("state", "all"))
        .and(query_param("per_page", "100"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(gitlab_notification_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.notification_ops().expect("notification ops");

    let notifications = ops
        .list_notifications(true, false, None)
        .await
        .expect("list todos");
    teardown_token();

    assert_eq!(notifications.len(), 1);

    assert_eq!(notifications[0].id, "99");
    assert_eq!(notifications[0].repository, "testgroup/my-project");

    assert_eq!(notifications[0].subject_title, "Merge request was approved");
    assert_eq!(notifications[0].subject_type, "MergeRequest");

    assert!(notifications[0].unread);
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_notifications_empty() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/todos"))
        .and(query_param("per_page", "100"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.notification_ops().expect("notification ops");

    let notifications = ops
        .list_notifications(false, false, None)
        .await
        .expect("list empty todos");
    teardown_token();

    assert!(notifications.is_empty());
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_environments() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/testgroup%2Fmy-project/environments"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(gitlab_environment_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.environment_ops().expect("environment ops");

    let result = ops.list_environments("testgroup", "my-project").await;

    teardown_token();

    assert!(result.is_ok());

    let envs = result.unwrap();

    assert_eq!(envs.total_count, 1);

    assert_eq!(envs.environments.len(), 1);
    assert_eq!(envs.environments[0].name, "production");
}

#[tokio::test]
#[serial]
async fn test_gitlab_create_environment() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/projects/testgroup%2Fmy-project/environments"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": 2,
            "name": "staging",
            "url": "https://staging.example.com",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.environment_ops().expect("environment ops");

    let result = ops
        .create_environment("testgroup", "my-project", "staging", None)
        .await;

    teardown_token();

    assert!(result.is_ok());

    let data = result.unwrap();

    assert_eq!(data["name"], "staging");
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_discussions() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/testgroup%2Fmy-project/issues"))
        .and(query_param("per_page", "10"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(gitlab_discussion_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.discussion_ops().expect("discussion ops");

    let result = ops
        .list_discussions("testgroup", "my-project", None, 10)
        .await;

    teardown_token();

    assert!(result.is_ok());

    let discussions = result.unwrap();

    assert_eq!(discussions.len(), 1);

    assert_eq!(discussions[0].title, "How to contribute?");
    assert_eq!(discussions[0].author, "octocat");
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_ssh_keys() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/user/keys"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(gitlab_ssh_key_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.identity_ops().expect("identity ops");

    let result = ops.list_ssh_keys().await;

    teardown_token();

    assert!(result.is_ok());

    let keys = result.unwrap();

    assert_eq!(keys.len(), 1);

    assert_eq!(keys[0].id, 5);
    assert_eq!(keys[0].title, "My SSH Key");
}

#[tokio::test]
#[serial]
async fn test_gitlab_add_ssh_key() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/user/keys"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": 42,
            "title": "New Key",
            "key": "ssh-ed25519 AAAAB3...",
            "created_at": "2024-01-01T00:00:00Z"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.identity_ops().expect("identity ops");

    let result = ops.add_ssh_key("New Key", "ssh-ed25519 AAAAB3...").await;

    teardown_token();

    assert!(result.is_ok());

    let key = result.unwrap();

    assert_eq!(key.id, 42);

    assert_eq!(key.title, "New Key");
}

#[tokio::test]
#[serial]
async fn test_gitlab_delete_ssh_key() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/user/keys/42"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.identity_ops().expect("identity ops");

    let result = ops.delete_ssh_key(42).await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_wiki_pages() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/testgroup%2Fmy-project/wikis"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(gitlab_wiki_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.wiki_ops().expect("wiki ops");

    let result = ops.list_wiki_pages("testgroup/my-project").await;

    teardown_token();

    assert!(result.is_ok());

    let pages = result.unwrap();

    assert_eq!(pages.len(), 1);

    assert_eq!(pages[0].title, "Home");
    assert_eq!(pages[0].path, "home");
}

#[tokio::test]
#[serial]
async fn test_gitlab_get_wiki_page() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/testgroup%2Fmy-project/wikis/home"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(gitlab_wiki_page_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.wiki_ops().expect("wiki ops");

    let result = ops.get_wiki_page("testgroup/my-project", "home").await;

    teardown_token();

    assert!(result.is_ok());

    let page = result.unwrap();

    assert_eq!(page.page.title, "Home");

    assert_eq!(page.content, "Welcome to the wiki");
}

#[tokio::test]
#[serial]
async fn test_gitlab_create_wiki_page() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/projects/testgroup%2Fmy-project/wikis"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "title": "New Page",
            "slug": "new-page",
            "format": "markdown",
            "content": "Page content"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.wiki_ops().expect("wiki ops");

    let result = ops
        .create_wiki_page("testgroup/my-project", "New Page", "Page content")
        .await;

    teardown_token();

    assert!(result.is_ok());

    let page = result.unwrap();

    assert_eq!(page.page.title, "New Page");

    assert_eq!(page.content, "Page content");
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_variables() {
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
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(variables_json))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.variable_ops().expect("variable ops");

    let result = ops.list_repo_variables("testgroup", "my-project").await;

    teardown_token();

    assert!(result.is_ok(), "list_repo_variables failed: {:?}", result);

    let vars = result.unwrap();

    assert_eq!(vars.total_count, 1);

    assert_eq!(vars.variables.len(), 1);
    assert_eq!(vars.variables[0].name, "MY_VAR");
}

#[tokio::test]
#[serial]
async fn test_gitlab_set_variable() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/projects/testgroup%2Fmy-project/variables"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "key": "MY_VAR",
            "value": "hello"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.variable_ops().expect("variable ops");

    let result = ops
        .set_repo_variable("testgroup", "my-project", "MY_VAR", "hello")
        .await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_delete_variable() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/projects/testgroup%2Fmy-project/variables/MY_VAR"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.variable_ops().expect("variable ops");

    let result = ops
        .delete_repo_variable("testgroup", "my-project", "MY_VAR")
        .await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_invite_collaborator() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/projects/testgroup%2Fmy-project/members"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(201))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.access_ops().expect("access ops");

    let result = ops
        .invite_collaborator("testgroup", "my-project", "newuser", "admin")
        .await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_search_issues() {
    let search_json = serde_json::json!([
        {
            "id": 100,
            "iid": 5,
            "title": "Bug in search",
            "state": "opened",
            "web_url": "https://gitlab.com/testgroup/my-project/-/issues/5"
        }
    ]);

    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/search"))
        .and(query_param("scope", "issues"))
        .and(query_param("search", "bug"))
        .and(query_param("per_page", "30"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(search_json))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.search_ops().expect("search ops");

    let result = ops.search_issues("bug", None, None, 30).await;

    teardown_token();

    assert!(result.is_ok());

    let search = result.unwrap();

    assert_eq!(search.total_count, 1);
}

#[tokio::test]
#[serial]
async fn test_gitlab_search_repos() {
    let search_json = serde_json::json!([
        {
            "id": 42,
            "name": "my-project",
            "path_with_namespace": "testgroup/my-project"
        }
    ]);

    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/search"))
        .and(query_param("scope", "projects"))
        .and(query_param("search", "my-project"))
        .and(query_param("per_page", "30"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(search_json))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.search_ops().expect("search ops");

    let result = ops.search_repos("my-project", None, None, 30).await;

    teardown_token();

    assert!(result.is_ok());

    let search = result.unwrap();

    assert_eq!(search.total_count, 1);
}

#[tokio::test]
#[serial]
async fn test_gitlab_repo_not_found_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/groups/nonexistent/projects"))
        .and(query_param("per_page", "100"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().expect("repo ops");

    let result = ops.list_org_repos("nonexistent").await;

    teardown_token();

    assert!(result.is_err());

    let err = result.unwrap_err();

    assert!(err.to_string().to_lowercase().contains("not found"));
}

#[tokio::test]
#[serial]
async fn test_gitlab_repo_unauthorized_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/groups/privategroup/projects"))
        .and(query_param("per_page", "100"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().expect("repo ops");

    let result = ops.list_org_repos("privategroup").await;

    teardown_token();

    assert!(result.is_err());
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_org_repos_empty() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/groups/emptygroup/projects"))
        .and(query_param("per_page", "100"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().expect("repo ops");

    let repos = ops
        .list_org_repos("emptygroup")
        .await
        .expect("list empty group repos");
    teardown_token();

    assert!(repos.is_empty());
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_labels_empty() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/testgroup%2Fmy-project/labels"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.label_ops().expect("label ops");

    let labels = ops
        .list_labels("testgroup/my-project")
        .await
        .expect("list empty labels");
    teardown_token();

    assert!(labels.is_empty());
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_runners_empty() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/testgroup%2Fmy-project/runners"))
        .and(query_param("per_page", "100"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.runner_ops().expect("runner ops");

    let result = ops.list_runners("testgroup/my-project").await;

    teardown_token();

    assert!(result.is_ok());

    let runners = result.unwrap();

    assert!(runners.is_empty());
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_gpg_keys() {
    let gpg_json = serde_json::json!([
        {
            "id": 10,
            "description": "My GPG Key",
            "key_id": "DEF456",
            "created_at": "2024-04-01T00:00:00Z"
        }
    ]);

    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/user/gpg_keys"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(gpg_json))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.identity_ops().expect("identity ops");

    let result = ops.list_gpg_keys().await;

    teardown_token();

    assert!(result.is_ok());

    let keys = result.unwrap();

    assert_eq!(keys.len(), 1);

    assert_eq!(keys[0].name, "My GPG Key");
    assert_eq!(keys[0].key_id, "DEF456");
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_group_members() {
    let members_json = serde_json::json!([
        {
            "id": 1,
            "username": "member1",
            "name": "Member One",
            "access_level": 30
        }
    ]);

    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/groups/testgroup/members"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(members_json))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.access_ops().expect("access ops");

    let result = ops.list_org_members("testgroup").await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_team_members() {
    let team_members_json = serde_json::json!([
        {
            "id": 1,
            "username": "dev1",
            "name": "Developer One",
            "access_level": 30
        }
    ]);

    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/groups/mygroup/members"))
        .and(query_param("per_page", "100"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(team_members_json))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.access_ops().expect("access ops");

    let result = ops.list_team_members("mygroup", "team-slug").await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_archive_repo() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/projects/testgroup%2Fmy-project/archive"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(single_project_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().expect("repo ops");

    let result = ops.archive_repo("testgroup/my-project").await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_unarchive_repo() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/projects/testgroup%2Fmy-project/unarchive"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(single_project_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().expect("repo ops");

    let result = ops.unarchive_repo("testgroup/my-project").await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_fork_repo() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/projects/testgroup%2Fmy-project/fork"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(single_project_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().expect("repo ops");

    let result = ops.fork_repo("testgroup/my-project").await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_mark_notifications_read() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("//mark_todos_as_done"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.notification_ops().expect("notification ops");

    let result = ops.mark_notifications_read().await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_notifications_with_repo() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/testgroup%2Fmy-project/todos"))
        .and(query_param("per_page", "100"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(gitlab_notification_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.notification_ops().expect("notification ops");

    let notifications = ops
        .list_notifications(false, false, Some("testgroup/my-project"))
        .await
        .expect("list repo todos");
    teardown_token();

    assert_eq!(notifications.len(), 1);

    assert_eq!(notifications[0].repository, "testgroup/my-project");
}

#[tokio::test]
#[serial]
async fn test_gitlab_create_repo_in_group() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/groups/testgroup"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(single_project_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().expect("repo ops");

    let result = ops
        .create_repo(
            "group-project",
            "private",
            Some("testgroup"),
            Some("group"),
            Some("A group project"),
        )
        .await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_update_repo() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/projects/testgroup%2Fmy-project"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 42,
            "name": "my-project-updated",
            "path_with_namespace": "testgroup/my-project-updated"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.repo_ops().expect("repo ops");

    let result = ops
        .update_repo(
            "testgroup/my-project",
            serde_json::json!({"name": "my-project-updated"}),
        )
        .await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_get_change() {
    let single_mr_json = serde_json::json!({
        "title": "Fix login bug",
        "state": "opened",
        "iid": 7,
        "draft": false,
        "web_url": "https://gitlab.com/testgroup/my-project/-/merge_requests/7",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-02T00:00:00Z",
        "description": "This fixes the login bug",
        "detailed_merge_status": "mergeable",
        "merged_at": null,
        "author": { "username": "dev1" },
        "merge_commit_sha": "abc123",
        "labels": [],
        "source_branch": "fix-login",
        "target_branch": "main",
        "sha": "def456"
    });

    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/testgroup%2Fmy-project/merge_requests/7"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(single_mr_json))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.change_ops().expect("change ops");

    let result = ops.get_change("testgroup/my-project", 7).await;

    teardown_token();

    assert!(result.is_ok());

    let mr = result.unwrap();

    assert_eq!(mr.title, "Fix login bug");

    assert_eq!(mr.number, 7);
    assert_eq!(mr.head.r#ref, "fix-login");

    assert_eq!(mr.base.r#ref, "main");
}

#[tokio::test]
#[serial]
async fn test_gitlab_create_change() {
    let created_mr_json = serde_json::json!({
        "title": "New feature",
        "state": "opened",
        "iid": 99,
        "draft": true,
        "web_url": "https://gitlab.com/testgroup/my-project/-/merge_requests/99",
        "created_at": "2024-07-01T00:00:00Z",
        "updated_at": "2024-07-01T00:00:00Z",
        "description": "Adds new feature",
        "detailed_merge_status": "unknown",
        "merged_at": null,
        "author": { "username": "dev" },
        "merge_commit_sha": null,
        "labels": [],
        "source_branch": "feature-branch",
        "target_branch": "main",
        "sha": "def456"
    });

    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/projects/testgroup%2Fmy-project/merge_requests"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(created_mr_json))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.change_ops().expect("change ops");

    let result = ops
        .create_change(
            "testgroup/my-project",
            "New feature",
            "feature-branch",
            "main",
            Some("Adds new feature"),
            true,
        )
        .await;

    teardown_token();

    assert!(result.is_ok());

    let mr = result.unwrap();

    assert_eq!(mr.number, 99);

    assert_eq!(mr.title, "New feature");
    assert_eq!(mr.draft, Some(true));
}

#[tokio::test]
#[serial]
async fn test_gitlab_merge_change() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path(
            "/projects/testgroup%2Fmy-project/merge_requests/7/merge",
        ))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "state": "merged",
            "iid": 7
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.change_ops().expect("change ops");

    let result = ops.merge_change("testgroup/my-project", 7, "squash").await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_comment_on_change() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path(
            "/projects/testgroup%2Fmy-project/merge_requests/7/notes",
        ))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": 999,
            "body": "Nice work!"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.change_ops().expect("change ops");

    let result = ops
        .comment_on_change("testgroup/my-project", 7, "Nice work!")
        .await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_change_comments() {
    let comments_json = serde_json::json!([
        {
            "id": 1,
            "body": "Looks good",
            "author": { "username": "reviewer" }
        }
    ]);

    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(
            "/projects/testgroup%2Fmy-project/merge_requests/7/notes",
        ))
        .and(query_param("per_page", "100"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(comments_json))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.change_ops().expect("change ops");

    let result = ops.list_change_comments("testgroup/my-project", 7).await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_create_issue_with_labels() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/projects/testgroup%2Fmy-project/issues"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": 200,
            "iid": 6,
            "title": "New bug",
            "description": "Details here",
            "state": "opened"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.issue_ops().expect("issue ops");

    let result = ops
        .create_issue(
            "testgroup/my-project",
            "New bug",
            Some("Details here"),
            &["bug".to_string()],
            &["1".to_string()],
        )
        .await;

    teardown_token();

    assert!(result.is_ok());

    let data = result.unwrap();

    assert_eq!(data["title"], "New bug");
}

#[tokio::test]
#[serial]
async fn test_gitlab_delete_wiki_page() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/projects/testgroup%2Fmy-project/wikis/old-page"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.wiki_ops().expect("wiki ops");

    let result = ops
        .delete_wiki_page("testgroup/my-project", "old-page")
        .await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_update_wiki_page() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/projects/testgroup%2Fmy-project/wikis/home"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "title": "Home",
            "slug": "home",
            "format": "markdown",
            "content": "Updated content"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.wiki_ops().expect("wiki ops");

    let result = ops
        .update_wiki_page("testgroup/my-project", "home", "Updated content")
        .await;

    teardown_token();

    assert!(result.is_ok());

    let page = result.unwrap();

    assert_eq!(page.content, "Updated content");
}

#[tokio::test]
#[serial]
async fn test_gitlab_get_issue() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/testgroup%2Fmy-project/issues/5"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 100,
            "iid": 5,
            "title": "Bug in search",
            "state": "opened"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.issue_ops().expect("issue ops");

    let result = ops.get_issue("testgroup/my-project", 5).await;

    teardown_token();

    assert!(result.is_ok());

    let data = result.unwrap();

    assert_eq!(data["title"], "Bug in search");
}

#[tokio::test]
#[serial]
async fn test_gitlab_update_issue() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/projects/testgroup%2Fmy-project/issues/5"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 100,
            "iid": 5,
            "title": "Updated title",
            "state": "closed"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.issue_ops().expect("issue ops");

    let result = ops
        .update_issue(
            "testgroup/my-project",
            5,
            serde_json::json!({"title": "Updated title", "state_event": "close"}),
        )
        .await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_comment_on_issue() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/projects/testgroup%2Fmy-project/issues/5/notes"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": 999,
            "body": "Comment text"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.issue_ops().expect("issue ops");

    let result = ops
        .comment_on_issue("testgroup/my-project", 5, "Comment text")
        .await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_issue_comments() {
    let comments_json = serde_json::json!([
        {
            "id": 1,
            "body": "First comment",
            "author": { "username": "dev" }
        }
    ]);

    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/testgroup%2Fmy-project/issues/5/notes"))
        .and(query_param("per_page", "100"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(comments_json))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.issue_ops().expect("issue ops");

    let result = ops.list_issue_comments("testgroup/my-project", 5).await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_delete_release() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/projects/testgroup%2Fmy-project/releases/1"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.release_ops().expect("release ops");

    let result = ops.delete_release("testgroup/my-project", 1).await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_fetch_release_by_tag() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/testgroup%2Fmy-project/releases/v1.0.0"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "tag_name": "v1.0.0",
            "name": "Release 1.0.0",
            "description": "First release"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.release_ops().expect("release ops");

    let result = ops
        .fetch_release_by_tag("testgroup/my-project", "v1.0.0")
        .await;

    teardown_token();

    assert!(result.is_ok());

    let data = result.unwrap();

    assert_eq!(data["tag_name"], "v1.0.0");
}

#[tokio::test]
#[serial]
async fn test_gitlab_cancel_pipeline() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/projects/testgroup%2Fmy-project/jobs/101/cancel"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.pipeline_ops().expect("pipeline ops");

    let result = ops.cancel_run("testgroup/my-project", 101).await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_delete_pipeline() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/projects/testgroup%2Fmy-project/pipelines/200"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.pipeline_ops().expect("pipeline ops");

    let result = ops.delete_run("testgroup/my-project", 200).await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_test_webhook() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/projects/testgroup%2Fmy-project/hooks/42/test"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.webhook_ops().expect("webhook ops");

    let result = ops.test_webhook("testgroup/my-project", 42).await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_delete_gpg_key() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/user/gpg_keys/10"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.identity_ops().expect("identity ops");

    let result = ops.delete_gpg_key(10).await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_add_gpg_key() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/user/gpg_keys"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": 10,
            "description": "My GPG Key",
            "key_id": "ABC123",
            "created_at": "2024-01-01T00:00:00Z"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.identity_ops().expect("identity ops");

    let result = ops
        .add_gpg_key("-----BEGIN PGP PUBLIC KEY BLOCK-----\n...")
        .await;

    teardown_token();

    assert!(result.is_ok());

    let key = result.unwrap();

    assert_eq!(key.id, 10);

    assert_eq!(key.name, "My GPG Key");
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_deployments() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "id": 1,
                "ref": "main",
                "environment": "production",
                "status": "success",
                "description": "Deploy main",
                "user": { "username": "deployer" },
                "created_at": "2024-01-01T00:00:00Z"
            }
        ])))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.deploy_ops().expect("deploy ops");

    let result = ops.list_deployments("testgroup/my-project", None, 30).await;

    teardown_token();

    assert!(result.is_ok());

    let deploys = result.unwrap();

    assert_eq!(deploys.len(), 1);

    assert_eq!(deploys[0].environment, "production");
}

#[tokio::test]
#[serial]
async fn test_gitlab_create_deployment() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": 2,
            "ref": "v1.0",
            "environment": "staging",
            "status": "running",
            "description": null,
            "user": { "username": "ci-bot" },
            "created_at": "2024-02-01T00:00:00Z"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.deploy_ops().expect("deploy ops");

    let result = ops
        .create_deployment("testgroup/my-project", serde_json::json!({}))
        .await;

    teardown_token();

    assert!(result.is_ok());

    let deploy = result.unwrap();

    assert_eq!(deploy.id, 2);

    assert_eq!(deploy.r#ref, "v1.0");
}

#[tokio::test]
#[serial]
async fn test_gitlab_get_traffic_views() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "fetches": { "count": 100 },
            "views": { "count": 50 }
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.analytics_ops().expect("analytics ops");

    let result = ops.get_traffic_views("testgroup/my-project").await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_get_traffic_clones() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "fetches": { "count": 200 },
            "views": { "count": 75 }
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.analytics_ops().expect("analytics ops");

    let result = ops.get_traffic_clones("testgroup/my-project").await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_rulesets() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "deny_delete_tag": true
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.governance_ops().expect("governance ops");

    let result = ops.list_rulesets("testgroup/my-project").await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_create_ruleset() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": 2,
            "deny_delete_tag": false
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.governance_ops().expect("governance ops");

    let input = gitfleet_core::types::RulesetInput {
        name: "test-rule".to_string(),
        target: None,
        rules: None,
        enforcement: None,
        conditions: None,
    };

    let result = ops.create_ruleset("testgroup/my-project", &input).await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_delete_ruleset() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.governance_ops().expect("governance ops");

    let result = ops.delete_ruleset("testgroup/my-project", 1).await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_repo_secrets() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "key": "MY_SECRET",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-02T00:00:00Z"
            }
        ])))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.secret_ops().expect("secret ops");

    let result = ops.list_repo_secrets("testgroup", "my-project").await;

    teardown_token();

    assert!(result.is_ok());

    let secrets = result.unwrap();

    assert_eq!(secrets.total_count, 1);

    assert_eq!(secrets.secrets[0].name, "MY_SECRET");
}

#[tokio::test]
#[serial]
async fn test_gitlab_set_repo_secret() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "key": "NEW_SECRET",
            "value": "***",
            "masked": true
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.secret_ops().expect("secret ops");

    let result = ops
        .set_repo_secret("testgroup", "my-project", "NEW_SECRET", "encrypted", "key1")
        .await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_delete_repo_secret() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.secret_ops().expect("secret ops");

    let result = ops
        .delete_repo_secret("testgroup", "my-project", "MY_SECRET")
        .await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_licenses() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "key": "mit",
                "name": "MIT License",
                "spdx_id": "MIT",
                "url": "https://opensource.org/licenses/MIT"
            }
        ])))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.license_ops().expect("license ops");

    let result = ops.list_licenses().await;

    teardown_token();

    assert!(result.is_ok());

    let licenses = result.unwrap();

    assert_eq!(licenses.len(), 1);

    assert_eq!(licenses[0].key, "mit");
}

#[tokio::test]
#[serial]
async fn test_gitlab_get_license() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "key": "mit",
            "name": "MIT License",
            "spdx_id": "MIT",
            "url": "https://opensource.org/licenses/MIT",
            "description": "A permissive license",
            "implementation": "Do what you want",
            "content": "MIT License text..."
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.license_ops().expect("license ops");

    let result = ops.get_license("mit").await;

    teardown_token();

    assert!(result.is_ok());

    let license = result.unwrap();

    assert_eq!(license.key, "mit");

    assert_eq!(license.name, "MIT License");
}

#[tokio::test]
#[serial]
async fn test_gitlab_repo_license() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1,
            "name": "my-project",
            "license": { "key": "mit" }
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.license_ops().expect("license ops");

    let result = ops.repo_license("testgroup/my-project").await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_sbom() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "name": "pkg1", "version": "1.0" }
        ])))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.dependency_ops().expect("dependency ops");

    let result = ops.sbom("testgroup/my-project").await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_review_dependencies() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "diffs": [
                { "new_path": "package.json", "new_file": true, "deleted_file": false, "diff": "" }
            ]
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.dependency_ops().expect("dependency ops");

    let result = ops
        .review_dependencies("testgroup/my-project", "main", "feature")
        .await;

    teardown_token();

    assert!(result.is_ok());

    let changes = result.unwrap();

    assert_eq!(changes.len(), 1);
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_dependabot_alerts() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "id": 1, "severity": "high" }
        ])))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.advisory_ops().expect("advisory ops");

    let result = ops
        .list_dependabot_alerts("testgroup/my-project", None)
        .await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_codeql_alerts() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "id": 2, "severity": "medium" }
        ])))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.advisory_ops().expect("advisory ops");

    let result = ops.list_codeql_alerts("testgroup/my-project", None).await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_secret_scanning_alerts() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "id": 3, "severity": "critical" }
        ])))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.advisory_ops().expect("advisory ops");

    let result = ops
        .list_secret_scanning_alerts("testgroup/my-project", None)
        .await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_get_dependabot_alert() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5,
            "severity": "low"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.advisory_ops().expect("advisory ops");

    let result = ops.get_dependabot_alert("testgroup/my-project", 5).await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_attestations() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "id": 1, "name": "artifact1" }
        ])))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.attestation_ops().expect("attestation ops");

    let result = ops
        .list_attestations("testgroup/my-project", "sha256:abc")
        .await;

    teardown_token();

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_gitlab_get_attestation() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(header("PRIVATE-TOKEN", "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 10,
            "name": "artifact10"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.attestation_ops().expect("attestation ops");

    let result = ops.get_attestation("testgroup/my-project", 10).await;

    teardown_token();

    assert!(result.is_ok());
}
