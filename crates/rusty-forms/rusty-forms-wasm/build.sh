#!/bin/bash
# Build script for RHTMX Validation WASM

set -e

echo "🚀 Building RHTMX Validation WASM..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack not found!"
    echo "Install it with: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
    exit 1
fi

# Build for web
echo "📦 Building for web target..."
wasm-pack build --target web --release

echo "✅ Build complete!"
echo ""
echo "Output directory: pkg/"
echo ""
echo "To test the demo:"
echo "  1. Start an HTTP server: python3 -m http.server 8000"
echo "  2. Open: http://localhost:8000/demo.html"
