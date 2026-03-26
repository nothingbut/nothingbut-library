#!/bin/bash

# EPUB Library Startup Script
# Starts the Tauri development server

echo "========================================"
echo "  EPUB Library - Development Mode"
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

echo "Starting EPUB Library..."
echo "Frontend: http://localhost:1420"
echo "EPUB Route: http://localhost:1420/epub"
echo ""
echo "Press Ctrl+C to stop"
echo ""

# Start Tauri dev
bun run tauri:dev
