#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

setup() { :; }
teardown() { print_summary; }
trap teardown EXIT
setup

step "API GET"
expect_exit_0 "api get succeeds" gitfleet api get --endpoint "/repos/$REPO"

step "API POST"
expect_exit_0 "api post succeeds" gitfleet api post --endpoint "/repos/$REPO/labels" --body '{"name":"test-label","color":"f29513"}'

step "API DELETE"
expect_exit_0 "api delete succeeds" gitfleet api delete --endpoint "/repos/$REPO/labels/test-label"

step "API GET with --json"
expect_exit_0 "api get --json succeeds" gitfleet --json api get --endpoint "/repos/$REPO"

step "API POST invalid JSON rejected"
expect_exit_non0 "api post with invalid json fails" gitfleet api post --endpoint "/repos/$REPO" --body 'not-json'
