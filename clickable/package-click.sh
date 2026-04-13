#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

clickable_bin="${CLICKABLE_BIN:-}"
if [ -z "$clickable_bin" ]; then
  if [ -x "../venv/bin/clickable" ]; then
    clickable_bin="../venv/bin/clickable"
  else
    clickable_bin="clickable"
  fi
fi

if [ "${SKIP_BINARY_BUILD:-0}" != "1" ]; then
  cargo build --manifest-path ../Cargo.toml --release
  cp ../target/release/offline-translator-linux translator
  strip translator || true
fi

exec "$clickable_bin" build "$@"
