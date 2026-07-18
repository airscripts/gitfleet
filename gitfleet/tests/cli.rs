use assert_cmd::Command;
use predicates::prelude::*;

fn get_help_output(args: &[&str]) -> String {
    let output = Command::cargo_bin("gitfleet")
        .unwrap()
        .args(args)
        .arg("--help")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8_lossy(&output).trim_end().to_string()
}

#[test]
fn test_version_flag() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("gitfleet"));
}

#[test]
fn test_help_flag() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Command every repository as one fleet",
        ));
}

#[test]
fn test_version_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("version")
        .assert()
        .success();
}

#[test]
fn test_alias_executes_expansion() {
    let home = tempfile::tempdir().unwrap();

    Command::cargo_bin("gitfleet")
        .unwrap()
        .env("GITFLEET_HOME", home.path())
        .args(["alias", "set", "v", "version"])
        .assert()
        .success();

    Command::cargo_bin("gitfleet")
        .unwrap()
        .env("GITFLEET_HOME", home.path())
        .arg("v")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.1.0"));
}

#[test]
fn test_alias_cannot_shadow_canonical_command() {
    let home = tempfile::tempdir().unwrap();

    Command::cargo_bin("gitfleet")
        .unwrap()
        .env("GITFLEET_HOME", home.path())
        .args(["alias", "set", "version", "repo list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "conflicts with a Gitfleet command",
        ));
}

#[test]
fn test_version_subcommand_ignores_malformed_credentials() {
    let home = tempfile::tempdir().unwrap();
    let folder = home.path().join(".config/gitfleet");
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(folder.join("credentials.toml"), "invalid = [toml").unwrap();

    Command::cargo_bin("gitfleet")
        .unwrap()
        .env("GITFLEET_HOME", home.path())
        .arg("version")
        .assert()
        .success();
}

#[test]
fn test_completion_ignores_malformed_credentials() {
    let home = tempfile::tempdir().unwrap();
    let folder = home.path().join(".config/gitfleet");
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(folder.join("credentials.toml"), "invalid = [toml").unwrap();

    Command::cargo_bin("gitfleet")
        .unwrap()
        .env("GITFLEET_HOME", home.path())
        .args(["completion", "generate", "bash"])
        .assert()
        .success();
}

#[test]
fn test_auth_logout_recovers_from_malformed_credentials() {
    let home = tempfile::tempdir().unwrap();
    let folder = home.path().join(".config/gitfleet");
    let credentials = folder.join("credentials.toml");
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(&credentials, "invalid = [toml").unwrap();

    Command::cargo_bin("gitfleet")
        .unwrap()
        .env("GITFLEET_HOME", home.path())
        .args(["auth", "logout", "--yes"])
        .assert()
        .success();

    assert!(!credentials.exists());
}

#[test]
fn test_repo_rejects_removed_noop_flags() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .args(["repo", "create", "example", "--template", "source"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument '--template'"));

    Command::cargo_bin("gitfleet")
        .unwrap()
        .args(["repo", "list", "--type", "private"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument '--type'"));

    Command::cargo_bin("gitfleet")
        .unwrap()
        .args(["repo", "fork", "create", "owner/repo", "--org", "legacy"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument '--org'"));
}

#[test]
fn test_repo_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("repo")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("repo"));
}

#[test]
fn test_change_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("change")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("change"));
}

#[test]
fn test_issue_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("issue")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("issue"));
}

#[test]
fn test_auth_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("auth")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("auth"));
}

#[test]
fn test_config_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("config")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("config"));
}

#[test]
fn test_workspace_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("workspace")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("workspace"));
}

#[test]
fn test_repo_list_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("repo")
        .arg("list")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_repo_create_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("repo")
        .arg("create")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_repo_clone_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("repo")
        .arg("clone")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--all"))
        .stdout(predicate::str::contains("--org"))
        .stdout(predicate::str::contains("--user"))
        .stdout(predicate::str::contains("--ssh"));
}

#[test]
fn test_repo_clone_rejects_repository_with_all() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .args(["repo", "clone", "owner/repo", "--all", "--org", "owner"])
        .assert()
        .failure();
}

#[test]
fn test_repo_clone_all_requires_owner_scope() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .args(["repo", "clone", "--all"])
        .assert()
        .failure();
}

#[test]
fn test_repo_clone_all_rejects_org_and_user() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .args(["repo", "clone", "--all", "--org", "org", "--user", "user"])
        .assert()
        .failure();
}

