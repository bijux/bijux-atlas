ARG RUST_VERSION=1.84.1
ARG IMAGE_VERSION=dev
ARG VCS_REF=deadbeef
ARG BUILD_DATE=1970-01-01T00:00:00Z
ARG SOURCE_DATE_EPOCH=0
FROM rust:1.84.1-bookworm@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa AS builder
WORKDIR /workspace
RUN cargo build --locked
FROM gcr.io/distroless/cc-debian12:nonroot@sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb AS runtime
WORKDIR /app
USER nonroot:nonroot
LABEL org.opencontainers.image.source="https://example.invalid/repo"
LABEL org.opencontainers.image.version="${IMAGE_VERSION}"
LABEL org.opencontainers.image.revision="${VCS_REF}"
LABEL org.opencontainers.image.created="${BUILD_DATE}"
LABEL org.opencontainers.image.ref.name="local"
LABEL org.opencontainers.image.licenses="Apache-2.0"
COPY --from=builder /workspace/Cargo.toml /app/Cargo.toml
ENTRYPOINT ["/app/bijux-atlas", "version"]
