# Multi-stage Dockerfile for Alttre SSH Ratatui Portfolio Server
FROM rust:1-bookworm AS builder

WORKDIR /app

# Cache dependencies first
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main() {}" > src/main.rs && echo "" > src/lib.rs
RUN cargo build --release && rm -rf src

# Build the real binary
COPY src ./src
RUN touch src/main.rs src/lib.rs && cargo build --release

# Minimal runtime image
FROM debian:bookworm-slim AS runtime

RUN useradd --create-home --shell /usr/sbin/nologin appuser \
    && mkdir -p /data \
    && chown appuser:appuser /data

COPY --from=builder /app/target/release/ssh-portfolio /usr/local/bin/ssh-portfolio

USER appuser
WORKDIR /data

ENV SSH_BIND=0.0.0.0
ENV SSH_PORT=2222
ENV SSH_HOST_KEY=/data/host_key
ENV MAX_SESSIONS=100
ENV IDLE_TIMEOUT_SECS=600
ENV RUST_LOG=info

EXPOSE 2222

VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/ssh-portfolio"]
