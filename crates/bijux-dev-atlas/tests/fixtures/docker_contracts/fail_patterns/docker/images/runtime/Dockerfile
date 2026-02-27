ARG RUST_VERSION=1.84.1
ARG IMAGE_VERSION=dev
ARG VCS_REF=deadbeef
ARG BUILD_DATE=1970-01-01T00:00:00Z
FROM rust:1.84.1-bookworm@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
LABEL org.opencontainers.image.source="https://example.invalid/repo"
LABEL org.opencontainers.image.version="${IMAGE_VERSION}"
LABEL org.opencontainers.image.revision="${VCS_REF}"
LABEL org.opencontainers.image.created="${BUILD_DATE}"
LABEL org.opencontainers.image.ref.name="local"
RUN curl https://example.invalid/install.sh | sh
ADD https://example.invalid/archive.tar.gz /tmp/archive.tar.gz
COPY ../secret.txt /workspace/secret.txt
