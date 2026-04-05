#!/usr/bin/env bash
set -euo pipefail

# Synoema VSCode Extension — one-command install
# Usage: ./install.sh [--keep]
#   --keep  Don't clean up node_modules/dist after install

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

KEEP=false
for arg in "$@"; do
  case "$arg" in
    --keep) KEEP=true ;;
    -h|--help)
      echo "Usage: ./install.sh [--keep]"
      echo "  --keep  Keep node_modules and dist after install"
      exit 0
      ;;
    *) echo "Unknown option: $arg"; exit 1 ;;
  esac
done

# --- Colors ---
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
NC='\033[0m'

info()  { echo -e "${CYAN}[info]${NC}  $1"; }
ok()    { echo -e "${GREEN}[ok]${NC}    $1"; }
warn()  { echo -e "${YELLOW}[warn]${NC}  $1"; }
fail()  { echo -e "${RED}[fail]${NC}  $1"; exit 1; }

# --- 1. Check dependencies ---
info "Checking dependencies..."

command -v node >/dev/null 2>&1 || fail "node not found. Install Node.js: https://nodejs.org"
command -v npm  >/dev/null 2>&1 || fail "npm not found. Install Node.js: https://nodejs.org"

if ! command -v code >/dev/null 2>&1; then
  fail "code CLI not found. Open VSCode → Cmd+Shift+P → 'Shell Command: Install code command in PATH'"
fi

ok "node $(node --version), npm $(npm --version), code CLI"

# --- 2. Install npm dependencies ---
info "Installing dependencies..."
npm install --no-audit --no-fund --loglevel=error
ok "npm dependencies installed"

# --- 3. Build extension ---
info "Building extension..."
npm run build 2>&1
ok "Extension built → dist/extension.js"

# --- 4. Package as .vsix ---
info "Packaging .vsix..."
npx vsce package --allow-missing-repository -o synoema.vsix 2>&1
ok "Packaged → synoema.vsix"

# --- 5. Install in VSCode ---
info "Installing extension in VSCode..."
code --install-extension synoema.vsix --force
ok "Extension installed in VSCode"

# --- 6. Cleanup ---
if [ "$KEEP" = false ]; then
  info "Cleaning up build artifacts..."
  rm -f synoema.vsix
  rm -rf node_modules dist
  ok "Cleaned up"
else
  info "Keeping build artifacts (--keep)"
fi

echo ""
echo -e "${GREEN}Done!${NC} Synoema extension is installed."
echo "Reload VSCode (Cmd+Shift+P → 'Reload Window') to activate."
