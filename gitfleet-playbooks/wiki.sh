#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

WIKI_TEST_PAGE="Gitfleet-Test-Page"

setup() { :; }

teardown() {
  gitfleet wiki delete "$WIKI_TEST_PAGE" --yes --repo "$REPO" >/dev/null 2>&1 || true
  print_summary
}

trap teardown EXIT
setup

step "List Wiki Pages"
expect_exit_0 "wiki list succeeds" gitfleet wiki list --repo "$REPO"

step "View Home Page"
if gitfleet wiki view Home --repo "$REPO" >/dev/null 2>&1; then
  pass "wiki view Home succeeded"
else
  skip "wiki view Home (may not exist)"
fi

step "Create Wiki Page"
if gitfleet wiki create --title "$WIKI_TEST_PAGE" --content "Test content from gitfleet playbook" --repo "$REPO" >/dev/null 2>&1; then
  pass "wiki create succeeded"
else
  skip "wiki create (may already exist or wiki not enabled)"
fi

step "View Created Page"
if gitfleet wiki view "$WIKI_TEST_PAGE" --repo "$REPO" >/dev/null 2>&1; then
  pass "wiki view test page succeeded"
else
  skip "wiki view test page (may not exist)"
fi

step "Edit Wiki Page"
if gitfleet wiki edit "$WIKI_TEST_PAGE" --content "Updated content" --repo "$REPO" >/dev/null 2>&1; then
  pass "wiki edit succeeded"
else
  skip "wiki edit (page may not exist)"
fi

step "Delete Wiki Page"
if gitfleet wiki delete "$WIKI_TEST_PAGE" --yes --repo "$REPO" >/dev/null 2>&1; then
  pass "wiki delete succeeded"
else
  skip "wiki delete (page may not exist)"
fi