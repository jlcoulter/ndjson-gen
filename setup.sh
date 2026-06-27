#!/usr/bin/env bash
# setup.sh — rename crate and clean up template markers
# Usage: ./setup.sh mytool

set -euo pipefail

if [ $# -lt 1 ]; then
  echo "Usage: $0 <tool-name>"
  echo "Example: $0 cronlint"
  exit 1
fi

NAME="$1"
OLD_NAME="rust-cli-template"

# Replace in Cargo.toml
sed -i "s/${OLD_NAME}/${NAME}/g" Cargo.toml

# Replace in src/main.rs
sed -i "s/${OLD_NAME}/${NAME}/g" src/main.rs

# Replace in tests/integration_test.rs
sed -i "s/${OLD_NAME}/${NAME}/g" tests/integration_test.rs

# Replace in Makefile
sed -i "s/${OLD_NAME}/${NAME}/g" Makefile

# Replace in Dockerfile
sed -i "s/${OLD_NAME}/${NAME}/g" Dockerfile

# Replace in README
sed -i "s/${OLD_NAME}/${NAME}/g" README.md

# Replace in CI
sed -i "s/${OLD_NAME}/${NAME}/g" .github/workflows/ci.yml

# Remove setup script
rm -- "$0"

echo "Template configured: name=${NAME}"
echo "Next steps:"
echo "  1. Run: cargo test"
echo "  2. Run: make lint"
echo "  3. Delete src/example.rs and replace with your modules"