use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let zig_dir = manifest_dir.join("../../zig/twoflow_parser");
    println!("cargo:rerun-if-changed={}", zig_dir.display());

    let status = Command::new("zig")
        .arg("build")
        .arg("-Doptimize=ReleaseFast")
        .current_dir(&zig_dir)
        .status()
        .expect("failed to execute `zig`; install zig 0.17.0-dev.305+bdfbf432d and ensure it is on PATH");
    assert!(status.success(), "zig parser build failed");

    let lib_dir = zig_dir.join("zig-out/lib");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=twoflow_zigparser");
}
