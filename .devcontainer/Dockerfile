FROM mcr.microsoft.com/devcontainers/base:ubuntu-24.04

# Install system dependencies
RUN apt-get update && apt-get install -y \
  build-essential \
  curl \
  wget

# Install rust-analyzer (latest release)
RUN curl -L https://github.com/rust-lang/rust-analyzer/releases/latest/download/rust-analyzer-$(uname -m)-unknown-linux-gnu.gz \
    | gunzip -c - > /usr/local/bin/rust-analyzer \
    && chmod +x /usr/local/bin/rust-analyzer

# Create user 'navila' with home directory and passwordless sudo
RUN useradd -m -s /bin/bash navila \
 && echo 'navila ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers

# Switch to non-root user
USER navila
WORKDIR /home/navila

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Add Cargo to PATH
ENV PATH="/home/navila/.cargo/bin:${PATH}"

# Finish Rust setup
RUN rustup toolchain install stable \
 && rustup target add wasm32-unknown-unknown
