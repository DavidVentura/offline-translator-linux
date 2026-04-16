#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

cargo_version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml)"
click_version="$(sed -n 's/.*"version": "\(.*\)".*/\1/p' packaging/ubuntu-touch/manifest.json)"


if [[ "$cargo_version" == "$click_version" ]]; then
    exit 0
else
    printf 'Cargo:     %s\n' "$cargo_version"
    printf 'Clickable: %s\n' "$click_version"
    echo "MISMATCH"
    exit 1
fi
