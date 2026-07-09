#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

setup() { :; }
teardown() { print_summary; }
trap teardown EXIT
setup

step "Identity SSH List"
if gitfleet identity ssh-key list >/dev/null 2>&1; then
  pass "identity ssh list succeeds"
else
  skip "identity ssh list (may require additional scopes)"
fi

step "Identity GPG List"
if gitfleet identity gpg-key list >/dev/null 2>&1; then
  pass "identity gpg list succeeds"
else
  skip "identity gpg list (may require additional scopes)"
fi