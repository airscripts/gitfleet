#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

WORKSPACE_NAME="gitfleet-test-workspace"

setup() { :; }

teardown() {
  gitfleet workspace remove "$WORKSPACE_NAME" >/dev/null 2>&1 || true
  print_summary
}

trap teardown EXIT
setup

step "Workspace List"
expect_exit_0 "workspace list succeeds" gitfleet workspace list

step "Workspace Define"
expect_exit_0 "workspace define succeeds" gitfleet workspace define --name "$WORKSPACE_NAME" --repos "$REPO"

step "Workspace List After Define"
expect_exit_0 "workspace list after define succeeds" gitfleet workspace list

step "Workspace Remove"
expect_exit_0 "workspace remove succeeds" gitfleet workspace remove "$WORKSPACE_NAME"