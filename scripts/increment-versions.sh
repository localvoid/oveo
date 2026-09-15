#!/usr/bin/env bash
set -euo pipefail

INCREMENT="${1:-}"
PKG_DIR="./packages/@oveo/optimizer"
ROLLDOWN_PKG_DIR="./packages/@oveo/rolldown"

case "$INCREMENT" in
  patch|minor|major) ;;
  *)
    echo "Usage: $0 <patch|minor|major>" >&2
    exit 1
    ;;
esac

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

# Sync rolldown plugin to the same version (lockstep with optimizer).
echo "$(jq --arg v "$NEW_VERSION" '.version = $v' "$ROLLDOWN_PKG_DIR/package.json")" > "$ROLLDOWN_PKG_DIR/package.json"

bun update

sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" crates/oveo/Cargo.toml
