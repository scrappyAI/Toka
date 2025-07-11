# Toka Development Environment Dockerfile
# Optimized for Cursor agents and collaborative development

FROM rust:1.86-slim-bullseye

# Set labels for better container management
LABEL name="toka-cursor-dev"
LABEL version="1.0.0"
LABEL description="Comprehensive Rust development environment for Toka OS"
LABEL maintainer="Toka Development Team"

# Set environment variables
ENV DEBIAN_FRONTEND=noninteractive \
    RUST_VERSION=1.86 \
    CARGO_HOME=/usr/local/cargo \
    RUSTUP_HOME=/usr/local/rustup \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_BACKTRACE=1 \
    CARGO_INCREMENTAL=1 \
    CARGO_NET_RETRY=10 \
    RUSTFLAGS="-D warnings" \
    TERM=xterm-256color

# Create non-root user for security
RUN groupadd --gid 1000 vscode \
    && useradd --uid 1000 --gid vscode --shell /bin/bash --create-home vscode

# Install system dependencies
RUN apt-get update && apt-get install -y \
    # Build essentials
    build-essential \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    # Database tools
    postgresql-client \
    sqlite3 \
    # Network tools
    curl \
    wget \
    # Version control
    git \
    # CLI utilities
    jq \
    ripgrep \
    fd-find \
    bat \
    exa \
    htop \
    tmux \
    vim \
    # Docker
    docker.io \
    docker-compose \
    # Development tools
    python3 \
    python3-pip \
    nodejs \
    npm \
    # Debugging and profiling
    valgrind \
    gdb \
    lldb \
    strace \
    ltrace \
    perf-tools-unstable \
    # Additional utilities
    less \
    tree \
    unzip \
    ca-certificates \
    gnupg \
    lsb-release \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Install Rust toolchain components
RUN rustup component add \
    clippy \
    rustfmt \
    rust-analyzer \
    rust-src \
    rust-docs

# Add additional targets
RUN rustup target add \
    x86_64-unknown-linux-gnu \
    aarch64-unknown-linux-gnu

# Install cargo tools
RUN cargo install \
    cargo-edit \
    cargo-watch \
    cargo-audit \
    cargo-deny \
    cargo-tarpaulin \
    cargo-fuzz \
    cargo-nextest \
    cargo-udeps \
    cargo-expand \
    cargo-tree \
    cargo-machete \
    cargo-outdated \
    cargo-bloat \
    cargo-deps \
    cargo-criterion \
    cargo-flamegraph \
    cargo-llvm-cov

# Install Python security tools
RUN pip3 install --no-cache-dir \
    semgrep \
    bandit \
    safety \
    black \
    flake8 \
    mypy

# Install Node.js tools for web development
RUN npm install -g \
    prettier \
    eslint \
    typescript \
    @types/node

# Install additional fuzzing tools
RUN apt-get update && apt-get install -y \
    afl++ \
    honggfuzz \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Install modern CLI tools
RUN curl -fsSL https://github.com/sharkdp/fd/releases/download/v8.7.0/fd_8.7.0_amd64.deb -o fd.deb \
    && dpkg -i fd.deb \
    && rm fd.deb

# Set up directories for development
RUN mkdir -p /home/vscode/data /home/vscode/logs \
    && chown -R vscode:vscode /home/vscode

# Copy essential container scripts only (not project code)
COPY .cursor/scripts/ /usr/local/bin/cursor-scripts/
RUN chmod +x /usr/local/bin/cursor-scripts/*

# Set up git configuration for the container
RUN git config --global init.defaultBranch main \
    && git config --global pull.rebase false \
    && git config --global safe.directory '*'

# Create cargo config for better caching
RUN mkdir -p /usr/local/cargo/config \
    && echo '[net]\nretry = 10\n[build]\njobs = 4\n[registry]\ndefault = "crates-io"\n[source.crates-io]\nreplace-with = "vendored-sources"\n[source.vendored-sources]\ndirectory = "vendor"' > /usr/local/cargo/config/config.toml

# Set up healthcheck
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD /usr/local/bin/cursor-scripts/cursor-health-check.sh || exit 1

# Switch to non-root user
USER vscode

# Set working directory to user home (best practice)
WORKDIR /home/vscode

# Set up user cargo config
RUN mkdir -p /home/vscode/.cargo \
    && echo '[net]\nretry = 10\n[build]\njobs = 4' > /home/vscode/.cargo/config.toml

# Expose ports for development services
EXPOSE 8080 9000 9001 3000

# Set default entrypoint
ENTRYPOINT ["/usr/local/bin/cursor-scripts/cursor-agent-init.sh"]

# Default command
CMD ["bash"] 