FROM rust:1.86-slim

# Install build essentials and development tools
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
       build-essential \
       pkg-config \
       libssl-dev \
       ca-certificates \
       git \
       curl \
       wget \
       vim \
       nano \
       lld \
       clang \
       lldb \
       gdb \
       valgrind \
       strace \
       htop \
       jq \
       tree \
       fd-find \
       ripgrep \
       bat \
       exa \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Install Rust components and tools
RUN rustup component add clippy rustfmt rust-analyzer rust-src \
    && rustup target add x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu \
    && cargo install cargo-edit cargo-watch cargo-expand cargo-audit cargo-outdated cargo-tree \
    && cargo install --locked cargo-deny cargo-nextest

# Create non-root user
RUN groupadd --gid 1000 vscode \
    && useradd --uid 1000 --gid vscode --shell /bin/bash --create-home vscode \
    && mkdir -p /home/vscode/.cargo \
    && chown -R vscode:vscode /home/vscode

# Set up environment for vscode user
USER vscode
ENV PATH="/home/vscode/.cargo/bin:${PATH}"
ENV RUST_BACKTRACE=1
ENV CARGO_TARGET_DIR=/tmp/target

WORKDIR /workspace 