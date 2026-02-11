#!/bin/bash
set -e

echo "Setting up OpenGP database..."

export DATABASE_URL="sqlite:opengp.db"

if [ ! -f "opengp.db" ]; then
    echo "Creating database..."
    sqlite3 opengp.db "SELECT 1;"
else
    echo "Database already exists"
fi

echo "Running migrations..."
if ! command -v sqlx &> /dev/null; then
    echo "Installing sqlx-cli..."
    cargo install sqlx-cli --features sqlite --no-default-features
fi

sqlx migrate run

echo "Database setup complete!"
echo ""
echo "Database info:"
sqlite3 opengp.db << 'EOF'
SELECT 'Tables:' as info;
.tables
SELECT '' as blank;
SELECT 'Patients count: ' || COUNT(*) FROM patients;
SELECT 'Users count: ' || COUNT(*) FROM users;
EOF
