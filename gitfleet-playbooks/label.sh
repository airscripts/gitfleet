#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

LABEL_NAME="gitfleet-test-label-$GITFLEET_PLAYBOOK_RESOURCE_SUFFIX"
LEGACY_LABEL="gitfleet-test-enhancement-$GITFLEET_PLAYBOOK_RESOURCE_SUFFIX"
RENAMED_LABEL="gitfleet-test-feature-$GITFLEET_PLAYBOOK_RESOURCE_SUFFIX"
LABEL_CREATED=false
LEGACY_CREATED=false
TEMPLATE_FILE=""

setup() { :; }

teardown() {
  if [ "$LABEL_CREATED" = true ]; then
    gitfleet label delete "$LABEL_NAME" --yes --repo "$GITFLEET_PLAYBOOK_REPO" >/dev/null 2>&1 || true
  fi
  if [ "$LEGACY_CREATED" = true ]; then
    gitfleet label delete "$LEGACY_LABEL" --yes --repo "$GITFLEET_PLAYBOOK_REPO" >/dev/null 2>&1 || true
  fi
  if [ -n "$TEMPLATE_FILE" ]; then
    rm -f "$TEMPLATE_FILE"
  fi
  print_summary
}

trap teardown EXIT
setup

step "List Labels"
expect_exit_0 "labels list succeeds" gitfleet label list --repo "$GITFLEET_PLAYBOOK_REPO"

step "Create Label"
if gitfleet label create "$LABEL_NAME" --color ff0000 --description "Test label from gitfleet" --repo "$GITFLEET_PLAYBOOK_REPO" >/dev/null 2>&1; then
  pass "labels create succeeds"
  LABEL_CREATED=true
else
  fail "labels create failed"
fi

step "Delete Label"
expect_exit_0 "labels delete succeeds" gitfleet label delete "$LABEL_NAME" --yes --repo "$GITFLEET_PLAYBOOK_REPO"
LABEL_CREATED=false

step "Delete Label Without --yes"
expect_exit_non0 "labels delete fails without --yes" gitfleet label delete "$LABEL_NAME" --repo "$GITFLEET_PLAYBOOK_REPO"

step "Rename Label Through Template"
if gitfleet label create "$LEGACY_LABEL" --color a2eeef --description "Legacy feature label" --repo "$GITFLEET_PLAYBOOK_REPO" >/dev/null 2>&1; then
  LEGACY_CREATED=true
  TEMPLATE_FILE=$(mktemp)
  cat >"$TEMPLATE_FILE" <<EOF
version = 1
name = "playbook-rename"

[[labels]]
name = "$RENAMED_LABEL"
rename_from = ["$LEGACY_LABEL"]
color = "1d7a1d"
description = "Renamed feature label"
EOF
  if gitfleet label sync --file "$TEMPLATE_FILE" --repo "$GITFLEET_PLAYBOOK_REPO" --yes >/dev/null 2>&1; then
    pass "template label rename succeeds"
    LEGACY_CREATED=false
    LABEL_NAME="$RENAMED_LABEL"
    LABEL_CREATED=true
    expect_output "renamed label is present" "$RENAMED_LABEL" gitfleet label list --repo "$GITFLEET_PLAYBOOK_REPO"
  else
    fail "template label rename succeeds (exited non-zero)"
  fi
else
  fail "legacy label creation failed"
fi

step "Label Replacement Dry Run"
if [ -n "$TEMPLATE_FILE" ]; then
  expect_exit_0 "label replacement dry run succeeds" gitfleet label sync --file "$TEMPLATE_FILE" --repo "$GITFLEET_PLAYBOOK_REPO" --replace --dry-run --json
else
  fail "label replacement dry run skipped because template setup failed"
fi
