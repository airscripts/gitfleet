#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

setup() { :; }
teardown() { print_summary; }
trap teardown EXIT
setup

step "Search Issues"
expect_exit_0 "search issues succeeds" gitfleet search issues "bug" --limit 5 --page 1

step "Search Repos"
expect_exit_0 "search repos succeeds" gitfleet search repos "gitfleet" --limit 5 --page 1
