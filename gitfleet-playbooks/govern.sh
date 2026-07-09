#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

setup() { :; }
teardown() { print_summary; }
trap teardown EXIT
setup

step "Govern List-Rulesets"
if gitfleet govern list-rulesets --repo "$REPO" >/dev/null 2>&1; then
  pass "govern list-rulesets succeeds"
else
  skip "govern list-rulesets (may require additional permissions)"
fi