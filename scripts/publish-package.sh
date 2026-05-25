#!/usr/bin/env bash
set -euo pipefail

DIR="$1"
shift

cd "$DIR"
PKG_NAME="$(jq -r .name package.json)"
CURRENT_VERSION="$(jq -r .version package.json)"
REMOTE_VERSION="$(npm --no-workspaces view "$PKG_NAME" version 2>/dev/null || true)"

if [ "$REMOTE_VERSION" != "$CURRENT_VERSION" ]; then
  filename="${PWD}/archive.tgz"
  bun pm pack --filename "${filename}"
  npm --no-workspaces publish "${filename}" "$@"
  rm -rf "${filename}"
fi
