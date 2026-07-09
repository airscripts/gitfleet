use std::collections::HashMap;

use gitfleet_core::types::{CredentialsFile, Profile};

fn setup_tmp_home() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();

    let gitfleet_dir = dir.path().join(".config").join("gitfleet");
    std::fs::create_dir_all(&gitfleet_dir).unwrap();

    std::env::set_var("HOME", dir.path().to_string_lossy().to_string());
    std::env::remove_var("GITFLEET_GITHUB_TOKEN");
    std::env::remove_var("GITFLEET_PROFILE");
    dir
}

fn teardown_tmp_home(_dir: tempfile::TempDir) {
    std::env::remove_var("GITFLEET_GITHUB_TOKEN");
    std::env::remove_var("GITFLEET_PROFILE");
}

#[test]
#[serial_test::serial]
fn test_config_write_and_read_round_trip() {
    let dir = setup_tmp_home();

    let mut profiles = HashMap::new();
    profiles.insert(
        "integration-profile".into(),
        Profile {
            token: Some("ghp_inttest123".into()),
            host: Some("github.com".into()),
            provider: Some("github".into()),
            extra: Default::default(),
        },
    );

    let creds = CredentialsFile {
        active_profile: "integration-profile".into(),
        profiles,
        aliases: std::collections::HashMap::new(),
    };

    gitfleet_core::config::write_credentials(&creds).unwrap();

    let read_creds = gitfleet_core::config::read_credentials().unwrap();

    assert_eq!(read_creds.active_profile, "integration-profile");

    assert!(read_creds.profiles.contains_key("integration-profile"));

    let profile = read_creds.profiles.get("integration-profile").unwrap();

    assert_eq!(profile.token.as_deref(), Some("ghp_inttest123"));

    assert_eq!(profile.host.as_deref(), Some("github.com"));
    assert_eq!(profile.provider.as_deref(), Some("github"));

    teardown_tmp_home(dir);
}

#[test]
#[serial_test::serial]
fn test_config_add_remove_profile() {
    let dir = setup_tmp_home();

    let profile = Profile {
        token: Some("ghp_test123".into()),
        host: Some("gitlab.com".into()),
        provider: Some("gitlab".into()),
        extra: Default::default(),
    };

    gitfleet_core::config::add_profile("gitlab-profile", profile).unwrap();

    let found = gitfleet_core::config::get_profile("gitlab-profile").unwrap();

    assert!(found.is_some());

    assert_eq!(found.unwrap().host.as_deref(), Some("gitlab.com"));

    gitfleet_core::config::remove_profile("gitlab-profile").unwrap();

    let found = gitfleet_core::config::get_profile("gitlab-profile").unwrap();

    assert!(found.is_none());

    teardown_tmp_home(dir);
}

#[test]
#[serial_test::serial]
fn test_config_remove_nonexistent_profile_errors() {
    let dir = setup_tmp_home();

    let result = gitfleet_core::config::remove_profile("no-such-profile");

    assert!(result.is_err());

    teardown_tmp_home(dir);
}

#[test]
#[serial_test::serial]
fn test_config_set_active_profile() {
    let dir = setup_tmp_home();

    let profile = Profile {
        token: Some("ghp_active_test".into()),
        host: None,
        provider: None,
        extra: Default::default(),
    };

    gitfleet_core::config::add_profile("active-test", profile).unwrap();

    gitfleet_core::config::set_active_profile("active-test").unwrap();

    let creds = gitfleet_core::config::read_credentials().unwrap();

    assert_eq!(creds.active_profile, "active-test");

    teardown_tmp_home(dir);
}

#[test]
#[serial_test::serial]
fn test_config_set_active_nonexistent_errors() {
    let dir = setup_tmp_home();

    let result = gitfleet_core::config::set_active_profile("nonexistent");

    assert!(result.is_err());

    teardown_tmp_home(dir);
}

#[test]
#[serial_test::serial]
fn test_config_clear_credentials() {
    let dir = setup_tmp_home();

    let profile = Profile {
        token: Some("ghp_clear_test".into()),
        host: None,
        provider: None,
        extra: Default::default(),
    };

    gitfleet_core::config::add_profile("clear-test", profile).unwrap();

    gitfleet_core::config::clear_credentials().unwrap();

    let creds = gitfleet_core::config::read_credentials().unwrap();

    assert!(creds.profiles.is_empty());

    teardown_tmp_home(dir);
}

