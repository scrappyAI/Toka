FROM rust:1.78-slim

# Install build essentials and common tools
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
       build-essential pkg-config libssl-dev ca-certificates git curl \
    && rustup component add clippy rustfmt \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace 