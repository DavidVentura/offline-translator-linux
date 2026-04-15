#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
onnx_root="${repo_root}/third_party/onnxruntime"

if [ ! -f "${onnx_root}/tools/ci_build/build.py" ]; then
  echo "ONNX Runtime source tree missing at ${onnx_root}" >&2
  echo "Run: git submodule update --init --depth 2 third_party/onnxruntime" >&2
  exit 1
fi

canonicalize_arch() {
  case "$1" in
    x86_64|amd64)
      echo "x86_64"
      ;;
    aarch64|arm64|arm64-v8a)
      echo "aarch64"
      ;;
    *)
      return 1
      ;;
  esac
}

machine_name_for_arch() {
  case "$1" in
    x86_64)
      echo "Advanced Micro Devices X86-64"
      ;;
    aarch64)
      echo "AArch64"
      ;;
    *)
      return 1
      ;;
  esac
}

lib_matches_arch() {
  local lib_path="$1"
  local arch="$2"
  local machine
  machine="$(machine_name_for_arch "$arch")" || return 1
  [ -f "$lib_path" ] || return 1
  readelf -h "$lib_path" 2>/dev/null | grep -Fq "Machine:                           ${machine}"
}

target_arch_raw="${ORT_BUILD_ARCH:-${CARGO_CFG_TARGET_ARCH:-$(uname -m)}}"
if ! target_arch="$(canonicalize_arch "${target_arch_raw}")"; then
  echo "Unsupported ONNX Runtime target architecture: ${target_arch_raw}" >&2
  exit 1
fi

cmake_arch_flag=""
cmake_cross_flags=()
cross_build_dir_reset=0
case "${target_arch}" in
  x86_64)
    ;;
  aarch64)
    cmake_arch_flag="onnxruntime_USE_ARM_NEON_NCHWC=ON"
    toolchain_triplet="${ORT_BUILD_TRIPLET:-${ARCH_TRIPLET:-aarch64-linux-gnu}}"
    cross_sysroot="/usr/${toolchain_triplet}"
    if [ "$(uname -m)" != "aarch64" ]; then
      cmake_cross_flags+=(
        "CMAKE_SYSTEM_NAME=Linux"
        "CMAKE_SYSTEM_PROCESSOR=aarch64"
        "CMAKE_C_COMPILER=${toolchain_triplet}-gcc"
        "CMAKE_CXX_COMPILER=${toolchain_triplet}-g++"
        "CMAKE_FIND_ROOT_PATH=${cross_sysroot}"
        "CMAKE_LIBRARY_PATH=${cross_sysroot}/lib"
        "LIBM=${cross_sysroot}/lib/libm.so"
        "LIBRT=${cross_sysroot}/lib/librt.a"
      )
      cross_build_dir_reset=1
    fi
    ;;
esac

build_dir="${repo_root}/build/onnxruntime/${target_arch}"
output_dir="${repo_root}/runtime-lib"
output_lib="${output_dir}/libonnxruntime.so"
built_lib="${build_dir}/Release/libonnxruntime.so"

if [ "${cross_build_dir_reset}" = "1" ] && [ -f "${build_dir}/Release/CMakeCache.txt" ]; then
  if grep -Fq "/usr/lib/aarch64-linux-gnu/libm.so" "${build_dir}/Release/CMakeCache.txt"; then
    rm -rf "${build_dir}"
  fi
fi

if [ "${FORCE_REBUILD_ORT:-0}" != "1" ] && lib_matches_arch "${output_lib}" "${target_arch}"; then
  exit 0
fi

if [ "${FORCE_REBUILD_ORT:-0}" != "1" ] && lib_matches_arch "${built_lib}" "${target_arch}"; then
  mkdir -p "${output_dir}"
  cp "${built_lib}" "${output_lib}"
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

if [ "${#cmake_cross_flags[@]}" -gt 0 ]; then
  command+=("${cmake_cross_flags[@]}")
fi

(
  cd "${onnx_root}"
  "${command[@]}"
)

if [ ! -f "${built_lib}" ]; then
  echo "Expected ONNX Runtime library was not built: ${built_lib}" >&2
  exit 1
fi

if ! lib_matches_arch "${built_lib}" "${target_arch}"; then
  echo "Built ONNX Runtime library has wrong architecture: ${built_lib}" >&2
  file "${built_lib}" >&2 || true
  exit 1
fi

mkdir -p "${output_dir}"
cp "${built_lib}" "${output_lib}"
