#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
onnx_root="${repo_root}/third_party/onnxruntime"

if [ ! -f "${onnx_root}/tools/ci_build/build.py" ]; then
  echo "ONNX Runtime source tree missing at ${onnx_root}" >&2
  echo "Run: git submodule update --init --depth 2 third_party/onnxruntime" >&2
  exit 1
fi

host_arch="${ORT_BUILD_ARCH:-$(uname -m)}"
case "${host_arch}" in
  x86_64|amd64)
    cmake_arch_flag=""
    ;;
  aarch64|arm64)
    cmake_arch_flag="onnxruntime_USE_ARM_NEON_NCHWC=ON"
    ;;
  *)
    echo "Unsupported host architecture for ONNX Runtime build: ${host_arch}" >&2
    exit 1
    ;;
esac

build_dir="${repo_root}/build/onnxruntime/${host_arch}"
output_dir="${repo_root}/runtime-lib"
output_lib="${output_dir}/libonnxruntime.so"
target_dir="${repo_root}/target"

if [ "${FORCE_REBUILD_ORT:-0}" != "1" ] && [ -f "${output_lib}" ]; then
  exit 0
fi

path_remap_flags="-ffile-prefix-map=${repo_root}=. -fdebug-prefix-map=${repo_root}=."

command=(
  python3
  tools/ci_build/build.py
  "--build_dir=${build_dir}"
  "--config=Release"
  "--update"
  "--build"
  "--targets"
  "onnxruntime"
  "--skip_tests"
  "--parallel"
  "--build_shared_lib"
  "--disable_ml_ops"
  "--disable_generation_ops"
  "--no_kleidiai"
  "--use_xnnpack"
  "--no_sve"
  "--skip_submodule_sync"
  "--cmake_extra_defines"
  "CMAKE_CXX_STANDARD=20"
  "CMAKE_CXX_STANDARD_REQUIRED=ON"
  "CMAKE_CXX_EXTENSIONS=OFF"
  "CMAKE_C_FLAGS=${path_remap_flags}"
  "CMAKE_CXX_FLAGS=${path_remap_flags}"
)

if [ -n "${cmake_arch_flag}" ]; then
  command+=("${cmake_arch_flag}")
fi

(
  cd "${onnx_root}"
  "${command[@]}"
)

built_lib="${build_dir}/Release/libonnxruntime.so"
if [ ! -f "${built_lib}" ]; then
  echo "Expected ONNX Runtime library was not built: ${built_lib}" >&2
  exit 1
fi

mkdir -p "${output_dir}" "${target_dir}/debug" "${target_dir}/release"
cp "${built_lib}" "${output_lib}"
cp "${built_lib}" "${target_dir}/debug/libonnxruntime.so"
cp "${built_lib}" "${target_dir}/release/libonnxruntime.so"
