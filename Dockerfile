# syntax=docker/dockerfile:1.7
ARG RUST_VERSION=1.84.1
ARG DEBIAN_FRONTEND=noninteractive

FROM rust:${RUST_VERSION}-bookworm AS builder
WORKDIR /workspace

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates pkg-config libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock rust-toolchain.toml rustfmt.toml clippy.toml deny.toml ./
COPY crates ./crates

ENV CARGO_NET_GIT_FETCH_WITH_CLI=true \
    CARGO_TERM_COLOR=always

RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/workspace/target,sharing=locked \
    cargo build --locked --release -p bijux-atlas-server

FROM gcr.io/distroless/cc-debian12:nonroot AS runtime
WORKDIR /app
COPY --from=builder /workspace/target/release/atlas-server /app/atlas-server

ENV RUST_LOG=info \
    TZ=UTC

USER nonroot:nonroot
EXPOSE 8080
ENTRYPOINT ["/app/atlas-server"]
