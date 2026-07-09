#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

setup() { :; }
teardown() { print_summary; }
trap teardown EXIT
setup

step "Release List"
expect_exit_0 "release list succeeds" gitfleet release list --repo "$REPO" --limit 5

step "Release View Missing Tag"
expect_exit_non0 "release view rejects a missing tag" gitfleet release view gitfleet-test-missing --repo "$REPO"