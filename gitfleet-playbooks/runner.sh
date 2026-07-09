#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

setup() { :; }
teardown() { print_summary; }
trap teardown EXIT
setup

step "Runner List"
expect_exit_0 "runner list succeeds" gitfleet runner list --repo "$REPO"