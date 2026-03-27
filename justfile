# Show this list of available recipes.
default:
    @just --list

# Run the proof-of-concept
showcase:
    gcc dummy.c -o protectme
    cargo build --bin execseal-runtime
    cargo run --bin execseal -- --password hello --output protected protectme 
    EXECSEALPASS=hello ./protected </dev/null 2>/dev/null
