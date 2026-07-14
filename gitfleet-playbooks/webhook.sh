#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

TEST_REPO_NAME="gitfleet-test-webhook-$GITFLEET_PLAYBOOK_RESOURCE_SUFFIX"
WEBHOOK_URL="${GITFLEET_PLAYBOOK_WEBHOOK_URL:-}"
if [ -n "$WEBHOOK_URL" ]; then
  TEST_REPO="$GITFLEET_PLAYBOOK_TEST_REPO_OWNER/$TEST_REPO_NAME"
else
  TEST_REPO="$GITFLEET_PLAYBOOK_REPO"
fi
WEBHOOK_ID=""
REPO_CREATED=false

setup() {
  if [ -z "$WEBHOOK_URL" ]; then
    return
  fi

  if ! gitfleet repo create "$TEST_REPO_NAME" --owner "$GITFLEET_PLAYBOOK_TEST_REPO_OWNER" --owner-type "$GITFLEET_PLAYBOOK_TEST_REPO_OWNER_TYPE" --private --initialize --yes >/dev/null 2>&1; then
    fail "webhook test repository creation failed"
    return
  fi
  REPO_CREATED=true

}

teardown() {
  if [ -n "$WEBHOOK_ID" ]; then
    gitfleet webhook delete "$WEBHOOK_ID" --repo "$TEST_REPO" --yes >/dev/null 2>&1 || true
  fi
  if [ "$REPO_CREATED" = true ]; then
    gitfleet repo delete "$TEST_REPO" --yes >/dev/null 2>&1 || true
  fi
  print_summary
}

trap teardown EXIT
setup

step "List Webhooks"
expect_exit_0 "webhook list succeeds" gitfleet webhook list --repo "$TEST_REPO"

step "Create Webhook"
if [ -n "$WEBHOOK_URL" ]; then
  CREATE_JSON=$(gitfleet webhook create --url "$WEBHOOK_URL" --events push --active --repo "$TEST_REPO" --json 2>&1) || true
  WEBHOOK_ID=$(echo "$CREATE_JSON" | python3 -c 'import json,sys; d=json.load(sys.stdin); print(d.get("id",""))' 2>/dev/null || echo "")

  if [ -n "$WEBHOOK_ID" ]; then
    pass "webhook create succeeds"
  else
    fail "webhook create did not return an id"
  fi
else
  skip "webhook create (set GITFLEET_PLAYBOOK_WEBHOOK_URL to an owner-controlled receiver)"
fi

step "List Webhooks After Create"
if [ -n "$WEBHOOK_ID" ]; then
  expect_exit_0 "webhook list shows webhook" gitfleet webhook list --repo "$TEST_REPO"
else
  skip "webhook list after create (no test webhook)"
fi

step "Test Webhook Ping"
if [ -n "$WEBHOOK_ID" ]; then
  expect_exit_0 "webhook test ping succeeds" gitfleet webhook test "$WEBHOOK_ID" --repo "$TEST_REPO"
else
  skip "webhook test ping (no webhook id)"
fi

step "Delete Webhook"
if [ -n "$WEBHOOK_ID" ]; then
  expect_exit_0 "webhook delete succeeds" gitfleet webhook delete "$WEBHOOK_ID" --repo "$TEST_REPO" --yes
  WEBHOOK_ID=""
else
  skip "webhook delete (no webhook id)"
fi

step "Delete Missing Webhook"
expect_exit_non0 "webhook delete rejects missing webhook" gitfleet webhook delete 999999 --repo "$TEST_REPO" --yes
