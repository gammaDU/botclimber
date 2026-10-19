# release.ps1 - Packages the Botclimber application for production

$ErrorActionPreference = "Stop"

# We now build the server and migrations for Linux using WSL to ensure compatibility.
Write-Host "Building server for Linux via WSL..."
wsl -d Debian -e /bin/bash -l -c "cargo build --release --bin botclimber"
Write-Host "Building migrations for Linux via WSL..."
wsl -d Debian -e /bin/bash -l -c "cargo build --release --bin migrate"

if (Test-Path .package) {
    Write-Host "Cleaning up old package directory..."
    Remove-Item -Recurse -Force .package
}

Write-Host "Creating package structure..."
New-Item -ItemType Directory -Path .package
New-Item -ItemType Directory -Path .package/frontend
New-Item -ItemType Directory -Path .package/frontend/dist

Write-Host "Copying frontend assets..."
if (Test-Path frontend/dist) {
    Copy-Item -Recurse frontend/dist/* .package/frontend/dist/
} else {
    Write-Warning "frontend/dist not found! Make sure to build the frontend first."
}

Write-Host "Copying Linux binaries..."
Copy-Item target/release/botclimber .package/
Copy-Item target/release/migrate .package/


Write-Host "Copying server configuration..."
Copy-Item server_config.toml .package/

Write-Host "Creating start.sh script..."
$startScript = @"
#!/bin/bash
set -e

# Ensure .data directory exists for SQLite
mkdir -p .data

# Check for required JWT_SECRET
if [ -z "`$BOTCLIMBER__AUTH__JWT_SECRET" ]; then
    echo "ERROR: BOTCLIMBER__AUTH__JWT_SECRET environment variable is not set."
    echo "Please set it to a secret string of at least 32 characters."
    exit 1
fi

echo "Running database migrations..."
./migrate .data/botclimber.db

echo "Starting Botclimber server..."
./botclimber server_config.toml
"@
$startScript | Out-File -FilePath .package/start.sh -Encoding ascii

if (Test-Path maps) {
    Write-Host "Copying maps..."
    Copy-Item -Recurse maps .package/
}

Write-Host "Creating release zip package..."
if (Test-Path botclimber-release.zip) {
    Remove-Item botclimber-release.zip
}
Compress-Archive -Path .package/* -DestinationPath botclimber-release.zip

Write-Host "Release package created: botclimber-release.zip"
