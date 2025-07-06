# Cursor-optimized Dockerfile for Toka Background Agents
# Multi-stage build optimized for cursor's background agent containers

# Stage 1: Build environment (optimized for cursor agent compilation)
FROM rust:1.86-slim AS cursor-builder

# Install build dependencies with cursor-specific optimizations
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    ca-certificates \
    git \
    curl \
    lld \
    clang \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create cursor-specific build user
RUN groupadd --gid 10001 cursor \
    && useradd --uid 10001 --gid cursor --shell /bin/bash --create-home cursor

# Rust environment optimized for cursor agents
ENV RUSTFLAGS="-C target-cpu=native -C link-arg=-fuse-ld=lld -C prefer-dynamic"
ENV CARGO_NET_RETRY=10
ENV CARGO_NET_TIMEOUT=60
ENV CARGO_INCREMENTAL=0
ENV CARGO_TARGET_DIR=/tmp/target

# Install cursor-specific Rust components
RUN rustup component add clippy rustfmt rust-analyzer rust-src \
    && rustup target add x86_64-unknown-linux-gnu \
    && cargo install --locked cargo-auditable cargo-watch

# Copy source with cursor-specific optimizations
WORKDIR /build
COPY --chown=cursor:cursor . .

# Build with cursor agent optimizations
USER cursor
RUN cargo auditable build --release --locked \
    --bin toka-cli \
    --bin toka-config-cli \
    --features cursor-agents \
    && cargo audit \
    && strip target/release/toka-cli \
    && strip target/release/toka-config-cli

# Stage 2: Cursor agent runtime environment
FROM debian:bookworm-slim AS cursor-runtime

# Install cursor agent runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl \
    jq \
    tini \
    procps \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create cursor agent runtime user
RUN groupadd --gid 10001 toka \
    && useradd --uid 10001 --gid toka --shell /bin/bash --create-home toka \
    && mkdir -p /app/bin /app/data /app/logs /app/config /app/scripts \
    && mkdir -p /app/data/locks /app/data/cache /app/data/sessions /app/data/context \
    && chown -R toka:toka /app

# Copy cursor-optimized binaries
COPY --from=cursor-builder --chown=toka:toka /build/target/release/toka-cli /app/bin/
COPY --from=cursor-builder --chown=toka:toka /build/target/release/toka-config-cli /app/bin/

# Copy cursor-specific configurations and scripts
COPY --chown=toka:toka config/cursor-agents.toml /app/config/
COPY --chown=toka:toka .cursor/scripts/ /app/scripts/

# Set execute permissions for scripts
RUN chmod +x /app/scripts/*.sh

# Set up cursor agent environment
USER toka
WORKDIR /app

# Cursor agent environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1
ENV TOKIO_WORKER_THREADS=4
ENV CURSOR_AGENT_MODE=true
ENV CURSOR_AGENT_DATA_DIR=/app/data
ENV CURSOR_AGENT_LOG_DIR=/app/logs
ENV CURSOR_AGENT_CONFIG_DIR=/app/config

# Cursor-specific security settings
ENV RUST_SECURE_ZERO_MEMORY=1
ENV CURSOR_SANDBOX_ENABLED=true
ENV CURSOR_CAPABILITY_VALIDATION=strict

# Ownership management settings
ENV CURSOR_SINGLE_OWNER_MODE=true
ENV CURSOR_OWNERSHIP_VALIDATION=strict
ENV CURSOR_OWNERSHIP_TIMEOUT=300
ENV CURSOR_CONFLICT_RESOLUTION=terminate_conflicting

# Cursor agent performance optimizations
ENV CURSOR_AGENT_POOL_SIZE=12
ENV CURSOR_AGENT_SPAWN_TIMEOUT=30
ENV CURSOR_WORKSTREAM_TIMEOUT=3600
ENV CURSOR_CONTEXT_CACHE_SIZE=2048
ENV CURSOR_CONTEXT_CACHE_TTL=7200

# Cursor agent communication settings
ENV CURSOR_WEBSOCKET_ENABLED=true
ENV CURSOR_WEBSOCKET_PORT=9001
ENV CURSOR_API_TIMEOUT=30
ENV CURSOR_MAX_CONCURRENT_REQUESTS=50

# Health check optimized for cursor agents with ownership validation
HEALTHCHECK --interval=15s --timeout=5s --start-period=10s --retries=3 \
    CMD /app/scripts/cursor-health-check.sh --quick || exit 1

# Expose cursor agent ports
EXPOSE 8080 9000 9001 3000

# Use tini for proper signal handling in cursor environment
ENTRYPOINT ["/usr/bin/tini", "--"]

# Default command uses the initialization script for proper ownership management
CMD ["/app/scripts/cursor-agent-init.sh"]
