# Dockerfile placeholder — see ADR-010 for hardening roadmap.
# Currently a minimal smoke image to verify Container Scan gate.
# Does NOT install Zig, so `cargo build --release` will fail if any crate
# pulls in spawn-core-sys (Zig shim). Skip this image from CI builds or
# install Zig 0.14.1 from https://ziglang.org/download/0.14.1/ before use.
FROM rust:1.85-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --locked --release 2>&1 | tail -10 || true
CMD ["./target/release/sharecli"]
