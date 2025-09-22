default:
    just --list

@run:
    cargo run --release

@run-dev:
    cargo run

@build:
    cargo build --release

@build-dev:
    cargo build
