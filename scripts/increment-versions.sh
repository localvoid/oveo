#!/usr/bin/env bash
set -euo pipefail

INCREMENT="$1"
PKG_DIR="./packages/@oveo/optimizer"

cd "$PKG_DIR"
bun pm version "$INCREMENT" --no-git-tag-version
NEW_VERSION="$(jq -r .version package.json)"
cd - > /dev/null

PKGS=(
  "darwin-arm64"
  "darwin-x64"
  "linux-arm64-gnu"
  "linux-x64-gnu"
  "win32-arm64-msvc"
  "win32-x64-msvc"
)

for dir in "${PKGS[@]}"; do
  echo "$(jq --arg v "$NEW_VERSION" '.version = $v' "$PKG_DIR/packages/${dir}/package.json")" > "$PKG_DIR/packages/${dir}/package.json"
done
bun update

sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" crates/oveo/Cargo.toml
