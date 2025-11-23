#!/bin/bash
set -e

APP_NAME="stele"
BUNDLE_NAME="${APP_NAME}.app"
BUILD_DIR="target/release"
APP_DIR="${BUILD_DIR}/${BUNDLE_NAME}"

echo "Building release binary..."
cargo build --release

echo "Creating app bundle..."
rm -rf "${APP_DIR}"
mkdir -p "${APP_DIR}/Contents/MacOS"
mkdir -p "${APP_DIR}/Contents/Resources"

echo "Copying files..."
cp "${BUILD_DIR}/stele" "${APP_DIR}/Contents/MacOS/stele"
cp "Info.plist" "${APP_DIR}/Contents/Info.plist"

# Copy icon if it exists
if [ -f "assets/icon.icns" ]; then
    cp "assets/icon.icns" "${APP_DIR}/Contents/Resources/icon.icns"
fi

echo "Done! App bundle created at: ${APP_DIR}"
echo "You can run it with: open ${APP_DIR}"
