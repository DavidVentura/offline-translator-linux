use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=scripts/prepare_onnxruntime.sh");
    println!("cargo:rerun-if-changed=.gitmodules");
    println!("cargo:rerun-if-env-changed=FORCE_REBUILD_ORT");
    println!("cargo:rerun-if-env-changed=ORT_BUILD_ARCH");

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let runtime_lib = manifest_dir.join("runtime-lib/libonnxruntime.so");
    if runtime_lib.is_file() {
        return;
    }

    let status = Command::new("bash")
        .arg("scripts/prepare_onnxruntime.sh")
        .current_dir(&manifest_dir)
        .status()
        .expect("failed to launch ONNX Runtime build helper");

    if !status.success() {
        panic!("ONNX Runtime build helper failed with status {status}");
    }
}
