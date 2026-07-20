# Build stage - always latest stable Rust
FROM rust:latest AS builder

WORKDIR /app
COPY Cargo.toml ./
COPY src ./src

RUN cargo generate-lockfile 2>/dev/null || true
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/mediacleaner-pro ./mediacleaner-pro

EXPOSE 8080

ENTRYPOINT ["./mediacleaner-pro"]
