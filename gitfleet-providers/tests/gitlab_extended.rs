use gitfleet_core::provider::GitProvider;
use gitfleet_providers::GitLabProvider;
use serial_test::serial;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn setup_token() {
    std::env::set_var("GITFLEET_GITLAB_TOKEN", "testtoken");
    std::env::set_var("GITFLEET_PROFILE", "__gitfleet_integration_test__");
}

fn teardown_token() {
    std::env::remove_var("GITFLEET_GITLAB_TOKEN");
    std::env::remove_var("GITFLEET_PROFILE");
}

const TOKEN_HEADER: &str = "PRIVATE-TOKEN";

fn discussion_json() -> serde_json::Value {
    serde_json::json!({
        "id": 1, "iid": 1, "title": "Discussion 1",
        "description": "Body text", "state": "opened",
        "web_url": "https://gitlab.com/testgroup/my-project/-/issues/1",
        "author": {"username": "dev1"}, "user_notes_count": 3,
        "created_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-02T00:00:00Z"
    })
}

// ===== RawApiOps =====

#[tokio::test]
#[serial]
async fn test_gitlab_raw_get() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/some/endpoint"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"result": "ok"})))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.raw_api_ops().unwrap();

    let result = ops.raw_get("/some/endpoint").await.unwrap();

    teardown_token();

    assert_eq!(result["result"], "ok");
}

#[tokio::test]
#[serial]
async fn test_gitlab_raw_post() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/some/endpoint"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(serde_json::json!({"created": true})),
        )
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
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
async fn test_gitlab_raw_delete() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/some/endpoint"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"deleted": true})),
        )
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.raw_api_ops().unwrap();

    let result = ops.raw_delete("/some/endpoint").await.unwrap();

    teardown_token();

    assert_eq!(result["deleted"], true);
}

#[tokio::test]
#[serial]
async fn test_gitlab_raw_delete_no_content() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/some/endpoint"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.raw_api_ops().unwrap();

    let result = ops.raw_delete("/some/endpoint").await.unwrap();

    teardown_token();

    assert_eq!(result["status"], "deleted");
}

// ===== BrowseOps =====

#[tokio::test]
#[serial]
async fn test_gitlab_list_contents() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/testgroup%2Fmy-project/repository/tree"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": "abc", "name": "README.md", "type": "blob", "path": "README.md"}
        ])))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.browse_ops().unwrap();

    let contents = ops
        .list_contents("testgroup/my-project", None)
        .await
        .unwrap();

    teardown_token();

    assert!(contents.is_array());
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_contents_with_path() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/testgroup%2Fmy-project/repository/tree"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": "def", "name": "main.rs", "type": "blob", "path": "src/main.rs"}
        ])))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.browse_ops().unwrap();

    let contents = ops
        .list_contents("testgroup/my-project", Some("src"))
        .await
        .unwrap();

    teardown_token();

    assert!(contents.is_array());
}

#[tokio::test]
#[serial]
async fn test_gitlab_file_contents() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(
            "/projects/testgroup%2Fmy-project/repository/files/src%2Fmain.rs",
        ))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "file_name": "main.rs", "file_path": "src/main.rs",
            "content": "fn main() {}", "encoding": "text", "ref": "main",
            "blob_id": "abc", "commit_id": "def", "last_commit_id": "ghi"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.browse_ops().unwrap();

    let contents = ops
        .file_contents("testgroup/my-project", "src/main.rs", Some("main"))
        .await
        .unwrap();

    teardown_token();

    assert_eq!(contents["file_name"], "main.rs");
}

// ===== TemplateOps =====

#[tokio::test]
#[serial]
async fn test_gitlab_list_issue_templates() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/testgroup%2Fmy-project/templates/Issues"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {"name": "Bug Report", "filename": "bug.md", "path": ".gitlab/issue_templates/bug.md", "content": "## Bug"}
            ])),
        )
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.template_ops().unwrap();

    let templates = ops
        .list_issue_templates("testgroup/my-project")
        .await
        .unwrap();

    teardown_token();

    assert_eq!(templates.len(), 1);

    assert_eq!(templates[0].name, "Bug Report");
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_issue_templates_empty() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/testgroup%2Fmy-project/templates/Issues"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.template_ops().unwrap();

    let templates = ops
        .list_issue_templates("testgroup/my-project")
        .await
        .unwrap();

    teardown_token();

    assert!(templates.is_empty());
}

// ===== DiscussionOps (get/create) =====

