#!/usr/bin/env bash
# Install Flash from a local git clone (delegates to install.sh).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export FLASH_REPO="$ROOT"
export FLASH_BRANCH="$(git -C "$ROOT" rev-parse --abbrev-ref HEAD 2>/dev/null || echo main)"
exec "$ROOT/install.sh"
