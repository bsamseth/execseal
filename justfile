# Show this list of available recipes.
default:
    @just --list

# Test with debug builds
test:
    gcc protectme.c -o protectme
    cargo build --bin execseal-runtime
    ln -sf target/debug/execseal-runtime rt
    cargo run --bin execseal -- --password hello --output protected protectme 
    env -i EXECSEALPASS=hello FOO=bar ./protected one two three


# Test with release builds
test-release: build-release-runtime
    gcc protectme.c -o protectme
    cargo run --release --bin execseal -- --password letmein --output protected protectme
    ls -la protected protectme
    env -i EXECSEALPASS=letmein FOO=bar ./protected one two three

# Build a minimized runtime. https://github.com/johnthagen/min-sized-rust
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