#[test]
fn test_pipeline_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("pipeline")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_release_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("release")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_label_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("label")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_inbox_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("inbox")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_access_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("access")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_advisory_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("advisory")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_alias_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("alias")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_analytics_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("analytics")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_attestation_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("attestation")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_browse_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("browse")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_code_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("code")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_comment_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("review")
        .arg("comment")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_dependency_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("deps")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_deploy_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("deploy")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_dev_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("dev")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_discussion_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("discussion")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_environment_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("environment")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_fork_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("repo")
        .arg("fork")
        .arg("--help")
        .assert()
        .success();

    Command::cargo_bin("gitfleet")
        .unwrap()
        .args(["repo", "fork", "create", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--owner"));
}

#[test]
fn test_govern_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("govern")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_identity_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("identity")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_license_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("license")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_milestone_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("planning")
        .arg("milestone")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_package_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("registry")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_policy_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("policy")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_project_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("planning")
        .arg("project")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_reaction_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("review")
        .arg("reaction")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_runner_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("runner")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_search_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("search")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_secret_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("secret")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_snippet_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("snippet")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_template_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("template")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_variable_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("variable")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_webhook_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("webhook")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_wiki_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("wiki")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_repo_delete_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("repo")
        .arg("delete")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_repo_star_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("repo")
        .arg("star")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_repo_unstar_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("repo")
        .arg("unstar")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_repo_view_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("repo")
        .arg("view")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_change_list_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("change")
        .arg("list")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_change_view_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("change")
        .arg("view")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_change_create_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("change")
        .arg("create")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_change_merge_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("change")
        .arg("create")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_issue_list_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("issue")
        .arg("list")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_issue_view_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("issue")
        .arg("view")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_issue_create_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("issue")
        .arg("create")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_label_list_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("label")
        .arg("list")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_label_create_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("label")
        .arg("create")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_label_delete_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("label")
        .arg("delete")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_pipeline_list_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("pipeline")
        .arg("list-runs")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_release_list_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("release")
        .arg("list")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_release_create_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("release")
        .arg("create")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_release_view_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("release")
        .arg("view")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_auth_login_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("auth")
        .arg("login")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_auth_status_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("auth")
        .arg("status")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--capabilities"));
}

#[test]
fn test_config_show_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("config")
        .arg("get")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_workspace_init_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("workspace")
        .arg("define")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_json_flag_accepted() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("--json")
        .arg("version")
        .assert()
        .success();
}

#[test]
fn test_debug_flag_accepted() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("--debug")
        .arg("version")
        .assert()
        .success();
}

#[test]
fn test_repo_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("repo")
        .assert()
        .failure();
}

#[test]
fn test_change_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("change")
        .assert()
        .failure();
}

#[test]
fn test_issue_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("issue")
        .assert()
        .failure();
}

#[test]
fn test_label_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("label")
        .assert()
        .failure();
}

#[test]
fn test_pipeline_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("pipeline")
        .assert()
        .failure();
}

#[test]
fn test_release_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("release")
        .assert()
        .failure();
}

#[test]
fn test_auth_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("auth")
        .assert()
        .failure();
}

#[test]
fn test_config_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("config")
        .assert()
        .failure();
}

#[test]
fn test_webhook_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("webhook")
        .assert()
        .failure();
}

#[test]
fn test_inbox_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("inbox")
        .assert()
        .failure();
}

#[test]
fn test_search_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("search")
        .assert()
        .failure();
}

#[test]
fn test_secret_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("secret")
        .assert()
        .failure();
}

#[test]
fn test_variable_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("variable")
        .assert()
        .failure();
}

#[test]
fn test_runner_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("runner")
        .assert()
        .failure();
}

#[test]
fn test_environment_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("environment")
        .assert()
        .failure();
}

#[test]
fn test_deploy_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("deploy")
        .assert()
        .failure();
}

#[test]
fn test_access_org_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("access")
        .arg("org")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_access_team_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("access")
        .arg("team")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_security_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("security")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("List security advisories"));
}

#[test]
fn test_security_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("security")
        .assert()
        .failure();
}

#[test]
fn test_completion_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("completion")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Generate shell completions"));
}

#[test]
fn test_milestone_list_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("planning")
        .arg("milestone")
        .arg("list")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_milestone_create_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("planning")
        .arg("milestone")
        .arg("create")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_milestone_view_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("planning")
        .arg("milestone")
        .arg("view")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_milestone_delete_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("planning")
        .arg("milestone")
        .arg("delete")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_project_list_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("planning")
        .arg("project")
        .arg("list")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_project_create_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("planning")
        .arg("project")
        .arg("create")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_project_view_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("planning")
        .arg("project")
        .arg("view")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_project_delete_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("planning")
        .arg("project")
        .arg("delete")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_reaction_list_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("review")
        .arg("reaction")
        .arg("list")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_reaction_create_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("review")
        .arg("reaction")
        .arg("create")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_reaction_delete_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("review")
        .arg("reaction")
        .arg("delete")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_comment_list_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("review")
        .arg("comment")
        .arg("list")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_snippet_list_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("snippet")
        .arg("list")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_snippet_view_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("snippet")
        .arg("view")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_snippet_create_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("snippet")
        .arg("create")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_snippet_delete_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("snippet")
        .arg("delete")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_package_list_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("registry")
        .arg("list")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_package_view_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("registry")
        .arg("view")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_dev_list_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("dev")
        .arg("list")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_dev_create_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("dev")
        .arg("create")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_dev_delete_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("dev")
        .arg("delete")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_wiki_view_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("wiki")
        .arg("view")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_wiki_create_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("wiki")
        .arg("create")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_wiki_edit_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("wiki")
        .arg("edit")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_wiki_delete_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("wiki")
        .arg("delete")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_discussion_list_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("discussion")
        .arg("list")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_discussion_view_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("discussion")
        .arg("view")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_discussion_create_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("discussion")
        .arg("create")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_dependency_review_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("deps")
        .arg("review")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_advisory_list_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("advisory")
        .arg("list")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_advisory_view_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("advisory")
        .arg("view")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_access_org_list_members_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("access")
        .arg("org")
        .arg("list-members")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_access_org_remove_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("access")
        .arg("org")
        .arg("remove")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_access_team_create_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("access")
        .arg("team")
        .arg("create")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_access_team_list_members_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("access")
        .arg("team")
        .arg("list-members")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_identity_gpg_add_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("identity")
        .arg("gpg-key")
        .arg("add")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_identity_gpg_delete_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("identity")
        .arg("gpg-key")
        .arg("delete")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_repo_archive_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("repo")
        .arg("archive")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_repo_unarchive_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("repo")
        .arg("unarchive")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_repo_rename_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("repo")
        .arg("rename")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_repo_edit_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("repo")
        .arg("edit")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_auth_setup_git_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("auth")
        .arg("setup-git")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_inbox_mark_read_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("inbox")
        .arg("mark-read")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_milestone_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("planning")
        .arg("milestone")
        .assert()
        .failure();
}

