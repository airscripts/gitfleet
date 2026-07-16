#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
CRATES=(gitfleet-core gitfleet gitfleet-providers)
SELECTED_CRATE=""

usage() {
    printf 'Usage: %s [-a|--all] [-c|--crate CRATE]\n' "$(basename "$0")"
    printf '\nCount non-empty Rust source lines in the workspace.\n'
    printf '  -a, --all    print one total for the workspace (default)\n'
    printf '  -c, --crate  count one crate: core, cli, or providers\n'
    printf '  -h, --help   show this help\n'
}

while (($# > 0)); do
    case "$1" in
        -a|--all)
            if [[ -n "$SELECTED_CRATE" ]]; then
                printf 'Error: --all and --crate cannot be combined.\n' >&2
                exit 2
            fi
            ;;
        -c|--crate)
            if (($# < 2)); then
                printf 'Error: --crate requires core, cli, or providers.\n' >&2
                exit 2
            fi
            case "$2" in
                core|gitfleet-core) SELECTED_CRATE=gitfleet-core ;;
                cli|gitfleet) SELECTED_CRATE=gitfleet ;;
                provider|providers|gitfleet-providers) SELECTED_CRATE=gitfleet-providers ;;
                *)
                    printf 'Error: unknown crate: %s\n' "$2" >&2
                    exit 2
                    ;;
            esac
            shift
            ;;
        --crate=*)
            crate_arg="${1#*=}"
            case "$crate_arg" in
                core|gitfleet-core) SELECTED_CRATE=gitfleet-core ;;
                cli|gitfleet) SELECTED_CRATE=gitfleet ;;
                provider|providers|gitfleet-providers) SELECTED_CRATE=gitfleet-providers ;;
                *)
                    printf 'Error: unknown crate: %s\n' "$crate_arg" >&2
                    exit 2
                    ;;
            esac
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            printf 'Error: unknown option: %s\n' "$1" >&2
            usage >&2
            exit 2
            ;;
    esac
    shift
done

count_loc() {
    local crate="$1"

    find "$ROOT_DIR/$crate" -type f -name '*.rs' -print0 |
        xargs -0 awk 'NF { count++ } END { print count + 0 }'
}

total=0

if [[ -n "$SELECTED_CRATE" ]]; then
    CRATES=("$SELECTED_CRATE")
fi

for crate in "${CRATES[@]}"; do
    loc=$(count_loc "$crate")
    total=$((total + loc))

done

printf '%d\n' "$total"
