use gitfleet_core::operations::{
    get_operation_family, operation_families, validate_capability_contract,
};
use gitfleet_core::provider::{ProviderCapability, ProviderId};

#[test]
fn test_all_operation_families_have_non_empty_names() {
    let families = operation_families();

    assert!(!families.is_empty());

    for f in &families {
        assert!(!f.name.is_empty(), "Operation family has empty name");

        assert!(
            !f.description.is_empty(),
            "Operation family '{}' has empty description",
            f.name
        );
    }
}

#[test]
fn test_operation_families_count() {
    let families = operation_families();

    assert!(
        families.len() >= 40,
        "Expected at least 40 operation families, got {}",
        families.len()
    );
}

#[test]
fn test_get_operation_family_known() {
    let repo = get_operation_family("repo").unwrap();

    assert_eq!(repo.name, "repo");

    assert_eq!(repo.capability, Some(ProviderCapability::Repositories));

    let change = get_operation_family("change").unwrap();

    assert_eq!(change.name, "change");

    assert_eq!(change.capability, Some(ProviderCapability::Changes));

    let auth = get_operation_family("auth").unwrap();

    assert_eq!(auth.name, "auth");

    assert_eq!(auth.capability, None);

    let planning = get_operation_family("planning").unwrap();

    assert_eq!(planning.name, "planning");

    assert_eq!(planning.capability, Some(ProviderCapability::Planning));

    let registry = get_operation_family("registry").unwrap();

    assert_eq!(registry.name, "registry");

    assert_eq!(registry.capability, Some(ProviderCapability::Registry));

    let deps = get_operation_family("deps").unwrap();

    assert_eq!(deps.name, "deps");

    assert_eq!(deps.capability, Some(ProviderCapability::Dependencies));
}

#[test]
fn test_get_operation_family_unknown() {
    assert!(get_operation_family("nonexistent").is_none());

    assert!(get_operation_family("").is_none());
}

#[test]
fn test_legacy_operation_families_are_not_registered() {
    for legacy in [
        "pr",
        "workflow",
        "run",
        "cache",
        "project",
        "milestone",
        "package",
        "dependency",
        "comment",
        "reaction",
        "fork",
    ] {
        assert!(
            get_operation_family(legacy).is_none(),
            "legacy family should not be registered: {legacy}"
        );
    }
}

#[test]
fn test_operation_family_capability_alignment() {
    let families = operation_families();

    let capability_names: Vec<&str> = families
        .iter()
        .filter_map(|f| f.capability.map(|_| f.name))
        .collect();

    assert!(
        !capability_names.is_empty(),
        "At least some families should have capabilities"
    );
}

#[test]
fn test_capability_contract_is_valid() {
    assert!(validate_capability_contract().is_ok());
}

#[test]
fn test_provider_capability_round_trip_serialization() {
    let caps = vec![
        ProviderCapability::Repositories,
        ProviderCapability::Changes,
        ProviderCapability::Issues,
        ProviderCapability::Pipelines,
    ];
    let json = serde_json::to_string(&caps).unwrap();

    let deserialized: Vec<ProviderCapability> = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.len(), caps.len());

    for (original, round_tripped) in caps.iter().zip(deserialized.iter()) {
        assert_eq!(format!("{}", original), format!("{}", round_tripped));
    }
}

#[test]
fn test_provider_id_round_trip_serialization() {
    let ids = vec![ProviderId::GitHub, ProviderId::GitLab];
    let json = serde_json::to_string(&ids).unwrap();

    let deserialized: Vec<ProviderId> = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.len(), 2);

    assert_eq!(deserialized[0], ProviderId::GitHub);
    assert_eq!(deserialized[1], ProviderId::GitLab);
}
