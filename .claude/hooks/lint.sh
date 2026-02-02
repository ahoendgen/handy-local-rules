#!/bin/bash
# Claude Code Hook: Lint files after Edit/Write
# Runs clippy for Rust files

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
    # Run clippy on the whole project (clippy doesn't support single files well)
    # Use --message-format=short for concise output
    cargo clippy --message-format=short -- -D warnings 2>&1 | head -20 || true
    ;;
esac

exit 0
