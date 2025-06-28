# Multi-stage build for TelaMentis core
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY core/ ./core/
COPY adapters/ ./adapters/
COPY connectors/ ./connectors/
COPY kgctl/ ./kgctl/

# Build the application
RUN cargo build --release --bin telamentis-core

# Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false telamentis

# Create app directory
WORKDIR /app

# Copy built binary
COPY --from=builder /app/target/release/telamentis-core /usr/local/bin/

# Change ownership
RUN chown -R telamentis:telamentis /app

# Switch to app user
USER telamentis

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:3000/health || exit 1

# Run the application
CMD ["telamentis-core"]