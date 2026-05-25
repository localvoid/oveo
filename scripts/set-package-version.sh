#!/usr/bin/env bash
set -euo pipefail

DIR="$1"
VER="$2"

echo "$(jq --arg v "$VER" '.version = $v' "$DIR/package.json")" > "$DIR/package.json"
