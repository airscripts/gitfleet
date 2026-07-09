#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

WEBHOOK_URL="https://example.com/gitfleet-test-webhook"
WEBHOOK_ID=""

setup() {
  : # Webhooks are created and deleted within the playbook.
}

teardown() {
  if [ -n "$WEBHOOK_ID" ]; then
    gitfleet webhook delete "$WEBHOOK_ID" --repo "$REPO" --yes >/dev/null 2>&1 || true
  fi
  print_summary
}

trap teardown EXIT
setup

step "List Webhooks"
expect_exit_0 "webhook list succeeds" gitfleet webhook list --repo "$REPO"

step "Create Webhook"
CREATE_JSON=$(gitfleet webhook create --url "$WEBHOOK_URL" --events push --repo "$REPO" --json 2>&1) || true
WEBHOOK_ID=$(echo "$CREATE_JSON" | python3 -c 'import json,sys; d=json.load(sys.stdin); print(d.get("id",""))' 2>/dev/null || echo "")

if [ -n "$WEBHOOK_ID" ]; then
  pass "webhook create succeeds"
else
  fail "webhook create did not return an id"
fi

step "List Webhooks After Create"
expect_exit_0 "webhook list shows webhook" gitfleet webhook list --repo "$REPO"

step "Test Webhook Ping"
if [ -n "$WEBHOOK_ID" ]; then
  expect_exit_0 "webhook test ping succeeds" gitfleet webhook test "$WEBHOOK_ID" --repo "$REPO"
else
  skip "webhook test ping (no webhook id)"
fi

step "Delete Webhook"
if [ -n "$WEBHOOK_ID" ]; then
  expect_exit_0 "webhook delete succeeds" gitfleet webhook delete "$WEBHOOK_ID" --repo "$REPO" --yes
  WEBHOOK_ID=""
else
  skip "webhook delete (no webhook id)"
fi

step "Delete Missing Webhook"
expect_exit_non0 "webhook delete rejects missing webhook" gitfleet webhook delete 999999 --repo "$REPO" --yes