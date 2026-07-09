#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

setup() { :; }
teardown() { print_summary; }
trap teardown EXIT
setup

step "Inbox List"
expect_exit_0 "inbox list succeeds" gitfleet inbox list

step "Inbox List --repo"
expect_exit_0 "inbox list --repo succeeds" gitfleet inbox list --repo "$REPO"

step "Inbox List --json"
expect_exit_0 "inbox list --json succeeds" gitfleet inbox list --json

step "Inbox Mark-Read"
expect_exit_0 "inbox mark-read succeeds" gitfleet inbox mark-read