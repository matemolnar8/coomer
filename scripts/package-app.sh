#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
cargo build --release
APP="$ROOT/packaging/Coomer.app"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS"
cp "$ROOT/target/release/coomer" "$APP/Contents/MacOS/coomer"
cp "$ROOT/packaging/Info.plist" "$APP/Contents/Info.plist"
chmod +x "$APP/Contents/MacOS/coomer"
echo "Built $APP"
