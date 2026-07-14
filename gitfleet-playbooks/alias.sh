#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

setup() { :; }
ALIAS_NAME="gitfleet-test-$GITFLEET_PLAYBOOK_RESOURCE_SUFFIX"
ALIAS_CREATED=false
teardown() {
  if [ "$ALIAS_CREATED" = true ]; then
    gitfleet alias delete "$ALIAS_NAME" --yes >/dev/null 2>&1 || true
  fi
  print_summary
}
trap teardown EXIT
setup

step "Alias List (empty)"
expect_exit_0 "alias list succeeds when empty" gitfleet alias list

step "Alias Set"
if gitfleet alias set "$ALIAS_NAME" "checkout" >/dev/null 2>&1; then
  pass "alias set succeeds"
  ALIAS_CREATED=true
else
  fail "alias set failed"
fi

step "Alias Set Duplicate Without Force (negative)"
expect_exit_non0 "alias set duplicate without --force fails" gitfleet alias set "$ALIAS_NAME" "checkout -b"

step "Alias Set Overwrite With Force"
expect_exit_0 "alias set --force overwrites" gitfleet alias set "$ALIAS_NAME" "checkout -b" --force

step "Alias Get"
expect_exit_0 "alias get succeeds" gitfleet alias get "$ALIAS_NAME"

step "Alias Get Nonexistent (negative)"
expect_exit_non0 "alias get nonexistent fails" gitfleet alias get nonexistent

step "Alias List (with entries)"
expect_exit_0 "alias list succeeds with entries" gitfleet alias list

step "Alias List JSON"
expect_exit_0 "alias list --json succeeds" gitfleet alias list --json

step "Alias Delete"
expect_exit_0 "alias delete succeeds" gitfleet alias delete "$ALIAS_NAME" --yes
ALIAS_CREATED=false

step "Alias Delete Nonexistent (negative)"
expect_exit_non0 "alias delete nonexistent fails" gitfleet alias delete nonexistent --yes
