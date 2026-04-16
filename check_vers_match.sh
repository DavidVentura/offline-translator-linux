#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

cargo_version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml)"
click_version="$(sed -n 's/.*"version": "\(.*\)".*/\1/p' packaging/ubuntu-touch/manifest.json)"

printf 'Cargo:     %s\n' "$cargo_version"
printf 'Clickable: %s\n' "$click_version"

if [[ "$cargo_version" == "$click_version" ]]; then
    echo "OK"
else
    echo "MISMATCH"
    exit 1
fi
