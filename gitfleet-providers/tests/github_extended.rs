use gitfleet_core::provider::GitProvider;
use gitfleet_core::types::MilestoneState;
use gitfleet_providers::GitHubProvider;
use serial_test::serial;
use wiremock::matchers::{body_string_contains, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn setup_token() {
    std::env::set_var("GITFLEET_GITHUB_TOKEN", "testtoken");
}

fn teardown_token() {
    std::env::remove_var("GITFLEET_GITHUB_TOKEN");
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

fn tag_protection_json() -> serde_json::Value {
    serde_json::json!([
        {
            "id": 42,
            "pattern": "v*",
            "created_at": "2024-01-01T00:00:00Z"
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
async fn test_fork_repo() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/repos/testorg/repo/forks"))
        .and(header("authorization", "Bearer testtoken"))
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

    let result = ops.fork_repo("testorg/repo").await.unwrap();

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
                    "organization": {
                        "projectsV2": {
                            "nodes": [
                                {"id": "P_1", "number": 1, "title": "Project 1", "shortDescription": null,
                                 "closed": false, "url": "https://github.com/orgs/testorg/projects/1",
                                 "updatedAt": "2024-01-01T00:00:00Z"}
                            ]
                        }
                    },
                    "user": null
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
        .update_release("testorg/repo", 1, serde_json::json!({"body": "Updated"}))
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

    ops.delete_release("testorg/repo", 1).await.unwrap();

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
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/repo/tags-protection"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(tag_protection_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.policy_ops().unwrap();

    let result = ops.list_tag_protection("testorg/repo").await.unwrap();

    teardown_token();

    assert_eq!(result.len(), 1);

    assert_eq!(result[0].pattern, "v*");
}

#[tokio::test]
#[serial]
async fn test_create_tag_protection() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/repos/testorg/repo/tags-protection"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": 43, "pattern": "v*", "created_at": "2024-01-01T00:00:00Z"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.policy_ops().unwrap();

    let result = ops
        .create_tag_protection("testorg/repo", "v*")
        .await
        .unwrap();

    teardown_token();

    assert_eq!(result.pattern, "v*");

    assert_eq!(result.id, 43);
}

#[tokio::test]
#[serial]
async fn test_delete_tag_protection() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/repos/testorg/repo/tags-protection/42"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.policy_ops().unwrap();

    ops.delete_tag_protection("testorg/repo", 42).await.unwrap();

    teardown_token();
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
        .create_pages("testorg/repo", "main", None)
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

    Mock::given(method("GET"))
        .and(path("/orgs/testorg/packages"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(package_json()))
        .mount(&server)
        .await;

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
        .and(path("/repos/testorg/repo/dependency-graph/compare/main...feature"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "changes": [
                    {"change_type": "added", "package": "serde", "ecosystem": "cargo", "version": "1.0", "severity": "low", "vulnerabilities": 0}
                ]
            })),
        )
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

#[tokio::test]
#[serial]
async fn test_get_attestation() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/repo/attestations/1"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"id": 1, "payload": "..."})),
        )
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.attestation_ops().unwrap();

    let result = ops.get_attestation("testorg/repo", 1).await.unwrap();

    teardown_token();

    assert_eq!(result["id"], 1);
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

    Mock::given(method("GET"))
        .and(path("/repos/testorg/repo/issue/templates"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {"name": "Bug Report", "filename": "bug.md", "path": ".github/ISSUE_TEMPLATE/bug.md",
                 "body": "## Bug", "about": "Report a bug", "title": "Bug: ", "labels": ["bug"], "assignees": []}
            ])),
        )
        .mount(&server)
        .await;

    setup_token();

    let provider = GitHubProvider::with_base_url(&server.uri());
    let ops = provider.template_ops().unwrap();

    let templates = ops.list_issue_templates("testorg/repo").await.unwrap();

    teardown_token();

    assert_eq!(templates.len(), 1);

    assert_eq!(templates[0].name, "Bug Report");
}

#[tokio::test]
#[serial]
async fn test_list_issue_templates_empty() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/repo/issue/templates"))
        .and(header("authorization", "Bearer testtoken"))
        .respond_with(ResponseTemplate::new(404))
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
