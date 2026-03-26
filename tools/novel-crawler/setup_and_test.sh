#!/bin/bash
# Installation and Test Script for macOS/Linux

echo "========================================"
echo "Novel Crawler - Setup and Test"
echo "========================================"
echo ""

# Check if venv exists
if [ ! -d "venv" ]; then
    echo "Creating virtual environment..."
    python3 -m venv venv
    if [ $? -ne 0 ]; then
        echo "ERROR: Failed to create virtual environment"
        echo "Please make sure Python 3.8+ is installed"
        exit 1
    fi
    echo "Virtual environment created successfully!"
    echo ""
fi

# Activate venv
echo "Activating virtual environment..."
source venv/bin/activate
if [ $? -ne 0 ]; then
    echo "ERROR: Failed to activate virtual environment"
    exit 1
fi
echo ""

# Install dependencies
echo "Installing dependencies..."
pip install -r requirements.txt
if [ $? -ne 0 ]; then
    echo "WARNING: Some packages may have failed to install"
    echo ""
fi
echo "Dependencies installed!"
echo ""

# Run tests
echo "========================================"
echo "Running Tests"
echo "========================================"
echo ""

echo "Running manual tests..."
python tests/test_manual.py all
echo ""

echo ""
echo "========================================"
echo "Setup Complete!"
echo "========================================"
echo ""
echo "You can now use the crawler with:"
echo "  python main.py single --book-id 1"
echo "  python main.py crawl --mode initial"
echo "  python main.py stats"
echo ""
