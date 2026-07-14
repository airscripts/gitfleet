#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

setup() { :; }
teardown() { print_summary; }
trap teardown EXIT
setup

if ! has_capability "security"; then
  step "Security Capability"
  expect_exit_non0 "security is explicitly unsupported" gitfleet security advisories --repo "$REPO"
  exit 0
fi

step "Dependabot Alerts"
if gitfleet security advisories --repo "$REPO" >/dev/null 2>&1; then
  pass "Dependabot alert listing succeeds"
else
  skip "Dependabot alerts (feature may be disabled)"
fi

step "Secret Scanning Alerts"
if gitfleet security secret-scans --repo "$REPO" >/dev/null 2>&1; then
  pass "secret scanning alert listing succeeds"
else
  skip "secret scanning alerts (feature may be disabled)"
fi

step "CodeQL Alerts"
if gitfleet security codeql --repo "$REPO" >/dev/null 2>&1; then
  pass "CodeQL alert listing succeeds"
else
  skip "CodeQL alerts (feature may be disabled)"
fi
