FROM rust:1.85-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release
CMD ["./target/release/sharecli"]
