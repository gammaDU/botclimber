#!/bin/bash
set -e

if [ ! -f "server_config.toml" ]; then
    echo "ERROR: server_config.toml not found. Please provide a configuration file."
    exit 1
fi

# Extract database path from server_config.toml using sed
# It looks for the [database] section and then the path = "..." line
DB_PATH=$(sed -n '/\[database\]/,/\[.*\]/p' server_config.toml | grep '^path =' | sed 's/path = "\(.*\)"/\1/')

if [ -z "$DB_PATH" ]; then
    echo "ERROR: Could not find database path in server_config.toml"
    exit 1
fi

# Ensure directory for the database exists
DB_DIR=$(dirname "$DB_PATH")
mkdir -p "$DB_DIR"

# Check for required JWT_SECRET
if [ -z "$BOTCLIMBER__AUTH__JWT_SECRET" ]; then
    echo "ERROR: BOTCLIMBER__AUTH__JWT_SECRET environment variable is not set."
    echo "Please set it to a secret string of at least 32 characters."
    exit 1
fi

echo "Running database migrations..."
./migrate "$DB_PATH"

echo "Starting Botclimber server..."
./botclimber server_config.toml
