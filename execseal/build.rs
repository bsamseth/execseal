use std::process::Command;

fn main() {
    let envs = std::env::vars()
        .filter(|(key, _)| key == "CARGO_HOME" || key == "RUSTUP_HOME" || key == "PATH");
    build_runtime(envs);

    println!("cargo:rerun-if-changed=../execseal-runtime");
    println!("cargo:rerun-if-changed=../execseal-common");
}

#[cfg(not(feature = "nightly"))]
fn build_runtime(env_vars: impl Iterator<Item = (String, String)>) {
    let target = std::env::var("TARGET").expect("cargo should set TARGET environment");
    let target_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../target/runtime");
    let runtime_binary = format!("{target_dir}/x86_64-unknown-linux-gnu/release/execseal-runtime");

    let status = Command::new("cargo")
        .args([
            "build",
            "--bin=execseal-runtime",
            "--package=execseal-runtime",
            "--profile=release",
            &format!("--target={target}"),
        ])
        .env_clear()
        .envs(env_vars)
        .env("CARGO_TARGET_DIR", target_dir)
        .status()
        .expect("Failed to build execseal-runtime");
    assert!(status.success(), "Building execseal-runtime failed");

    #[cfg(feature = "upx")]
    compress_with_upx(&runtime_binary);

    println!("cargo:rustc-env=EXECSEAL_RUNTIME={runtime_binary}");
}

#[cfg(feature = "nightly")]
fn build_runtime(env_vars: impl Iterator<Item = (String, String)>) {
    let target = std::env::var("TARGET").expect("cargo should set TARGET environment");
    let target_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../target/runtime");
    let runtime_binary = format!("{target_dir}/{target}/release-nightly/execseal-runtime");

    let status = Command::new("cargo")
        .args([
            "+nightly",
            "build",
            "--bin=execseal-runtime",
            "--package=execseal-runtime",
            "--profile=release-nightly",
            &format!("--target={target}"),
            "-Zbuild-std",
            "-Zbuild-std-features=optimize_for_size",
        ])
        .env_clear()
        .envs(env_vars)
        .env(
            "RUSTFLAGS",
            "-Zlocation-detail=none -Zfmt-debug=none -Zunstable-options -Cpanic=immediate-abort",
        )
        .env("CARGO_TARGET_DIR", target_dir)
        .status()
        .expect("Failed to build execseal-runtime");
    assert!(status.success(), "Building execseal-runtime failed");

    #[cfg(feature = "upx")]
    compress_with_upx(&runtime_binary);

    println!("cargo:rustc-env=EXECSEAL_RUNTIME={runtime_binary}");
}

#[cfg(feature = "upx")]
fn compress_with_upx(binary: &str) {
    let r = Command::new("upx").arg("--version").status();
    assert!(
        r.is_ok_and(|s| s.success()),
        "The `upx` feature is enabled, but `upx` could not be executed. Ensure it is installed and in PATH. On debian: `apt install upx-ucl`"
    );
    let status = Command::new("upx")
        .args(["--best", "--lzma", binary])
        .status()
        .expect("Failed to compress with UPX");
    assert!(status.success(), "Compressing with UPX failed");
}
