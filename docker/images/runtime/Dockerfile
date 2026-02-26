# syntax=docker/dockerfile:1.7
ARG RUST_VERSION=1.84.1
ARG DEBIAN_FRONTEND=noninteractive
ARG IMAGE_VERSION=dev-local
ARG VCS_REF=unknown
ARG BUILD_DATE=1970-01-01T00:00:00Z
ARG IMAGE_PROVENANCE=local

FROM rust:${RUST_VERSION}-bookworm AS builder
WORKDIR /workspace

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates pkg-config libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY configs/rust/rustfmt.toml /workspace/configs/rust/rustfmt.toml
COPY configs/rust/clippy.toml /workspace/configs/rust/clippy.toml
COPY configs/security/deny.toml /workspace/configs/security/deny.toml
COPY crates ./crates

ENV CARGO_NET_GIT_FETCH_WITH_CLI=true \
    CARGO_TERM_COLOR=always \
    CARGO_TARGET_DIR=/workspace/artifacts/target

RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    cargo build --locked --release -p bijux-atlas-server -p bijux-atlas-cli

FROM gcr.io/distroless/cc-debian12:nonroot AS runtime
WORKDIR /app
COPY --from=builder /workspace/artifacts/target/release/atlas-server /app/atlas-server
COPY --from=builder /workspace/artifacts/target/release/bijux-atlas /app/bijux-atlas

ENV RUST_LOG=info \
    TZ=UTC

USER nonroot:nonroot
EXPOSE 8080
LABEL org.bijux.plugin="atlas"
LABEL org.opencontainers.image.source="https://github.com/bijux/bijux-atlas"
LABEL org.opencontainers.image.version="${IMAGE_VERSION}"
LABEL org.opencontainers.image.revision="${VCS_REF}"
LABEL org.opencontainers.image.created="${BUILD_DATE}"
LABEL org.opencontainers.image.ref.name="${IMAGE_PROVENANCE}"
ENTRYPOINT ["/app/bijux-atlas", "atlas", "serve"]
