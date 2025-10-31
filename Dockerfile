# Multi-stage build for optimized production image
FROM rust:1.83-slim-bookworm AS builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build release binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/evo-server /usr/local/bin/evo-server

# Copy static files for frontend
COPY static /app/static

# Create directories for data persistence
RUN mkdir -p /app/checkpoints /app/data

# Set environment variables
ENV RUST_LOG=info

# Expose WebSocket port
EXPOSE 8080

# Volume for persistent data
VOLUME ["/app/checkpoints", "/app/data"]

# Run the server
CMD ["evo-server", "--config", "/app/data/config.json"]
