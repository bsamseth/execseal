# Show this list of available recipes.
default:
    @just --list

# Showcase an example.
showcase:
    gcc protectme.c -o protectme
    cargo run --bin execseal -- --password hello --output protected protectme
    ls -lah protected protectme
    env -i EXECSEALPASS=hello FOO=bar ./protected one two three
