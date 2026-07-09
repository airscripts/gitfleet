#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

PROJECT_ID=""

setup() { :; }

teardown() {
  if [ -n "$PROJECT_ID" ]; then
    gitfleet planning project delete "$PROJECT_ID" --yes >/dev/null 2>&1 || true
  fi
  print_summary
}

trap teardown EXIT
setup

step "Project List"
expect_exit_0 "project list succeeds" gitfleet planning project list --owner "$OWNER"

step "Project Create"
output=$(gitfleet planning project create "gitfleet-test-project" --owner "$OWNER" --json 2>&1) || true
PROJECT_ID=$(echo "$output" | python3 -c "import sys,json; print(json.load(sys.stdin).get('number',''))" 2>/dev/null || echo "")

if [ -n "$PROJECT_ID" ]; then
  pass "project create succeeded (id=$PROJECT_ID)"
else
  fail "project create failed"
fi

step "Project View"
if [ -n "$PROJECT_ID" ]; then
  expect_exit_0 "project view succeeds" gitfleet planning project view "$PROJECT_ID"
else
  skip "project view (create failed)"
fi

step "Project Delete"
if [ -n "$PROJECT_ID" ]; then
  expect_exit_0 "project delete succeeds" gitfleet planning project delete "$PROJECT_ID" --yes
  PROJECT_ID=""
else
  skip "project delete (create failed)"
fi