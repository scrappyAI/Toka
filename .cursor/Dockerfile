# Development environment for Toka agents with comprehensive debugging tools
FROM rust:1.86-slim AS development


# Install comprehensive system dependencies and development tools
RUN apt-get update && apt-get install -y --no-install-recommends \
    # Core build tools
    build-essential \
    pkg-config \
    libssl-dev \
    ca-certificates \
    git \
    curl \
    wget \
    # Debugging and profiling tools
    gdb \
    valgrind \
    strace \
    ltrace \
    perf-tools-unstable \
    linux-perf \
    # System monitoring and analysis
    htop \
    iotop \
    lsof \
    netstat-nat \
    tcpdump \
    # Development utilities
    vim \
    nano \
    tmux \
    screen \
    jq \
    tree \
    # Network and protocol tools
    netcat-openbsd \
    telnet \
    dnsutils \
    iputils-ping \
    # Database tools
    sqlite3 \
    # Container and orchestration tools
    docker.io \
    docker-compose \
    # Additional libraries for agent development
    libpq-dev \
    libsqlite3-dev \
    libc6-dev \
    libffi-dev \
    # Memory debugging
    libc6-dbg \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create development user with proper permissions
RUN groupadd --gid 1000 vscode \
    && useradd --uid 1000 --gid vscode --shell /bin/bash --create-home vscode \
    && usermod -aG docker vscode \
    && mkdir -p /home/vscode/data /home/vscode/logs /home/vscode/config \
    && chown -R vscode:vscode /home/vscode

# Install Rust development tools and components
RUN rustup component add \
    clippy \
    rustfmt \
    rust-src \
    rust-analyzer \
    && rustup target add x86_64-unknown-linux-gnu \
    && rustup toolchain install nightly \
    && rustup component add --toolchain nightly miri

# Install essential Cargo tools for debugging and profiling
RUN cargo install --locked \
    cargo-watch \
    cargo-edit \
    cargo-expand \
    cargo-audit \
    cargo-outdated \
    cargo-tree \
    flamegraph \
    cargo-pgo \
    cargo-llvm-cov \
    cargo-nextest \
    cargo-udeps \
    cargo-deny \
    cargo-machete \
    sccache

# Install additional debugging and analysis tools
RUN cargo install --locked \
    cargo-asm \
    cargo-bloat \
    cargo-geiger \
    cargo-modules \
    cargo-deps \
    tokio-console

# Set up development environment - use user's home directory
WORKDIR /home/vscode
USER vscode

# Environment variables optimized for development
ENV RUST_LOG=debug
ENV RUST_BACKTRACE=full
ENV CARGO_INCREMENTAL=1
ENV CARGO_TARGET_DIR=/home/vscode/target
ENV SCCACHE_DIR=/home/vscode/.sccache
ENV RUSTFLAGS="-C debuginfo=2 -C opt-level=0"

# Development-specific agent environment
ENV AGENT_DEV_MODE=true
ENV AGENT_DEBUG_ENABLED=true
ENV AGENT_PROFILING_ENABLED=true
ENV AGENT_DATA_DIR=/home/vscode/data
ENV AGENT_LOG_DIR=/home/vscode/logs
ENV AGENT_CONFIG_DIR=/home/vscode/config

# Performance monitoring for development
ENV TOKIO_CONSOLE_BIND=0.0.0.0:6669
ENV AGENT_METRICS_DETAILED=true
ENV AGENT_TRACE_LEVEL=trace

# Security settings for development (more permissive)
ENV AGENT_SANDBOX_DEV=true
ENV CAPABILITY_VALIDATION=moderate

# Create necessary directories with proper permissions
RUN mkdir -p \
    /home/vscode/data \
    /home/vscode/logs \
    /home/vscode/config \
    /home/vscode/target \
    /home/vscode/.sccache \
    /home/vscode/.cargo \
    && chown -R vscode:vscode /home/vscode

# Default command for development
CMD ["bash"] 