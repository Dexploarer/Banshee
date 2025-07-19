#!/bin/bash
set -e

echo "🧠 Emotional AI Agents Framework Demo"
echo "====================================="

echo "📋 Running pre-commit checks..."
cargo fmt --check
cargo clippy --all-targets --all-features
echo "✅ All checks passed!"

echo ""
echo "🧪 Running comprehensive tests..."
cargo test --all
echo "✅ All tests passed!"

echo ""
echo "📊 Performance benchmark..."
cargo test --release -p emotion_engine -- --ignored
echo "✅ Performance tests completed!"

echo ""
echo "🎯 Framework is ready for development!"
echo ""
echo "Next steps:"
echo "  1. cargo watch -x check -x test  # Live development"
echo "  2. cargo test -p emotion_engine  # Test specific crate"
echo "  3. cargo run --example <name>    # Run examples (when available)"
echo ""
echo "📚 See README.md for detailed usage instructions"