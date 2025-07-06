# Cursor Background Agent Environment
# Provides development environment for Toka project
# Code will be cloned by the agent - do NOT copy code here

FROM rust:1.86-slim

# Install development dependencies
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
    jq \
    procps \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create development user as recommended
RUN groupadd --gid 1000 ubuntu \
    && useradd --uid 1000 --gid ubuntu --shell /bin/bash --create-home ubuntu

# Install Rust development tools
RUN rustup component add clippy rustfmt rust-analyzer rust-src \
    && rustup target add x86_64-unknown-linux-gnu \
    && cargo install --locked cargo-auditable cargo-watch

# Set up user environment
USER ubuntu
WORKDIR /home/ubuntu

# Environment variables for development
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1
ENV RUSTFLAGS="-C target-cpu=native -C link-arg=-fuse-ld=lld"
ENV CARGO_NET_RETRY=10
ENV CARGO_NET_TIMEOUT=60

# Default command for background agent
CMD ["/bin/bash"]