#[tokio::test]
#[serial]
async fn test_gitlab_get_discussion() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/testgroup%2Fmy-project/issues/1"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(discussion_json()))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let Some(ops) = provider.discussion_ops() else {
        return;
    };

    let discussion = ops
        .get_discussion("testgroup", "my-project", 1)
        .await
        .unwrap();

    teardown_token();

    assert_eq!(discussion.number, 1);

    assert_eq!(discussion.title, "Discussion 1");
}

#[tokio::test]
#[serial]
async fn test_gitlab_create_discussion() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/projects/testgroup%2Fmy-project/issues"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": 2, "iid": 2, "title": "New Discussion",
            "description": "Body here", "state": "opened",
            "web_url": "https://gitlab.com/testgroup/my-project/-/issues/2",
            "author": {"username": "dev1"}, "user_notes_count": 0,
            "created_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let Some(ops) = provider.discussion_ops() else {
        return;
    };

    let discussion = ops
        .create_discussion(
            "testgroup",
            "my-project",
            "New Discussion",
            "Body here",
            None,
        )
        .await
        .unwrap();

    teardown_token();

    assert_eq!(discussion.number, 2);

    assert_eq!(discussion.title, "New Discussion");
}

// ===== PipelineOps (get_run calls get_job) =====

#[tokio::test]
#[serial]
async fn test_gitlab_get_run() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/testgroup%2Fmy-project/jobs/100"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 100, "name": "build", "status": "success", "stage": "test", "ref": "main"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.pipeline_ops().unwrap();

    let result = ops.get_run("testgroup/my-project", 100).await.unwrap();

    teardown_token();

    assert_eq!(result["id"], 100);
}

// ===== ReviewOps (Award Emojis) =====

#[tokio::test]
#[serial]
async fn test_gitlab_list_reactions() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/org%2Frepo/issues/1/award_emoji"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "name": "thumbsup", "user": {"username": "alice"}, "created_at": "2024-01-01T00:00:00Z"}
        ])))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.review_ops().unwrap();

    let result = ops.list_reactions_for_issue("org/repo", 1).await.unwrap();

    teardown_token();

    assert_eq!(result.len(), 1);

    assert_eq!(result[0].id, 1);
    assert_eq!(result[0].content, "thumbsup");
}

#[tokio::test]
#[serial]
async fn test_gitlab_create_reaction() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/projects/org%2Frepo/issues/1/award_emoji"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5, "name": "thumbsup", "user": {"username": "bob"}, "created_at": "2024-01-01T00:00:00Z"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.review_ops().unwrap();

    let result = ops
        .create_reaction_for_issue("org/repo", 1, "thumbsup")
        .await
        .unwrap();

    teardown_token();

    assert_eq!(result.id, 5);

    assert_eq!(result.content, "thumbsup");
}

#[tokio::test]
#[serial]
async fn test_gitlab_delete_reaction() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/projects/org%2Frepo/issues/1/award_emoji/5"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.review_ops().unwrap();

    ops.delete_reaction_for_issue("org/repo", 1, 5)
        .await
        .unwrap();

    teardown_token();
}

// ===== SnippetOps =====

#[tokio::test]
#[serial]
async fn test_gitlab_list_snippets() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/snippets"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "title": "Test", "visibility": "internal", "web_url": "https://gitlab.com/snippets/1", "created_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "file_name": "main.py"}
        ])))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.snippet_ops().unwrap();

    let result = ops.list_snippets("owner").await.unwrap();

    teardown_token();

    assert_eq!(result.len(), 1);

    assert_eq!(result[0].id, "1");
    assert!(result[0].public);
}

#[tokio::test]
#[serial]
async fn test_gitlab_get_snippet() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/snippets/42"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"id": 42, "title": "Test"})),
        )
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.snippet_ops().unwrap();

    let result = ops.get_snippet("42").await.unwrap();

    teardown_token();

    assert_eq!(result["id"], 42);
}

#[tokio::test]
#[serial]
async fn test_gitlab_create_snippet() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/snippets"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 10, "title": "My Snippet", "visibility": "internal", "web_url": "https://gitlab.com/snippets/10", "created_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "file_name": "snippet.txt"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.snippet_ops().unwrap();

    let result = ops
        .create_snippet("My Snippet", true, serde_json::json!({"content": "hello"}))
        .await
        .unwrap();

    teardown_token();

    assert_eq!(result.id, "10");
}

#[tokio::test]
#[serial]
async fn test_gitlab_delete_snippet() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/snippets/42"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.snippet_ops().unwrap();

    ops.delete_snippet("42").await.unwrap();

    teardown_token();
}

