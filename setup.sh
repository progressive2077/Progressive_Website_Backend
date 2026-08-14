#!/usr/bin/env bash
# PCFI Backend setup script
# Installs sqlx-cli, creates the database, runs migrations, and prepares
# offline SQLx query cache so `cargo build` works without a live DB later.

set -e

echo "==> Checking for sqlx-cli..."
if ! command -v sqlx &> /dev/null; then
    echo "Installing sqlx-cli (this can take a few minutes)..."
    cargo install sqlx-cli --no-default-features --features postgres
fi

if [ ! -f .env ]; then
    echo "==> No .env found, copying from .env.example"
    cp .env.example .env
    echo "    Edit .env and set a strong JWT_SECRET before deploying to production."
fi

export $(grep -v '^#' .env | xargs)

echo "==> Creating database (if it doesn't exist)..."
sqlx database create

echo "==> Running migrations..."
sqlx migrate run

echo "==> Preparing offline query cache (.sqlx/)..."
cargo sqlx prepare

echo ""
echo "Setup complete. You can now run:"
echo "  cargo run"
echo ""
echo "Default admin login:"
echo "  email:    admin@pcfi.com.np"
echo "  password: Admin@123"
