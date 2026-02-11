#!/bin/bash
# Development run script for OpenGP TUI
# Sets up environment variables and runs the application

set -e

echo "🏥 OpenGP - Development Mode"
echo "=============================="

# Set development environment variables
export DATABASE_URL="${DATABASE_URL:-sqlite:opengp.db}"
export ENCRYPTION_KEY="${ENCRYPTION_KEY:-0000000000000000000000000000000000000000000000000000000000000000}"
export SESSION_TIMEOUT_SECS="${SESSION_TIMEOUT_SECS:-900}"
export LOG_LEVEL="${LOG_LEVEL:-info}"
export DATA_DIR="${DATA_DIR:-./data}"

echo "📋 Configuration:"
echo "  Database: $DATABASE_URL"
echo "  Log Level: $LOG_LEVEL"
echo "  Data Dir: $DATA_DIR"
echo ""
echo "⚠️  WARNING: Using development encryption key!"
echo "    Never use this in production!"
echo ""
echo "🚀 Starting OpenGP..."
echo ""

# Run the application
cargo run --release

# Note: Use Ctrl+C or 'q' to quit
