# Dockerfile — see ADR-010 for hardening roadmap.
# Smoke image to verify Container Scan gate.
# Uses .dockerignore to exclude sensitive data (addresses SonarCloud CRITICAL).
# Does NOT install Zig, so `cargo build --release` will fail if any crate
# pulls in spawn-core-sys (Zig shim).
FROM rust:1.85-slim AS builder
RUN groupadd -r sharecli && useradd -r -g sharecli -m sharecli
WORKDIR /app
COPY . .
RUN cargo build --locked --release 2>&1 | tail -10 || true

FROM debian:bookworm-slim
RUN groupadd -r sharecli && useradd -r -g sharecli -m sharecli
COPY --from=builder --chown=sharecli:sharecli /app/target/release/sharecli /usr/local/bin/sharecli
USER sharecli
CMD ["sharecli"]
