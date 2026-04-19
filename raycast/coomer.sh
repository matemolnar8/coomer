#!/bin/bash
# Required parameters:
# @raycast.schemaVersion 1
# @raycast.title coomer
# @raycast.mode silent
# @raycast.packageName coomer
# @raycast.icon 🎯

COOMER_BIN="${COOMER_BIN:-$HOME/projects/coomer/target/release/coomer}"
exec "$COOMER_BIN"
