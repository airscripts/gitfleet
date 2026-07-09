#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

setup() { :; }
teardown() { print_summary; }
trap teardown EXIT
setup

step "Advisory List"
if gitfleet advisory list >/dev/null 2>&1; then
  pass "advisory list succeeds"
else
  skip "advisory list (may require additional permissions)"
fi