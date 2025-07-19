#!/bin/bash
set -e

echo "Running pre-commit checks..."

# Format code
echo "📝 Formatting code with rustfmt..."
cargo fmt --all -- --check

# Run clippy
echo "🔍 Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
echo "🧪 Running tests..."
cargo test --all

# Check for unused dependencies
echo "📦 Checking for unused dependencies..."
if command -v cargo-udeps &> /dev/null; then
    cargo +nightly udeps --all-targets
fi

# Security audit
echo "🔒 Running security audit..."
if command -v cargo-audit &> /dev/null; then
    cargo audit
fi

echo "✅ All pre-commit checks passed!"