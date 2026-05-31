FROM rust:bookworm AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    clang \
    llvm \
    libudev-dev \
    && rm -rf /var/lib/apt/lists/*

RUN rustup toolchain install nightly && rustup default nightly

COPY Cargo.toml Cargo.lock ./
# Copy local .env into the image so dotenvy can load environment variables
# during development/testing. For production (Railway) prefer setting env vars
# via the platform instead of baking secrets into the image.
COPY .env ./
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
# Also copy .env into the runtime image so dotenvy can load it at runtime
COPY .env ./

EXPOSE 3000

CMD ["/app/backend-rust"]
