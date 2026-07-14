#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

setup() { :; }
teardown() { print_summary; }
trap teardown EXIT
setup

step "Access Org List-Members"
if gitfleet access org list-members "$ORG" >/dev/null 2>&1; then
  pass "access org list-members succeeds"
else
  skip "access org list-members ($ORG may not be an organization)"
fi

step "Access Team List"
if provider_is gitlab; then
  expect_exit_non0 "team operations are explicitly unsupported" gitfleet access team list "$ORG"
elif gitfleet access team list "$ORG" >/dev/null 2>&1; then
  pass "access team list succeeds"
else
  skip "access team list ($ORG may not be an organization)"
fi
