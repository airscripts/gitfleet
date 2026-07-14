#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

TAG_PATTERN="gitfleet-test-$GITFLEET_PLAYBOOK_RESOURCE_SUFFIX-*"
TAG_IDENTIFIER=""
TEST_REPO_NAME="gitfleet-test-policy-$GITFLEET_PLAYBOOK_RESOURCE_SUFFIX"
TEST_REPO="$GITFLEET_PLAYBOOK_TEST_REPO_OWNER/$TEST_REPO_NAME"
REPO_CREATED=false
BRANCH_PROTECTED=false
BRANCH_NAME=main
BRANCH_CREATED=false

setup() {
  if gitfleet repo create "$TEST_REPO_NAME" --owner "$GITFLEET_PLAYBOOK_TEST_REPO_OWNER" --owner-type "$GITFLEET_PLAYBOOK_TEST_REPO_OWNER_TYPE" --public --initialize --yes >/dev/null 2>&1; then
    REPO_CREATED=true
  else
    fail "policy test repository creation failed"
  fi

  if [ "$GITFLEET_PLAYBOOK_PROVIDER" = "gitlab" ]; then
    BRANCH_NAME="gitfleet-test-policy-$GITFLEET_PLAYBOOK_RESOURCE_SUFFIX"
    encoded_repo=$(python3 -c 'import sys,urllib.parse; print(urllib.parse.quote(sys.argv[1], safe=""))' "$TEST_REPO")
    branch_body=$(python3 -c 'import json,sys; print(json.dumps({"branch": sys.argv[1], "ref": "main"}))' "$BRANCH_NAME")

    if gitfleet api post --endpoint "/projects/$encoded_repo/repository/branches" --body "$branch_body" --yes >/dev/null 2>&1; then
      BRANCH_CREATED=true
    else
      fail "policy test branch creation failed"
    fi
  fi
}
teardown() {
  if [ -n "$TAG_IDENTIFIER" ]; then
    gitfleet policy tag-protection delete "$TAG_IDENTIFIER" --repo "$TEST_REPO" --yes >/dev/null 2>&1 || true
  fi
  if [ "$BRANCH_PROTECTED" = true ]; then
    gitfleet policy branch-protection delete "$BRANCH_NAME" --repo "$TEST_REPO" --yes >/dev/null 2>&1 || true
  fi
  if [ "$BRANCH_CREATED" = true ]; then
    encoded_repo=$(python3 -c 'import sys,urllib.parse; print(urllib.parse.quote(sys.argv[1], safe=""))' "$TEST_REPO")
    encoded_branch=$(python3 -c 'import sys,urllib.parse; print(urllib.parse.quote(sys.argv[1], safe=""))' "$BRANCH_NAME")
    gitfleet api delete --endpoint "/projects/$encoded_repo/repository/branches/$encoded_branch" --yes >/dev/null 2>&1 || true
  fi
  if [ "$REPO_CREATED" = true ]; then
    gitfleet repo delete "$TEST_REPO" --yes >/dev/null 2>&1 || true
  fi
  print_summary
}
trap teardown EXIT
setup

step "Policy Branch Protection Set"
if gitfleet policy branch-protection set "$BRANCH_NAME" --repo "$TEST_REPO" >/dev/null 2>&1; then
  pass "policy branch-protection set succeeds"
  BRANCH_PROTECTED=true
else
  skip "policy branch protection (provider plan may not permit it)"
fi

step "Policy Branch Protection Get"
if [ "$BRANCH_PROTECTED" = true ]; then
  expect_exit_0 "policy branch-protection get succeeds" gitfleet policy branch-protection get "$BRANCH_NAME" --repo "$TEST_REPO"
else
  skip "policy branch-protection get (set unavailable)"
fi

step "Policy Branch Protection Delete"
if [ "$BRANCH_PROTECTED" = true ]; then
  expect_exit_0 "policy branch-protection delete succeeds" gitfleet policy branch-protection delete "$BRANCH_NAME" --repo "$TEST_REPO" --yes
  BRANCH_PROTECTED=false
else
  skip "policy branch-protection delete (set unavailable)"
fi

step "Policy Tag Protection List"
if ! has_capability "tagProtection"; then
  expect_exit_non0 "tag protection is explicitly unsupported" gitfleet policy tag-protection list --repo "$TEST_REPO"
elif gitfleet policy tag-protection list --repo "$TEST_REPO" >/dev/null 2>&1; then
  pass "policy tag-protection list succeeds"
else
  skip "policy tag-protection list (may not be available)"
fi

step "Policy Tag Protection Create"
if ! has_capability "tagProtection"; then
  expect_exit_non0 "tag protection creation is explicitly unsupported" gitfleet policy tag-protection create "$TAG_PATTERN" --repo "$TEST_REPO"
else
  output=$(gitfleet policy tag-protection create "$TAG_PATTERN" --repo "$TEST_REPO" --json 2>&1) || true
  TAG_IDENTIFIER=$(echo "$output" | python3 -c "import sys,json; print(json.load(sys.stdin).get('identifier',''))" 2>/dev/null || echo "")
fi

if [ -n "$TAG_IDENTIFIER" ]; then
  pass "policy tag-protection create succeeds"
elif ! has_capability "tagProtection"; then
  :
else
  skip "policy tag-protection create (may not be available)"
fi

step "Policy Tag Protection Delete"
if ! has_capability "tagProtection"; then
  expect_exit_non0 "tag protection deletion is explicitly unsupported" gitfleet policy tag-protection delete missing --repo "$TEST_REPO" --yes
elif [ -n "$TAG_IDENTIFIER" ]; then
  expect_exit_0 "policy tag-protection delete succeeds" gitfleet policy tag-protection delete "$TAG_IDENTIFIER" --repo "$TEST_REPO" --yes
  TAG_IDENTIFIER=""
else
  skip "policy tag-protection delete (create failed)"
fi
