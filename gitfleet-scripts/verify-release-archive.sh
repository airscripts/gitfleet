#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 2 ]; then
    echo "usage: $0 <archive> <target>" >&2
    exit 2
fi

archive="$1"
target="$2"
workdir="$(mktemp -d)"

cleanup() {
    rm -rf "$workdir"
}

trap cleanup EXIT

case "$archive" in
    *.tar.gz)
        tar -xzf "$archive" -C "$workdir"
        ;;
    *.zip)
        if command -v unzip >/dev/null 2>&1; then
            unzip -q "$archive" -d "$workdir"
        else
            powershell -NoProfile -Command \
                "Expand-Archive -LiteralPath '$archive' -DestinationPath '$workdir'"
        fi
        ;;
    *)
        echo "unsupported archive format: $archive" >&2
        exit 1
        ;;
esac

case "$target" in
    *windows*)
        gitfleet_bin="gitfleet.exe"
        gf_bin="gf.exe"
        ;;
    *)
        gitfleet_bin="gitfleet"
        gf_bin="gf"
        ;;
esac

for required in "$gitfleet_bin" "$gf_bin" LICENSE; do
    if ! find "$workdir" -type f -name "$required" | grep -q .; then
        echo "archive $archive is missing $required" >&2
        exit 1
    fi
done
