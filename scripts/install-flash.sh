#!/usr/bin/env bash
# Install the `flash` CLI globally from this repo.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if ! command -v cargo >/dev/null 2>&1; then
  echo "Error: Rust/cargo not found. Install from https://rustup.rs"
  exit 1
fi

echo "Building flash-cli (release)..."
cargo build --release -p flash-cli

echo "Installing flash to ~/.cargo/bin ..."
cargo install --path cli --force

if command -v flash >/dev/null 2>&1; then
  echo ""
  echo "✓ flash installed: $(which flash)"
  flash help
else
  echo ""
  echo "Add ~/.cargo/bin to your PATH, then run: flash help"
  echo '  echo export PATH="\$HOME/.cargo/bin:\$PATH" >> ~/.zshrc'
fi
