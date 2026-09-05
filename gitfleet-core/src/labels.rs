use std::collections::{HashMap, HashSet};

use serde::Serialize;

use crate::errors::{GitfleetError, PartialFailureError, UnprocessableError};
use crate::provider::LabelOps;
use crate::types::{Label, LabelTemplate, LabelTemplateEntry};

const TEMPLATE_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LabelActionKind {
    Create,
    Update,
    Rename,
    Delete,
}

#[derive(Debug, Clone, Serialize)]
pub struct LabelAction {
    pub kind: LabelActionKind,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(skip_serializing)]
    pub label: Option<Label>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LabelSyncPlan {
    pub actions: Vec<LabelAction>,
    pub preserved: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LabelActionResult {
    pub kind: LabelActionKind,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct LabelSyncCounts {
    pub created: usize,
    pub updated: usize,
    pub renamed: usize,
    pub deleted: usize,
    pub preserved: usize,
    pub skipped: usize,
    pub failed: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct LabelSyncReport {
    pub actions: Vec<LabelActionResult>,
    pub preserved: Vec<String>,
    pub skipped: Vec<String>,
    pub counts: LabelSyncCounts,
    pub failed: bool,
}

pub fn builtins() -> Result<Vec<LabelTemplate>, GitfleetError> {
    [
        include_str!("../templates/github.toml"),
        include_str!("../templates/gitfleet.toml"),
    ]
    .into_iter()
    .map(parse_template)
    .collect()
}

pub fn builtin(name: &str) -> Result<LabelTemplate, GitfleetError> {
    builtins()?
        .into_iter()
        .find(|template| template.name == name)
        .ok_or_else(|| {
            GitfleetError::from(UnprocessableError::new(format!(
                "Unknown label template '{name}'."
            )))
        })
}

pub fn parse_template(content: &str) -> Result<LabelTemplate, GitfleetError> {
    let template: LabelTemplate = toml::from_str(content).map_err(|error| {
        GitfleetError::from(UnprocessableError::new(format!(
            "Invalid label template: {error}"
        )))
    })?;

    validate_template(&template)?;

    Ok(template)
}

pub fn validate_template(template: &LabelTemplate) -> Result<(), GitfleetError> {
    if template.version != TEMPLATE_VERSION {
        return Err(GitfleetError::from(UnprocessableError::new(format!(
            "Unsupported label template version {}.",
            template.version
        ))));
    }

    if template.name.trim().is_empty() {
        return Err(GitfleetError::from(UnprocessableError::new(
            "Label template name cannot be empty.",
        )));
    }

    if template.labels.is_empty() {
        return Err(GitfleetError::from(UnprocessableError::new(
            "Label template must contain at least one label.",
        )));
    }

    let mut names = HashSet::new();

    for entry in &template.labels {
        validate_entry(entry)?;

        let name = normalize_name(&entry.name);
        if !names.insert(name) {
            return Err(GitfleetError::from(UnprocessableError::new(format!(
                "Label template contains duplicate label '{}'.",
                entry.name
            ))));
        }
    }

    for entry in &template.labels {
        for alias in &entry.rename_from {
            let alias_name = normalize_name(alias);
            if names.contains(&alias_name) {
                return Err(GitfleetError::from(UnprocessableError::new(format!(
                    "Label alias '{}' is also a desired label.",
                    alias
                ))));
            }
            if !names.insert(alias_name) {
                return Err(GitfleetError::from(UnprocessableError::new(format!(
                    "Label template contains duplicate alias '{}'.",
                    alias
                ))));
            }
        }
    }

    Ok(())
}

pub fn plan(
    template: &LabelTemplate,
    current: &[Label],
    replace: bool,
) -> Result<LabelSyncPlan, GitfleetError> {
    validate_template(template)?;

    let mut current_by_name = HashMap::new();
    for (index, label) in current.iter().enumerate() {
        current_by_name.insert(normalize_name(&label.name), index);
    }

    let mut matched = HashSet::new();
    let mut actions = Vec::new();

    for entry in &template.labels {
        let desired = entry_to_label(entry);
        let desired_key = normalize_name(&entry.name);

        if let Some(index) = current_by_name.get(&desired_key).copied() {
            for alias in &entry.rename_from {
                if current_by_name.contains_key(&normalize_name(alias)) {
                    return Err(rename_collision(&entry.name, alias));
                }
            }

            matched.insert(index);
            if !same_label(&current[index], &desired) {
                actions.push(LabelAction {
                    kind: LabelActionKind::Update,
                    name: entry.name.clone(),
                    from: Some(current[index].name.clone()),
                    label: Some(desired),
                });
            }

            continue;
        }

        let aliases: Vec<usize> = entry
            .rename_from
            .iter()
            .filter_map(|alias| current_by_name.get(&normalize_name(alias)).copied())
            .collect();

        match aliases.as_slice() {
            [] => actions.push(LabelAction {
                kind: LabelActionKind::Create,
                name: entry.name.clone(),
                from: None,
                label: Some(desired),
            }),
            [index] => {
                matched.insert(*index);
                actions.push(LabelAction {
                    kind: LabelActionKind::Rename,
                    name: entry.name.clone(),
                    from: Some(current[*index].name.clone()),
                    label: Some(desired),
                });
            }
            _ => {
                return Err(GitfleetError::from(UnprocessableError::new(format!(
                    "Multiple rename sources exist for '{}'.",
                    entry.name
                ))));
            }
        }
    }

    let preserved: Vec<String> = current
        .iter()
        .enumerate()
        .filter(|(index, _)| !matched.contains(index))
        .map(|(_, label)| label.name.clone())
        .collect();

    if replace {
        for name in &preserved {
            actions.push(LabelAction {
                kind: LabelActionKind::Delete,
                name: name.clone(),
                from: None,
                label: None,
            });
        }
    }

    Ok(LabelSyncPlan {
        actions,
        preserved: if replace { Vec::new() } else { preserved },
    })
}

pub async fn execute(plan: &LabelSyncPlan, ops: &dyn LabelOps, repo: &str) -> LabelSyncReport {
    let mut results = Vec::new();
    let mut failed = false;
    let mut upsert_failed = false;

    for action in &plan.actions {
        if action.kind == LabelActionKind::Delete {
            continue;
        }

        let Some(label) = action.label.as_ref() else {
            failed = true;
            upsert_failed = true;
            results.push(action_result(
                action,
                "failed",
                Some("Label action is missing its desired label data.".to_string()),
            ));
            continue;
        };

        let result = match action.kind {
            LabelActionKind::Create => ops.create_label(label, repo).await.map(|_| ()),
            LabelActionKind::Update | LabelActionKind::Rename => ops
                .update_label(action.from.as_deref().unwrap_or(&action.name), label, repo)
                .await
                .map(|_| ()),
            LabelActionKind::Delete => continue,
        };

        match result {
            Ok(_) => results.push(action_result(action, "applied", None)),
            Err(error) => {
                failed = true;
                upsert_failed = true;
                results.push(action_result(action, "failed", Some(error.to_string())));
            }
        }
    }

    for action in &plan.actions {
        if action.kind != LabelActionKind::Delete {
            continue;
        }

        if upsert_failed {
            failed = true;
            results.push(action_result(
                action,
                "skipped",
                Some("Skipped because a label create or update failed.".to_string()),
            ));
            continue;
        }

        match ops.delete_label(&action.name, repo).await {
            Ok(()) => results.push(action_result(action, "applied", None)),
            Err(error) => {
                failed = true;
                results.push(action_result(action, "failed", Some(error.to_string())));
            }
        }
    }

    let skipped = results
        .iter()
        .filter(|result| result.status == "skipped")
        .map(|result| result.name.clone())
        .collect();

    let counts = counts_from_results(&results, plan.preserved.len());

    LabelSyncReport {
        actions: results,
        preserved: plan.preserved.clone(),
        skipped,
        counts,
        failed,
    }
}

pub fn plan_counts(plan: &LabelSyncPlan) -> LabelSyncCounts {
    let mut counts = LabelSyncCounts {
        preserved: plan.preserved.len(),
        ..LabelSyncCounts::default()
    };

    for action in &plan.actions {
        increment_kind(&mut counts, action.kind);
    }

    counts
}

pub fn ensure_success(report: &LabelSyncReport) -> Result<(), GitfleetError> {
    if report.failed {
        return Err(GitfleetError::from(PartialFailureError::new(
            "Label synchronization completed with failures.",
        )));
    }

    Ok(())
}

fn validate_entry(entry: &LabelTemplateEntry) -> Result<(), GitfleetError> {
    if entry.name.trim().is_empty() {
        return Err(GitfleetError::from(UnprocessableError::new(
            "Label names cannot be empty.",
        )));
    }

    let color = entry.color.trim_start_matches('#');
    if color.len() != 6 || !color.chars().all(|character| character.is_ascii_hexdigit()) {
        return Err(GitfleetError::from(UnprocessableError::new(format!(
            "Label '{}' has an invalid color '{}'.",
            entry.name, entry.color
        ))));
    }

    if entry.description.chars().count() > 100 {
        return Err(GitfleetError::from(UnprocessableError::new(format!(
            "Description for '{}' exceeds 100 characters.",
            entry.name
        ))));
    }

    for alias in &entry.rename_from {
        if alias.trim().is_empty() {
            return Err(GitfleetError::from(UnprocessableError::new(
                "Label rename aliases cannot be empty.",
            )));
        }
    }

    Ok(())
}

fn entry_to_label(entry: &LabelTemplateEntry) -> Label {
    Label {
        name: entry.name.clone(),
        color: entry.color.trim_start_matches('#').to_ascii_lowercase(),
        new_name: None,
        description: entry.description.clone(),
    }
}

fn same_label(current: &Label, desired: &Label) -> bool {
    current.name == desired.name
        && current
            .color
            .trim_start_matches('#')
            .eq_ignore_ascii_case(&desired.color)
        && current.description == desired.description
}

fn normalize_name(name: &str) -> String {
    name.to_lowercase()
}

fn rename_collision(desired: &str, alias: &str) -> GitfleetError {
    GitfleetError::from(UnprocessableError::new(format!(
        "Cannot rename '{alias}' to '{desired}' because both labels already exist.",
    )))
}

fn action_result(action: &LabelAction, status: &str, error: Option<String>) -> LabelActionResult {
    LabelActionResult {
        kind: action.kind,
        name: action.name.clone(),
        from: action.from.clone(),
        status: status.to_string(),
        error,
    }
}

fn counts_from_results(results: &[LabelActionResult], preserved: usize) -> LabelSyncCounts {
    let mut counts = LabelSyncCounts {
        preserved,
        ..LabelSyncCounts::default()
    };

    for result in results {
        match result.status.as_str() {
            "applied" => increment_kind(&mut counts, result.kind),
            "skipped" => counts.skipped += 1,
            "failed" => counts.failed += 1,
            _ => {}
        }
    }

    counts
}

fn increment_kind(counts: &mut LabelSyncCounts, kind: LabelActionKind) {
    match kind {
        LabelActionKind::Create => counts.created += 1,
        LabelActionKind::Update => counts.updated += 1,
        LabelActionKind::Rename => counts.renamed += 1,
        LabelActionKind::Delete => counts.deleted += 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_validates_template() {
        let template = parse_template(
            r##"
version = 1
name = "team"

[[labels]]
name = "feature"
rename_from = ["enhancement"]
color = "#a2eeef"
description = "New feature"
"##,
        )
        .unwrap();

        assert_eq!(template.labels[0].color, "#a2eeef");
    }

    #[test]
    fn plans_alias_rename_without_delete() {
        let template = builtin("gitfleet").unwrap();
        let current = vec![Label {
            name: "Enhancement".into(),
            color: "a2eeef".into(),
            new_name: None,
            description: "New feature or request".into(),
        }];

        let plan = plan(&template, &current, true).unwrap();
        assert!(
            plan.actions
                .iter()
                .any(|action| action.kind == LabelActionKind::Rename && action.name == "feature")
        );
        assert_eq!(
            plan.actions
                .iter()
                .find(|action| action.kind == LabelActionKind::Rename)
                .and_then(|action| action.from.as_deref()),
            Some("Enhancement")
        );
        assert!(
            !plan.actions.iter().any(
                |action| action.kind == LabelActionKind::Delete && action.name == "enhancement"
            )
        );
    }

    #[test]
    fn rejects_existing_destination_and_alias() {
        let mut template = builtin("gitfleet").unwrap();
        template.labels.retain(|entry| entry.name == "feature");
        let current = vec![
            Label {
                name: "feature".into(),
                color: "1d7a1d".into(),
                new_name: None,
                description: "New feature or request.".into(),
            },
            Label {
                name: "enhancement".into(),
                color: "a2eeef".into(),
                new_name: None,
                description: "New feature or request".into(),
            },
        ];

        assert!(plan(&template, &current, true).is_err());
    }

    #[test]
    fn plans_case_only_name_update() {
        let template = LabelTemplate {
            version: TEMPLATE_VERSION,
            name: "case".into(),
            description: String::new(),
            labels: vec![LabelTemplateEntry {
                name: "bug".into(),
                rename_from: Vec::new(),
                color: "d73a4a".into(),
                description: "Bug".into(),
            }],
        };
        let current = vec![Label {
            name: "Bug".into(),
            color: "d73a4a".into(),
            new_name: None,
            description: "Bug".into(),
        }];

        let plan = plan(&template, &current, false).unwrap();
        assert_eq!(plan.actions.len(), 1);
        assert_eq!(plan.actions[0].kind, LabelActionKind::Update);
        assert_eq!(plan.actions[0].from.as_deref(), Some("Bug"));
    }
}
