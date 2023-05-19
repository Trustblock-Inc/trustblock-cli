#!/bin/bash

# Check if rust is installed
if ! command -v rustc &>/dev/null; then
    echo "Rust is not installed. Installing..."

    # Download and install rustup (which includes rustc, cargo, etc)
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

    # Add cargo to PATH in the current shell
    source $HOME/.cargo/env
else
    echo "Rust is already installed."
fi

# Check if trustblock-cli is installed
if ! cargo install --list | grep -q 'trustblock-cli v'; then
    echo "trustblock is not installed. Installing..."
    cargo install trustblock-cli
    echo "Run: trustblock help"

else
    echo "trustblock-cli is already installed."
fi