// ===== PolicyOps (Protected Branches + Tags) =====

#[tokio::test]
#[serial]
async fn test_gitlab_get_branch_protection() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/org%2Frepo/protected_branches/main"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            serde_json::json!({"name": "main", "push_access_levels": [{"access_level": 40}]}),
        ))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.policy_ops().unwrap();

    let result = ops.get_branch_protection("org/repo", "main").await.unwrap();

    teardown_token();

    assert_eq!(result["name"], "main");
}

#[tokio::test]
#[serial]
async fn test_gitlab_protect_branch() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/projects/org%2Frepo/protected_branches"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"name": "main"})))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.policy_ops().unwrap();

    let result = ops
        .protect_branch("org/repo", "main", serde_json::json!({}))
        .await
        .unwrap();

    teardown_token();

    assert_eq!(result["name"], "main");
}

#[tokio::test]
#[serial]
async fn test_gitlab_unprotect_branch() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/projects/org%2Frepo/protected_branches/main"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.policy_ops().unwrap();

    ops.unprotect_branch("org/repo", "main").await.unwrap();

    teardown_token();
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_tag_protection() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/org%2Frepo/protected_tags"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "name": "v*", "created_at": "2024-01-01T00:00:00Z"}
        ])))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.policy_ops().unwrap();

    let result = ops.list_tag_protection("org/repo").await.unwrap();

    teardown_token();

    assert_eq!(result.len(), 1);

    assert_eq!(result[0].id, 1);
    assert_eq!(result[0].pattern, "v*");
}

#[tokio::test]
#[serial]
async fn test_gitlab_create_tag_protection() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/projects/org%2Frepo/protected_tags"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            serde_json::json!({"id": 2, "name": "v*", "created_at": "2024-01-01T00:00:00Z"}),
        ))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.policy_ops().unwrap();

    let result = ops.create_tag_protection("org/repo", "v*").await.unwrap();

    teardown_token();

    assert_eq!(result.id, 2);

    assert_eq!(result.pattern, "v*");
}

#[tokio::test]
#[serial]
async fn test_gitlab_delete_tag_protection() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/projects/org%2Frepo/protected_tags/1"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.policy_ops().unwrap();

    ops.delete_tag_protection("org/repo", 1).await.unwrap();

    teardown_token();
}

// ===== SiteOps (Pages) =====

#[tokio::test]
#[serial]
async fn test_gitlab_get_pages() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/org%2Frepo/pages"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"url": "https://org.gitlab.io/repo"})),
        )
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.site_ops().unwrap();

    let result = ops.get_pages("org/repo").await.unwrap();

    teardown_token();

    assert_eq!(result["url"], "https://org.gitlab.io/repo");
}

#[tokio::test]
#[serial]
async fn test_gitlab_create_pages() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/projects/org%2Frepo/pages"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"url": "https://org.gitlab.io/repo"})),
        )
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.site_ops().unwrap();

    let result = ops.create_pages("org/repo", "main", None).await.unwrap();

    teardown_token();

    assert_eq!(result["url"], "https://org.gitlab.io/repo");
}

#[tokio::test]
#[serial]
async fn test_gitlab_remove_pages() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/projects/org%2Frepo/pages"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.site_ops().unwrap();

    ops.remove_pages("org/repo").await.unwrap();

    teardown_token();
}

// ===== RegistryOps (Package Registry) =====

#[tokio::test]
#[serial]
async fn test_gitlab_list_packages() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/org%2Frepo/packages"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "name": "my-pkg", "package_type": "npm", "visibility": "public", "created_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z"}
        ])))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.registry_ops().unwrap();

    let result = ops.list_packages("org/repo", None, 100).await.unwrap();

    teardown_token();

    assert_eq!(result.len(), 1);

    assert_eq!(result[0].name, "my-pkg");
    assert_eq!(result[0].package_type, "npm");
}

#[tokio::test]
#[serial]
async fn test_gitlab_get_package() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/org%2Frepo/packages"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "name": "my-pkg", "package_type": "npm"}
        ])))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.registry_ops().unwrap();

    let result = ops.get_package("org/repo", "npm", "my-pkg").await.unwrap();

    teardown_token();

    assert_eq!(result[0]["name"], "my-pkg");
}

// ===== PlanningOps (Milestones) =====

