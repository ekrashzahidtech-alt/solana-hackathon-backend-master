FROM rust:nightly-bookworm AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    clang \
    llvm \
    libudev-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
RUN cargo fetch

COPY src ./src
COPY migrations ./migrations

RUN cargo build --release --bin backend-rust

FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/backend-rust /app/backend-rust
COPY --from=builder /app/migrations /app/migrations

EXPOSE 3000

CMD ["/app/backend-rust"]
