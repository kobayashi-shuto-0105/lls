# ------------------------------
# Stage 1. Build an app
# ------------------------------
FROM rust:1.96.0 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# ------------------------------
# Stage 2. Build for runtime
# ------------------------------
FROM dhi.io/debian-base:trixie
ARG GIT_REVISION
ARG BUILD_DATE
ARG VERSION
LABEL org.opencontainers.image.title="lls" \
      org.opencontainers.image.description="LLS command line application written in Rust" \
      org.opencontainers.image.url="https://kobayashi-shuto-0105.github.io/lls" \
      org.opencontainers.image.source="https://github.com/kobayashi-shuto-0105/lls" \
      org.opencontainers.image.version=${VERSION} \
      org.opencontainers.image.revision=${GIT_REVISION} \
      org.opencontainers.image.created=${BUILD_DATE} \
      org.opencontainers.image.licenses="MIT"

COPY --from=builder /app/target/release/lls /app/lls
WORKDIR /opt
ENTRYPOINT [ "/app/lls" ]
