#!/bin/bash
set -e

# Read version from Cargo.toml
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
APP_NAME="Praymodoro"

echo "Building Praymodoro v${VERSION}..."

# Build release binary
cargo build --release

# Create app bundle structure
APP_DIR="target/release/${APP_NAME}.app"
CONTENTS_DIR="${APP_DIR}/Contents"
MACOS_DIR="${CONTENTS_DIR}/MacOS"
RESOURCES_DIR="${CONTENTS_DIR}/Resources"

# Clean and create directories
rm -rf "${APP_DIR}"
mkdir -p "${MACOS_DIR}"
mkdir -p "${RESOURCES_DIR}"

# Copy binary
cp "target/release/praymodoro" "${MACOS_DIR}/"

# Copy Info.plist
cp "macos/Info.plist" "${CONTENTS_DIR}/"

# Copy icon
cp "assets/icons/Praymodoro.icns" "${RESOURCES_DIR}/"

# Copy assets (preserve directory structure for path resolution)
mkdir -p "${RESOURCES_DIR}/assets"
cp -r "assets/characters" "${RESOURCES_DIR}/assets/"
cp -r "assets/ui" "${RESOURCES_DIR}/assets/"
cp -r "assets/fonts" "${RESOURCES_DIR}/assets/"

echo "Built ${APP_DIR}"

# Create DMG using create-dmg (same approach as Tauri)
DMG_NAME="${APP_NAME}_${VERSION}_aarch64.dmg"
DMG_PATH="target/release/${DMG_NAME}"

echo "Creating DMG with create-dmg..."

# Clean up any existing DMG
rm -f "${DMG_PATH}"

# Create the DMG with create-dmg
create-dmg \
    --volname "${APP_NAME}" \
    --volicon "assets/icons/Praymodoro.icns" \
    --window-size 500 350 \
    --icon-size 128 \
    --icon "${APP_NAME}.app" 150 175 \
    --app-drop-link 350 175 \
    --hide-extension "${APP_NAME}.app" \
    "${DMG_PATH}" \
    "${APP_DIR}"

echo ""
echo "=== Build Complete ==="
echo "App bundle: ${APP_DIR}"
echo "DMG: ${DMG_PATH}"
echo ""
echo "To install: Open the DMG and drag Praymodoro to Applications"
