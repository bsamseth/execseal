use std::process::Command;

fn main() {
    // Build execseal-runtime with custom minimizing options, so that it can be embedded.
    let envs = std::env::vars()
        .filter(|(key, _)| key == "CARGO_HOME" || key == "RUSTUP_HOME" || key == "PATH");
    let target_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../target/nightly");
    let runtime_binary =
        format!("{target_dir}/x86_64-unknown-linux-gnu/release-nightly/execseal-runtime");

    let status = Command::new("cargo")
        .args([
            "+nightly",
            "build",
            "--bin=execseal-runtime",
            "--package=execseal-runtime",
            "--profile=release-nightly",
            "--target=x86_64-unknown-linux-gnu",
            "-Zbuild-std",
            "-Zbuild-std-features=optimize_for_size",
        ])
        .env_clear()
        .envs(envs)
        .env(
            "RUSTFLAGS",
            "-Zlocation-detail=none -Zfmt-debug=none -Zunstable-options -Cpanic=immediate-abort",
        )
        .env("CARGO_TARGET_DIR", target_dir)
        .status()
        .expect("Failed to build execseal-runtime");
    assert!(status.success(), "Building execseal-runtime failed");

    let status = Command::new("upx")
        .args(["--best", "--lzma", &runtime_binary])
        .status()
        .expect("Failed to compress with UPX");
    assert!(status.success(), "Compressing with UPX failed");

    println!("cargo:rerun-if-changed=../execseal-runtime");
    println!("cargo:rerun-if-changed=../execseal-common");
    println!("cargo:rustc-env=EXECSEAL_RUNTIME={runtime_binary}");
}
