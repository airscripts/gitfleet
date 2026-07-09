#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

LABEL_NAME="gitfleet-test-label"

setup() { :; }

teardown() {
  gitfleet label delete "$LABEL_NAME" --yes --repo "$REPO" >/dev/null 2>&1 || true
  print_summary
}

trap teardown EXIT
setup

step "List Labels"
expect_exit_0 "labels list succeeds" gitfleet label list --repo "$REPO"

step "Create Label"
expect_exit_0 "labels create succeeds" gitfleet label create "$LABEL_NAME" --color ff0000 --description "Test label from gitfleet" --repo "$REPO"

step "Delete Label"
expect_exit_0 "labels delete succeeds" gitfleet label delete "$LABEL_NAME" --yes --repo "$REPO"

step "Delete Label Without --yes"
expect_exit_non0 "labels delete fails without --yes" gitfleet label delete "$LABEL_NAME" --repo "$REPO"