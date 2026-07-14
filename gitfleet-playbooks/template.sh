#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

setup() { :; }
teardown() { print_summary; }
trap teardown EXIT
setup

step "List Issue Templates"
expect_exit_0 "issue template listing succeeds" gitfleet template list --repo "$GITFLEET_PLAYBOOK_REPO"

step "List Issue Templates As JSON"
expect_exit_0 "JSON issue template listing succeeds" gitfleet template list --repo "$GITFLEET_PLAYBOOK_REPO" --json
