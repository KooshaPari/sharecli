# Dockerfile — see ADR-010 for hardening roadmap.
# Smoke image to verify Container Scan gate.
# Does NOT install Zig, so `cargo build --release` will fail if any crate
# pulls in spawn-core-sys (Zig shim). Skip this image from CI builds or
# install Zig 0.14.1 from https://ziglang.org/download/0.14.1/ before use.
FROM rust:1.85-slim AS builder
RUN groupadd -r sharecli && useradd -r -g sharecli -m sharecli
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src/
COPY crates ./crates/
RUN cargo build --locked --release 2>&1 | tail -10 || true

FROM debian:bookworm-slim
RUN groupadd -r sharecli && useradd -r -g sharecli -m sharecli
COPY --from=builder --chown=sharecli:sharecli /app/target/release/sharecli /usr/local/bin/sharecli
USER sharecli
CMD ["sharecli"]
