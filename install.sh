#!/usr/bin/env bash
# Flash SDK installer — one command, like Flutter/RN setup.
# Usage: curl -fsSL https://raw.githubusercontent.com/Anu-Code07/flash/main/install.sh | bash
set -euo pipefail

FLASH_HOME="${FLASH_HOME:-$HOME/.flash}"
SDK_DIR="$FLASH_HOME/sdk"
REPO_URL="${FLASH_REPO:-https://github.com/Anu-Code07/flash.git}"
BRANCH="${FLASH_BRANCH:-main}"

bold() { printf '\033[1m%s\033[0m\n' "$*"; }
green() { printf '\033[0;32m✓\033[0m %s\n' "$*"; }
yellow() { printf '\033[0;33m!\033[0m %s\n' "$*"; }

bold "⚡ Flash SDK installer"

# ── Rust ─────────────────────────────────────────────────────────────────────
if ! command -v cargo >/dev/null 2>&1; then
  yellow "Rust not found — installing via rustup..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
  # shellcheck disable=SC1091
  source "${HOME}/.cargo/env"
fi
green "Rust $(rustc --version | awk '{print $2}')"

# ── SDK checkout ─────────────────────────────────────────────────────────────
mkdir -p "$FLASH_HOME"
if [[ -d "$SDK_DIR/.git" ]]; then
  bold "Updating Flash SDK at $SDK_DIR"
  git -C "$SDK_DIR" fetch origin "$BRANCH" --quiet
  git -C "$SDK_DIR" checkout "$BRANCH" --quiet 2>/dev/null || true
  git -C "$SDK_DIR" pull origin "$BRANCH" --ff-only --quiet || yellow "Could not fast-forward SDK (local changes?)"
else
  bold "Cloning Flash SDK → $SDK_DIR"
  git clone --depth 1 --branch "$BRANCH" "$REPO_URL" "$SDK_DIR"
fi
green "SDK ready"

# ── CLI ──────────────────────────────────────────────────────────────────────
bold "Installing flash CLI..."
cargo install --path "$SDK_DIR/cli" --force --quiet 2>/dev/null || cargo install --path "$SDK_DIR/cli" --force
green "flash CLI → $(command -v flash)"

# ── Shell profile ────────────────────────────────────────────────────────────
PROFILE="${HOME}/.zshrc"
[[ -f "${HOME}/.bashrc" ]] && PROFILE="${HOME}/.bashrc"

mark_begin="# >>> flash sdk >>>"
mark_end="# <<< flash sdk <<<"
if ! grep -q "$mark_begin" "$PROFILE" 2>/dev/null; then
  cat >>"$PROFILE" <<EOF

$mark_begin
export FLASH_SDK="$SDK_DIR"
export PATH="\$HOME/.cargo/bin:\$PATH"
$mark_end
EOF
  green "Added FLASH_SDK to $PROFILE"
else
  green "FLASH_SDK already in $PROFILE"
fi

export FLASH_SDK="$SDK_DIR"
export PATH="${HOME}/.cargo/bin:${PATH}"

bold ""
bold "Done! Run:"
echo "  source $PROFILE"
echo "  flash doctor"
echo "  flash create my_app"
echo "  cd my_app && flash dev"
bold ""

if command -v flash >/dev/null 2>&1; then
  flash doctor || true
fi
