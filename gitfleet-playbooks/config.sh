#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

ORIGINAL_TOKEN=""

setup() {
  ORIGINAL_TOKEN=$(gitfleet config get token 2>/dev/null || true)
}

teardown() {
  if [ -n "$ORIGINAL_TOKEN" ]; then
    gitfleet config set token "$ORIGINAL_TOKEN" >/dev/null 2>&1 || true
  fi
  print_summary
}

trap teardown EXIT
setup

step "Config Set Arbitrary Key"
expect_exit_0 "config set arbitrary key succeeds" gitfleet config set default_org "myorg"

step "Config Get Arbitrary Key"
expect_exit_0 "config get arbitrary key succeeds" gitfleet config get default_org

step "Config Get Unset Key"
expect_exit_0 "config get unset key returns not-set" gitfleet config get unset_key_xyz

step "Config Unset Arbitrary Key"
expect_exit_0 "config unset arbitrary key succeeds" gitfleet config unset default_org

step "Config Unset Nonexistent Key Fails"
expect_exit_non0 "config unset nonexistent key fails" gitfleet config unset nonexistent_key_xyz