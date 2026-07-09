#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

setup() { :; }
teardown() { print_summary; }
trap teardown EXIT
setup

step "Alias List (empty)"
expect_exit_0 "alias list succeeds when empty" gitfleet alias list

step "Alias Set"
expect_exit_0 "alias set succeeds" gitfleet alias set co "checkout"

step "Alias Set Duplicate Without Force (negative)"
expect_exit_non0 "alias set duplicate without --force fails" gitfleet alias set co "checkout -b"

step "Alias Set Overwrite With Force"
expect_exit_0 "alias set --force overwrites" gitfleet alias set co "checkout -b" --force

step "Alias Get"
expect_exit_0 "alias get succeeds" gitfleet alias get co

step "Alias Get Nonexistent (negative)"
expect_exit_non0 "alias get nonexistent fails" gitfleet alias get nonexistent

step "Alias List (with entries)"
expect_exit_0 "alias list succeeds with entries" gitfleet alias list

step "Alias List JSON"
expect_exit_0 "alias list --json succeeds" gitfleet alias list --json

step "Alias Delete"
expect_exit_0 "alias delete succeeds" gitfleet alias delete co --yes

step "Alias Delete Nonexistent (negative)"
expect_exit_non0 "alias delete nonexistent fails" gitfleet alias delete nonexistent --yes