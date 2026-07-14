#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

WIKI_TEST_PAGE="Gitfleet-Test-Page-$GITFLEET_PLAYBOOK_RESOURCE_SUFFIX"
WIKI_CREATED=false

setup() { :; }

teardown() {
  if [ "$WIKI_CREATED" = true ]; then
    gitfleet wiki delete "$WIKI_TEST_PAGE" --yes --repo "$GITFLEET_PLAYBOOK_REPO" >/dev/null 2>&1 || true
  fi
  print_summary
}

trap teardown EXIT
setup

step "List Wiki Pages"
if gitfleet wiki list --repo "$GITFLEET_PLAYBOOK_REPO" >/dev/null 2>&1; then
  pass "wiki list succeeds"
else
  skip "wiki operations are not supported by the active provider"
  exit 0
fi

step "View Missing Page"
expect_exit_non0 "wiki view rejects a missing page" gitfleet wiki view "Gitfleet-Missing-$GITFLEET_PLAYBOOK_RESOURCE_SUFFIX" --repo "$GITFLEET_PLAYBOOK_REPO"

step "Create Wiki Page"
if gitfleet wiki create --title "$WIKI_TEST_PAGE" --content "Test content from gitfleet playbook" --repo "$GITFLEET_PLAYBOOK_REPO" >/dev/null 2>&1; then
  pass "wiki create succeeded"
  WIKI_CREATED=true
else
  fail "wiki create failed"
fi

step "View Created Page"
if gitfleet wiki view "$WIKI_TEST_PAGE" --repo "$GITFLEET_PLAYBOOK_REPO" >/dev/null 2>&1; then
  pass "wiki view test page succeeded"
else
  fail "wiki view test page failed"
fi

step "Edit Wiki Page"
if gitfleet wiki edit "$WIKI_TEST_PAGE" --content "Updated content" --repo "$GITFLEET_PLAYBOOK_REPO" >/dev/null 2>&1; then
  pass "wiki edit succeeded"
else
  fail "wiki edit failed"
fi

step "Delete Wiki Page"
if gitfleet wiki delete "$WIKI_TEST_PAGE" --yes --repo "$GITFLEET_PLAYBOOK_REPO" >/dev/null 2>&1; then
  pass "wiki delete succeeded"
  WIKI_CREATED=false
else
  fail "wiki delete failed"
fi
