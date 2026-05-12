#!/bin/bash

# NothingBut Library Startup Script
# Starts the Tauri development server

echo "========================================"
echo "  NothingBut Library - Dev Mode"
echo "========================================"
echo ""

# Check if bun is installed
if ! command -v bun &> /dev/null; then
    echo "Error: bun is not installed"
    echo "Please install bun: https://bun.sh"
    exit 1
fi

# Check if node_modules exists
if [ ! -d "node_modules" ]; then
    echo "Installing dependencies..."
    bun install
    echo ""
fi

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo is not installed"
    echo "Please install Rust: https://rustup.rs"
    exit 1
fi

echo "Starting NothingBut Library..."
echo ""
echo "🏠 Homepage:   http://localhost:1420"
echo "📚 Novel:      http://localhost:1420/novel"
echo "📖 EPUB:       http://localhost:1420/epub"
echo "🎵 Music:      http://localhost:1420/music"
echo ""
echo "Press Ctrl+C to stop"
echo ""

# Start Tauri dev
bun run tauri:dev