#[test]
#[serial_test::serial]
fn test_config_get_token_from_env() {
    let dir = setup_tmp_home();

    std::env::set_var("GITFLEET_GITHUB_TOKEN", "env-token-xyz");

    let token = gitfleet_core::config::get_token_optional();

    assert_eq!(token, Some("env-token-xyz".to_string()));

    std::env::remove_var("GITFLEET_GITHUB_TOKEN");
    teardown_tmp_home(dir);
}

#[test]
#[serial_test::serial]
fn test_config_get_token_empty_env_falls_through() {
    let dir = setup_tmp_home();

    std::env::set_var("GITFLEET_GITHUB_TOKEN", "");

    let token = gitfleet_core::config::get_token_optional();

    assert!(token.is_none() || token != Some(String::new()));

    std::env::remove_var("GITFLEET_GITHUB_TOKEN");
    teardown_tmp_home(dir);
}

#[test]
#[serial_test::serial]
fn test_config_get_token_errors_when_none() {
    let dir = setup_tmp_home();

    std::env::remove_var("GITFLEET_GITHUB_TOKEN");

    let result = gitfleet_core::config::get_token();

    assert!(result.is_err());

    teardown_tmp_home(dir);
}

#[test]
#[serial_test::serial]
fn test_config_write_read_key() {
    let dir = setup_tmp_home();

    let profile = Profile {
        token: Some("ghp_keytest".into()),
        host: None,
        provider: None,
        extra: Default::default(),
    };

    gitfleet_core::config::add_profile("key-test", profile).unwrap();

    gitfleet_core::config::set_active_profile("key-test").unwrap();
    gitfleet_core::config::write("host", "gitlab.com").unwrap();

    let found = gitfleet_core::config::get_profile("key-test")
        .unwrap()
        .unwrap();

    assert_eq!(found.host.as_deref(), Some("gitlab.com"));

    teardown_tmp_home(dir);
}

#[test]
#[serial_test::serial]
fn test_config_unset_key() {
    let dir = setup_tmp_home();

    let profile = Profile {
        token: Some("ghp_unsettest".into()),
        host: Some("github.com".into()),
        provider: None,
        extra: Default::default(),
    };

    gitfleet_core::config::add_profile("unset-test", profile).unwrap();

    gitfleet_core::config::set_active_profile("unset-test").unwrap();
    gitfleet_core::config::unset("host").unwrap();

    let found = gitfleet_core::config::get_profile("unset-test")
        .unwrap()
        .unwrap();

    assert!(found.host.is_none());

    teardown_tmp_home(dir);
}

#[test]
#[serial_test::serial]
fn test_config_list_profiles() {
    let dir = setup_tmp_home();

    let profile_a = Profile {
        token: Some("ghp_a".into()),
        host: None,
        provider: None,
        extra: Default::default(),
    };

    let profile_b = Profile {
        token: None,
        host: Some("gitlab.com".into()),
        provider: Some("gitlab".into()),
        extra: Default::default(),
    };

    gitfleet_core::config::add_profile("alpha", profile_a).unwrap();

    gitfleet_core::config::add_profile("beta", profile_b).unwrap();

    let entries = gitfleet_core::config::list_profiles().unwrap();

    assert!(entries.iter().any(|e| e.name == "alpha"));

    assert!(entries.iter().any(|e| e.name == "beta"));

    let alpha = entries.iter().find(|e| e.name == "alpha").unwrap();

    assert!(alpha.has_token);

    let beta = entries.iter().find(|e| e.name == "beta").unwrap();

    assert!(!beta.has_token);

    teardown_tmp_home(dir);
}

#[test]
#[serial_test::serial]
fn test_config_get_host_default() {
    let dir = setup_tmp_home();

    let host = gitfleet_core::config::get_host();

    assert_eq!(host, "github.com");

    teardown_tmp_home(dir);
}

#[test]
fn test_config_toml_round_trip_via_file() {
    let dir = tempfile::tempdir().unwrap();

    let path = dir.path().join("credentials.toml");

    let mut profiles = HashMap::new();
    profiles.insert(
        "round-trip".into(),
        Profile {
            token: Some("ghp_roundtrip".into()),
            host: Some("github.com".into()),
            provider: Some("github".into()),
            extra: Default::default(),
        },
    );

    let creds = CredentialsFile {
        active_profile: "round-trip".into(),
        profiles,
        aliases: std::collections::HashMap::new(),
    };

    let content = toml::to_string_pretty(&creds).unwrap();
    std::fs::write(&path, &content).unwrap();

    let read_back: CredentialsFile =
        toml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();

    assert_eq!(read_back.active_profile, "round-trip");

    assert!(read_back.profiles.contains_key("round-trip"));
}
