#!/bin/bash
# Set version in Cargo.toml and package.json
#
# Usage: ./scripts/set-version.sh 0.2.0

set -e

VERSION="$1"

if [ -z "$VERSION" ]; then
  echo "Usage: $0 <version>"
  echo "Example: $0 0.2.0"
  exit 1
fi

# Validate version format (semver)
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Error: Version must be in semver format (e.g., 0.2.0)"
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "Setting version to $VERSION"

# Update Cargo.toml
sed -i '' "s/^version = \".*\"/version = \"$VERSION\"/" "$PROJECT_DIR/Cargo.toml"
echo "✓ Updated Cargo.toml"

# Update package.json
sed -i '' "s/\"version\": \".*\"/\"version\": \"$VERSION\"/" "$PROJECT_DIR/package.json"
echo "✓ Updated package.json"

echo ""
echo "Version set to $VERSION in both files."
