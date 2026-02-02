#!/bin/bash
# Claude Code Hook: Auto-format files after Edit/Write
# Formats Rust files with cargo fmt, others with Prettier

set -e

INPUT=$(cat)
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty')

if [[ -z "$FILE_PATH" ]]; then
  exit 0
fi

# Get file extension
EXT="${FILE_PATH##*.}"

case "$EXT" in
  rs)
    # Format Rust files with cargo fmt
    cargo fmt -- "$FILE_PATH" 2>/dev/null || true
    ;;
  md|json|yaml|yml)
    # Format with Prettier
    pnpm prettier --write "$FILE_PATH" 2>/dev/null || true
    ;;
esac

exit 0
