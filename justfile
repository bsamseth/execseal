# Show this list of available recipes.
default:
    @just --list

# Test with debug builds
test:
    gcc dummy.c -o protectme
    cargo build --bin execseal-runtime
    ln -sf target/debug/execseal-runtime rt
    cargo run --bin execseal -- --password hello --output protected protectme 
    EXECSEALPASS=hello ./protected


# Test with release builds
test-release: build-release-runtime
    gcc dummy.c -o protectme
    cargo run --release --bin execseal -- --password hello --output protected protectme
    ls -la protected protectme
    EXECSEALPASS=hello ./protected

# Build a minimized runtime.
build-release-runtime target="x86_64-unknown-linux-gnu":
    RUSTFLAGS="-Zlocation-detail=none -Zfmt-debug=none -Zunstable-options -Cpanic=immediate-abort" \
        cargo +nightly build \
        --bin execseal-runtime \
        --release \
        --target {{target}} \
        -Z build-std \
        -Z build-std-features="optimize_for_size"
    ln -sf target/{{target}}/release/execseal-runtime rt
    upx --best --lzma $(readlink -f rt)

