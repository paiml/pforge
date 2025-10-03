# Multi-stage build for pforge
# Produces optimized, minimal Docker image

# Build stage
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# Build dependencies (cached layer)
RUN mkdir -p crates/pforge-cli/src && \
    echo "fn main() {}" > crates/pforge-cli/src/main.rs && \
    cargo build --release --bin pforge && \
    rm -rf crates/pforge-cli/src

# Copy source code
COPY crates/pforge-cli/src ./crates/pforge-cli/src

# Build application
RUN cargo build --release --bin pforge

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 pforge

# Copy binary from builder
COPY --from=builder /app/target/release/pforge /usr/local/bin/pforge

# Set user
USER pforge
WORKDIR /home/pforge

# Expose default port (if using HTTP transport)
EXPOSE 8080

# Default command
ENTRYPOINT ["pforge"]
CMD ["--help"]
