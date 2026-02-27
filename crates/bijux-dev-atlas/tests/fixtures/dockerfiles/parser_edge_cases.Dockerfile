# parser edge coverage
ARG RUST_VERSION=1.85.0
FROM --platform=linux/amd64 rust:${RUST_VERSION}@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa AS builder
ARG IMAGE_VERSION=dev
ARG VCS_REF=abc
ARG BUILD_DATE=2026-01-01T00:00:00Z
LABEL ORG.OPENCONTAINERS.IMAGE.SOURCE="https://example.invalid/repo" \
      org.opencontainers.image.version="${IMAGE_VERSION}" \
      org.opencontainers.image.revision="${VCS_REF}" \
      org.opencontainers.image.created="${BUILD_DATE}" \
      org.opencontainers.image.ref.name="runtime"
COPY --from=builder /workspace/bin/bijux-atlas /usr/local/bin/bijux-atlas
COPY ["Cargo.toml", "README.md", "/workspace/"]
RUN echo "ok"
