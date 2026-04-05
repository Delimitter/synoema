#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# Copyright Synoema Contributors
#
# Atomically bump version in all npm package.json files.
# Usage: ./scripts/npm-bump-version.sh <version>
# Example: ./scripts/npm-bump-version.sh 0.1.0-alpha.2

set -euo pipefail

VERSION="${1:-}"

if [ -z "$VERSION" ]; then
  echo "Usage: $0 <version>"
  echo "Example: $0 0.1.0-alpha.2"
  exit 1
fi

# Validate version format: MAJOR.MINOR.PATCH[-STAGE.N]
if ! echo "$VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+(-[a-z]+\.[0-9]+)?$'; then
  echo "Error: invalid version format '$VERSION'"
  echo "Expected: MAJOR.MINOR.PATCH or MAJOR.MINOR.PATCH-STAGE.N"
  exit 1
fi

# Cross-platform sed -i
sedi() {
  if [[ "$OSTYPE" == "darwin"* ]]; then
    sed -i '' "$@"
  else
    sed -i "$@"
  fi
}

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

PACKAGES=(
  "$REPO_ROOT/npm/synoema-mcp/package.json"
  "$REPO_ROOT/npm/platforms/darwin-arm64/package.json"
  "$REPO_ROOT/npm/platforms/darwin-x64/package.json"
  "$REPO_ROOT/npm/platforms/linux-x64/package.json"
  "$REPO_ROOT/npm/platforms/win32-x64/package.json"
)

for pkg in "${PACKAGES[@]}"; do
  if [ ! -f "$pkg" ]; then
    echo "Warning: $pkg not found, skipping"
    continue
  fi
  sedi "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" "$pkg"
  echo "  updated: $pkg"
done

# Update optionalDependencies in main package
MAIN_PKG="$REPO_ROOT/npm/synoema-mcp/package.json"
for platform in darwin-arm64 darwin-x64 linux-x64 win32-x64; do
  sedi "s/\"@delimitter\/mcp-$platform\": \"[^\"]*\"/\"@delimitter\/mcp-$platform\": \"$VERSION\"/" "$MAIN_PKG"
done

echo ""
echo "All npm packages bumped to $VERSION"
echo ""
echo "Next steps:"
echo "  git add npm/"
echo "  git commit -m \"chore: bump npm packages to $VERSION\""
echo "  git tag v$VERSION"
echo "  git push origin v$VERSION"
