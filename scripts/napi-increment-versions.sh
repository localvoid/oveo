#!/usr/bin/env bash
set -euo pipefail

INCREMENT="$1"
PKG_DIR="./packages/@oveo/optimizer"

cd "$PKG_DIR"
bun pm version "$INCREMENT" --no-git-tag-version
NEW_VERSION="$(jq -r .version package.json)"
cd - > /dev/null

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
for dir in darwin-arm64 darwin-x64 linux-arm64-gnu linux-x64-gnu win32-arm64-msvc win32-x64-msvc; do
  "$SCRIPT_DIR/set-package-version.sh" "$PKG_DIR/packages/$dir" "$NEW_VERSION"
done

sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" crates/oveo/Cargo.toml