#[test]
fn test_project_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("planning")
        .arg("project")
        .assert()
        .failure();
}

#[test]
fn test_reaction_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("review")
        .arg("reaction")
        .assert()
        .failure();
}

#[test]
fn test_comment_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("review")
        .arg("comment")
        .assert()
        .failure();
}

#[test]
fn test_snippet_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("snippet")
        .assert()
        .failure();
}

#[test]
fn test_package_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("registry")
        .assert()
        .failure();
}

#[test]
fn test_dev_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("dev")
        .assert()
        .failure();
}

#[test]
fn test_wiki_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("wiki")
        .assert()
        .failure();
}

#[test]
fn test_discussion_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("discussion")
        .assert()
        .failure();
}

#[test]
fn test_dependency_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("deps")
        .assert()
        .failure();
}

#[test]
fn test_advisory_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("advisory")
        .assert()
        .failure();
}

#[test]
fn test_attestation_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("attestation")
        .assert()
        .failure();
}

#[test]
fn test_identity_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("identity")
        .assert()
        .failure();
}

#[test]
fn test_analytics_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("analytics")
        .assert()
        .failure();
}

#[test]
fn test_alias_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("alias")
        .assert()
        .failure();
}

#[test]
fn test_browse_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("browse")
        .assert()
        .failure();
}

#[test]
fn test_code_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("code")
        .assert()
        .failure();
}

#[test]
fn test_completion_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("completion")
        .assert()
        .failure();
}

#[test]
fn test_fork_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("repo")
        .arg("fork")
        .assert()
        .failure();
}

#[test]
fn test_govern_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("govern")
        .assert()
        .failure();
}

#[test]
fn test_license_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("license")
        .assert()
        .failure();
}

#[test]
fn test_policy_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("policy")
        .assert()
        .failure();
}

#[test]
fn test_template_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("template")
        .assert()
        .failure();
}

#[test]
fn test_workspace_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("workspace")
        .assert()
        .failure();
}

#[test]
fn test_dry_run_flag_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--dry-run"));
}

#[test]
fn test_dry_run_flag_accepted() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("--dry-run")
        .arg("version")
        .assert()
        .success();
}

#[test]
fn test_git_credential_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("git-credential")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_yes_flag_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--yes"));
}

#[test]
fn test_api_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("api")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("raw"));
}

#[test]
fn test_api_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("api")
        .assert()
        .failure();
}

#[test]
fn test_api_get_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("api")
        .arg("get")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_api_post_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("api")
        .arg("post")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_api_delete_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("api")
        .arg("delete")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_site_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("site")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Pages"));
}

#[test]
fn test_site_requires_subcommand() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("site")
        .assert()
        .failure();
}

#[test]
fn test_site_get_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("site")
        .arg("get")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_site_create_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("site")
        .arg("create")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_site_delete_help() {
    Command::cargo_bin("gitfleet")
        .unwrap()
        .arg("site")
        .arg("delete")
        .arg("--help")
        .assert()
        .success();
}

// ===== Insta snapshot tests for --help output =====

#[test]
fn snapshot_help_main() {
    let output = get_help_output(&[]);

    insta::assert_snapshot!("help_main", output);
}

#[test]
fn snapshot_help_repo() {
    let output = get_help_output(&["repo"]);

    insta::assert_snapshot!("help_repo", output);
}

#[test]
fn snapshot_help_api() {
    let output = get_help_output(&["api"]);

    insta::assert_snapshot!("help_api", output);
}

#[test]
fn snapshot_help_site() {
    let output = get_help_output(&["site"]);

    insta::assert_snapshot!("help_site", output);
}

#[test]
fn snapshot_help_config() {
    let output = get_help_output(&["config"]);

    insta::assert_snapshot!("help_config", output);
}
