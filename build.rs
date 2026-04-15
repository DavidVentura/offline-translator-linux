use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=scripts/prepare_onnxruntime.sh");
    println!("cargo:rerun-if-changed=.gitmodules");
    println!("cargo:rerun-if-env-changed=FORCE_REBUILD_ORT");
    println!("cargo:rerun-if-env-changed=ORT_BUILD_ARCH");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_ARCH");
    println!("cargo:rerun-if-env-changed=ARCH_TRIPLET");

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").expect("target arch");

    let mut command = Command::new("bash");
    command
        .arg("scripts/prepare_onnxruntime.sh")
        .current_dir(&manifest_dir);
    command.env("ORT_BUILD_ARCH", target_arch);
    if let Ok(triplet) = std::env::var("ARCH_TRIPLET") {
        command.env("ORT_BUILD_TRIPLET", triplet);
    }

    let status = command
        .status()
        .expect("failed to launch ONNX Runtime build helper");

    if !status.success() {
        panic!("ONNX Runtime build helper failed with status {status}");
    }
}
