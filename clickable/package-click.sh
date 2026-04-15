#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"

cd "$repo_root"

clickable_bin="${CLICKABLE_BIN:-}"
if [ -z "$clickable_bin" ]; then
  if [ -x "$repo_root/venv/bin/clickable" ]; then
    clickable_bin="$repo_root/venv/bin/clickable"
  else
    clickable_bin="clickable"
  fi
fi

translator_host_path="/home/david/AndroidStudioProjects/Translator/translator"
if [ -d "$translator_host_path" ]; then
  extra_mount="${translator_host_path}:${translator_host_path}:ro"
  if [ -n "${CLICKABLE_EXTRA_MOUNTS:-}" ]; then
    export CLICKABLE_EXTRA_MOUNTS="${CLICKABLE_EXTRA_MOUNTS},${extra_mount}"
  else
    export CLICKABLE_EXTRA_MOUNTS="${extra_mount}"
  fi
fi

ort_build_arch="${ORT_BUILD_ARCH:-}"
if [ -z "$ort_build_arch" ]; then
  args=("$@")
  for ((i=0; i<${#args[@]}; i++)); do
    if [ "${args[$i]}" = "--arch" ] && [ $((i + 1)) -lt ${#args[@]} ]; then
      case "${args[$((i + 1))]}" in
        arm64) ort_build_arch="aarch64" ;;
        amd64) ort_build_arch="x86_64" ;;
      esac
    fi
  done
fi

if [ -n "$ort_build_arch" ]; then
  ort_build_triplet="${ORT_BUILD_TRIPLET:-}"
  if [ -z "$ort_build_triplet" ] && [ "$ort_build_arch" = "aarch64" ]; then
    ort_build_triplet="aarch64-linux-gnu"
  fi
  ORT_BUILD_ARCH="$ort_build_arch" ORT_BUILD_TRIPLET="$ort_build_triplet" "$repo_root/scripts/prepare_onnxruntime.sh"
else
  "$repo_root/scripts/prepare_onnxruntime.sh"
fi
exec "$clickable_bin" build -c clickable.yaml "$@"
