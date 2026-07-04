# Multi-stage build for ZUZ blockchain node
FROM rust:1.90-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update \
    && apt-get install --yes --no-install-recommends \
        ca-certificates \
        clang \
        libssl-dev \
        lld \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock ./

# Create src directory with dummy main
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (cached layer)
RUN cargo build --release && rm -rf src

# Copy source code
COPY src ./src
COPY core ./core
COPY network ./network
COPY wallet ./wallet
COPY extensions ./extensions
COPY runtime ./runtime

# Build the application
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:12-slim

# Install runtime dependencies
RUN apt-get update \
    && apt-get install --yes --no-install-recommends \
        ca-certificates \
        libgcc-s1 \
        libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd --system --gid 1001 zuz \
    && useradd --system --uid 1001 --gid zuz --create-home --home-dir /home/zuz --shell /usr/sbin/nologin zuz

# Create data directory
RUN mkdir -p /data /etc/zuz \
    && chown -R zuz:zuz /data /etc/zuz

# Copy the binary from builder stage
COPY --from=builder /app/target/release/zuz-node /usr/local/bin/zuz-node

# Copy any additional assets if needed

# Copy configuration
COPY docker/config/node.toml /etc/zuz/node.toml

# Set user
USER zuz

# Set working directory
WORKDIR /data

# Expose ports
EXPOSE 9944 30333

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD zuz-node --version || exit 1

# Set entrypoint
ENTRYPOINT ["zuz-node"]
CMD ["--config", "/etc/zuz/node.toml"]
