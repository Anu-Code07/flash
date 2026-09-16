#!/usr/bin/env bash
# Flash SDK installer — full dev setup in one command.
#
#   curl -fsSL https://raw.githubusercontent.com/Anu-Code07/flash/main/install.sh | bash
#
# Options (env vars):
#   FLASH_QUICKSTART=1   Create demo app `my_app` after install
#   FLASH_REPO=...       Override git URL
#   FLASH_BRANCH=main    Branch to install
set -euo pipefail

FLASH_HOME="${FLASH_HOME:-$HOME/.flash}"
SDK_DIR="$FLASH_HOME/sdk"
REPO_URL="${FLASH_REPO:-https://github.com/Anu-Code07/flash.git}"
BRANCH="${FLASH_BRANCH:-main}"

# ── UI helpers ───────────────────────────────────────────────────────────────
bold()  { printf '\033[1m%s\033[0m\n' "$*"; }
dim()   { printf '\033[2m%s\033[0m\n' "$*"; }
green() { printf '\033[0;32m  ✓\033[0m %s\n' "$*"; }
cyan()  { printf '\033[36m  →\033[0m %s\n' "$*"; }
yellow(){ printf '\033[0;33m  !\033[0m %s\n' "$*"; }
red()   { printf '\033[0;31m  ✗\033[0m %s\n' "$*"; }

detect_profile() {
  if [[ -n "${ZSH_VERSION:-}" ]] || [[ "${SHELL:-}" == *zsh* ]]; then
    echo "${HOME}/.zshrc"
  elif [[ -f "${HOME}/.bash_profile" ]]; then
    echo "${HOME}/.bash_profile"
  else
    echo "${HOME}/.bashrc"
  fi
}

banner() {
  echo ""
  bold "⚡ Flash SDK installer"
  dim "  Compiled UI for iOS, Android & Web"
  echo ""
}

banner

# ── 1. Rust ──────────────────────────────────────────────────────────────────
cyan "Step 1/4 — Rust toolchain"
if ! command -v cargo >/dev/null 2>&1; then
  yellow "Rust not found — installing via rustup (this may take a minute)..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
  # shellcheck disable=SC1091
  source "${HOME}/.cargo/env"
fi
green "Rust $(rustc --version 2>/dev/null | awk '{print $2}')"

# ── 2. SDK ───────────────────────────────────────────────────────────────────
cyan "Step 2/4 — Flash SDK"
mkdir -p "$FLASH_HOME"
if [[ -d "$SDK_DIR/.git" ]]; then
  git -C "$SDK_DIR" fetch origin "$BRANCH" --quiet 2>/dev/null || true
  git -C "$SDK_DIR" checkout "$BRANCH" --quiet 2>/dev/null || true
  git -C "$SDK_DIR" pull origin "$BRANCH" --ff-only --quiet 2>/dev/null \
    || yellow "SDK update skipped (local changes in $SDK_DIR)"
  green "SDK updated at $SDK_DIR"
else
  git clone --depth 1 --branch "$BRANCH" "$REPO_URL" "$SDK_DIR"
  green "SDK cloned to $SDK_DIR"
fi

# ── 3. CLI ───────────────────────────────────────────────────────────────────
cyan "Step 3/4 — flash CLI"
export PATH="${HOME}/.cargo/bin:${PATH}"
if cargo install --path "$SDK_DIR/cli" --force --quiet 2>/dev/null; then
  green "flash → $(command -v flash)"
else
  cargo install --path "$SDK_DIR/cli" --force
  green "flash → $(command -v flash)"
fi

# ── 4. Shell profile ─────────────────────────────────────────────────────────
cyan "Step 4/4 — Shell environment"
PROFILE="$(detect_profile)"
mark_begin="# >>> flash sdk >>>"
mark_end="# <<< flash sdk <<<"

if ! grep -q "$mark_begin" "$PROFILE" 2>/dev/null; then
  cat >>"$PROFILE" <<EOF

$mark_begin
export FLASH_SDK="$SDK_DIR"
export PATH="\$HOME/.cargo/bin:\$PATH"
$mark_end
EOF
  green "Updated $PROFILE"
else
  green "Shell profile already configured"
fi

export FLASH_SDK="$SDK_DIR"
export PATH="${HOME}/.cargo/bin:${PATH}"

# ── Verify ───────────────────────────────────────────────────────────────────
echo ""
cyan "Verifying installation..."
if command -v flash >/dev/null 2>&1; then
  flash doctor || true
else
  red "flash not on PATH — run: source $PROFILE"
fi

# ── Quickstart (optional) ────────────────────────────────────────────────────
if [[ "${FLASH_QUICKSTART:-}" == "1" ]]; then
  echo ""
  cyan "Quickstart — creating demo app..."
  DEMO_DIR="${FLASH_DEMO_APP:-my_app}"
  if [[ ! -d "$DEMO_DIR" ]]; then
    flash create "$DEMO_DIR"
    green "Demo app created: $DEMO_DIR"
  else
    yellow "$DEMO_DIR already exists — skipping create"
  fi
fi

# ── Done ─────────────────────────────────────────────────────────────────────
echo ""
bold "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
bold "  Flash is ready!"
bold "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
dim "  Run these commands to start building:"
echo ""
echo "    source $PROFILE"
echo "    flash create my_app"
echo "    cd my_app"
echo "    flash run"
echo ""
dim "  flash run     Hot reload on save"
dim "  flash devices List simulators & native targets"
dim "  flash upgrade Update SDK + CLI later"
echo ""
dim "  Docs: https://github.com/Anu-Code07/flash"
echo ""
