# Containerfile for Podman development workflows
FROM rust:1.85-bullseye AS builder

WORKDIR /usr/src/realearth

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY engine ./engine
COPY tools ./tools
COPY apps ./apps
COPY benchmarks ./benchmarks
COPY tests ./tests
COPY assets ./assets
COPY data ./data

RUN cargo build --workspace --release

FROM debian:bullseye-slim

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/realearth/target/release/realearth-cli /usr/local/bin/
COPY --from=builder /usr/src/realearth/target/release/realearth-viewer /usr/local/bin/
COPY --from=builder /usr/src/realearth/assets /app/assets
COPY --from=builder /usr/src/realearth/data /app/data

ENV RUST_LOG=info

ENTRYPOINT ["realearth-cli"]
CMD ["--help"]
