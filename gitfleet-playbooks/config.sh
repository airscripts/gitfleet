#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

CONFIG_KEY="gitfleet_test_$PB_RESOURCE_SUFFIX"
CONFIG_SET=false

setup() { :; }

teardown() {
  if [ "$CONFIG_SET" = true ]; then
    gitfleet config unset "$CONFIG_KEY" >/dev/null 2>&1 || true
  fi
  print_summary
}

trap teardown EXIT
setup

step "Config Set Arbitrary Key"
if gitfleet config set "$CONFIG_KEY" "myorg" >/dev/null 2>&1; then
  pass "config set arbitrary key succeeds"
  CONFIG_SET=true
else
  fail "config set arbitrary key failed"
fi

step "Config Get Arbitrary Key"
expect_exit_0 "config get arbitrary key succeeds" gitfleet config get "$CONFIG_KEY"

step "Config Get Unset Key"
expect_exit_0 "config get unset key returns not-set" gitfleet config get unset_key_xyz

step "Config Unset Arbitrary Key"
expect_exit_0 "config unset arbitrary key succeeds" gitfleet config unset "$CONFIG_KEY"
CONFIG_SET=false

step "Config Unset Nonexistent Key Fails"
expect_exit_non0 "config unset nonexistent key fails" gitfleet config unset nonexistent_key_xyz
