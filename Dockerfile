FROM rust:slim-trixie AS builder

RUN apt-get update && apt-get install -y \
    just \
    libssl-dev \
    pkg-config \
    gcc \
		file


# Set up workspace
WORKDIR /app

COPY src ./src/
COPY Cargo.toml .
COPY justfile .

RUN just build

FROM debian:trixie-slim

WORKDIR /app

COPY --from=builder /app/target/release/collar ./collar
RUN chmod +x ./collar
ENTRYPOINT ["./collar"]
