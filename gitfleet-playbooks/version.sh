#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

setup() { :; }
teardown() { print_summary; }
trap teardown EXIT
setup

step "Version"
expect_exit_0 "version succeeds" gitfleet version

step "Version JSON"
expect_json_field "version JSON has version" "version" "0.1.0" gitfleet version