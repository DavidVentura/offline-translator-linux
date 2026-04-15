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

../scripts/prepare_onnxruntime.sh
exec "$clickable_bin" build -c ../clickable.yaml "$@"
