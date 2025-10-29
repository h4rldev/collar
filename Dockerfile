FROM rust:slim-trixie AS chef
RUN cargo install cargo-chef

WORKDIR /app

FROM chef AS planner

COPY . .

RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

WORKDIR /app

# hadolint ignore=DL3008
RUN apt-get update && apt-get install -y \
    just \
    libssl-dev \
    pkg-config \
    gcc \
    sqlite3 --no-install-recommends

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .

RUN cargo build --release

FROM debian:trixie-slim AS runtime

WORKDIR /app

COPY --from=builder /app/target/release/collar ./collar

ENTRYPOINT ["./collar"]
