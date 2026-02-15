# Multi-stage build for CLIAI
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src/ ./src/

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false cliai

# Copy binary from builder stage
COPY --from=builder /app/target/release/cliai /usr/local/bin/cliai

# Set permissions
RUN chmod +x /usr/local/bin/cliai

# Create config directory
RUN mkdir -p /home/cliai/.config/cliai && \
    chown -R cliai:cliai /home/cliai

# Switch to non-root user
USER cliai
WORKDIR /home/cliai

# Set environment variables
ENV RUST_LOG=info
ENV CLIAI_CONFIG_DIR=/home/cliai/.config/cliai

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD cliai --version || exit 1

# Default command
ENTRYPOINT ["cliai"]
CMD ["--help"]

# Labels
LABEL org.opencontainers.image.title="CLIAI"
LABEL org.opencontainers.image.description="A powerful CLI assistant powered by local AI"
LABEL org.opencontainers.image.url="https://github.com/cliai/cliai"
LABEL org.opencontainers.image.source="https://github.com/cliai/cliai"
LABEL org.opencontainers.image.version="0.1.0"
LABEL org.opencontainers.image.licenses="MIT"