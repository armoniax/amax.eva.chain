#!/usr/bin/env bash
# This script is meant to be run on Unix/Linux based systems
set -e

echo "*** Initializing develop environment"

function install_rustup {
  echo "Installing Rust Toolchain..."
  if rustup --version &> /dev/null; then
    echo "Rust toolchain is already installed"
  else
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    source "$HOME"/.cargo/env
  fi
  rustup show
}

function install_cargo_binary {
  BIN_NAME=$1
  if cargo install --list | grep "$BIN_NAME" &> /dev/null; then
    echo "$BIN_NAME is already installed"
  else
    cargo install "$BIN_NAME"
  fi
}

install_rustup

install_cargo_binary "taplo-cli"