#[tokio::test]
#[serial]
async fn test_gitlab_list_milestones() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/org%2Frepo/milestones"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "title": "v1.0", "state": "active", "web_url": "https://gitlab.com/org/repo/-/milestones/1", "open_issues": 2, "closed_issues": 3}
        ])))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.planning_ops().unwrap();

    let result = ops.list_milestones("org/repo", None, 100).await.unwrap();

    teardown_token();

    assert_eq!(result.len(), 1);

    assert_eq!(result[0].title, "v1.0");
}

#[tokio::test]
#[serial]
async fn test_gitlab_create_milestone() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/projects/org%2Frepo/milestones"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5, "title": "v2.0", "state": "active", "web_url": "https://gitlab.com/org/repo/-/milestones/5"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.planning_ops().unwrap();

    let result = ops
        .create_milestone("org/repo", "v2.0", Some("Release 2.0"))
        .await
        .unwrap();

    teardown_token();

    assert_eq!(result.title, "v2.0");
}

#[tokio::test]
#[serial]
async fn test_gitlab_get_milestone() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/org%2Frepo/milestones/5"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5, "title": "v2.0", "state": "active", "web_url": "https://gitlab.com/org/repo/-/milestones/5"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.planning_ops().unwrap();

    let result = ops.get_milestone("org/repo", 5).await.unwrap();

    teardown_token();

    assert_eq!(result.title, "v2.0");
}

#[tokio::test]
#[serial]
async fn test_gitlab_update_milestone() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/projects/org%2Frepo/milestones/5"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5, "title": "v2.0-updated", "state": "closed", "web_url": "https://gitlab.com/org/repo/-/milestones/5"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.planning_ops().unwrap();

    let result = ops
        .update_milestone("org/repo", 5, serde_json::json!({"state": "closed"}))
        .await
        .unwrap();

    teardown_token();

    assert_eq!(result.title, "v2.0-updated");
}

#[tokio::test]
#[serial]
async fn test_gitlab_delete_milestone() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/projects/org%2Frepo/milestones/5"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.planning_ops().unwrap();

    ops.delete_milestone("org/repo", 5).await.unwrap();

    teardown_token();
}

#[tokio::test]
#[serial]
async fn test_gitlab_list_projects_unsupported() {
    setup_token();

    let provider = GitLabProvider::new();
    let ops = provider.planning_ops().unwrap();

    let result = ops.list_projects("owner", 100).await;

    teardown_token();

    assert!(result.is_err());
}

#[tokio::test]
#[serial]
async fn test_gitlab_create_project_unsupported() {
    setup_token();

    let provider = GitLabProvider::new();
    let ops = provider.planning_ops().unwrap();

    let result = ops.create_project("owner", "title", None).await;

    teardown_token();

    assert!(result.is_err());
}

// ===== Insta snapshot tests for normalized wire payloads =====

#[tokio::test]
#[serial]
async fn snapshot_gitlab_milestone_normalization() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/org%2Frepo/milestones/5"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5, "title": "v2.0", "state": "active",
            "web_url": "https://gitlab.com/org/repo/-/milestones/5",
            "open_issues": 2, "closed_issues": 8, "due_date": "2024-12-31"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.planning_ops().unwrap();

    let milestone = ops.get_milestone("org/repo", 5).await.unwrap();

    teardown_token();

    insta::assert_json_snapshot!("gitlab_milestone_normalized", milestone);
}

#[tokio::test]
#[serial]
async fn snapshot_gitlab_snippet_normalization() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/snippets/42"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 42, "title": "My Snippet", "visibility": "internal",
            "web_url": "https://gitlab.com/snippets/42",
            "raw_url": "https://gitlab.com/snippets/42/raw",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-02T00:00:00Z",
            "author": {"username": "alice"}, "file_name": "main.py"
        })))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.snippet_ops().unwrap();

    let snippet = ops.get_snippet("42").await.unwrap();

    teardown_token();

    insta::assert_json_snapshot!("gitlab_snippet_raw", snippet);
}

#[tokio::test]
#[serial]
async fn snapshot_gitlab_tag_protection_normalization() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects/org%2Frepo/protected_tags"))
        .and(header(TOKEN_HEADER, "testtoken"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "name": "v*", "created_at": "2024-01-01T00:00:00Z"},
            {"id": 2, "name": "release-*", "created_at": "2024-02-01T00:00:00Z"}
        ])))
        .mount(&server)
        .await;

    setup_token();

    let provider = GitLabProvider::with_base_url(&server.uri());
    let ops = provider.policy_ops().unwrap();

    let tags = ops.list_tag_protection("org/repo").await.unwrap();

    teardown_token();

    insta::assert_json_snapshot!("gitlab_tag_protection_normalized", tags);
}
