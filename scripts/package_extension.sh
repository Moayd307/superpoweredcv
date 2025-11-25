#!/bin/bash
set -e

# Directory containing the extension source
SRC_DIR="extension"
DIST_DIR="dist"

# Ensure we are in the project root
if [ ! -d "$SRC_DIR" ]; then
    echo "Error: 'extension' directory not found. Please run this script from the project root."
    exit 1
fi

# Create dist directory
mkdir -p "$DIST_DIR"

echo "Packaging for Chrome..."
# Create a temporary directory for Chrome build
rm -rf "$DIST_DIR/chrome_build"
mkdir -p "$DIST_DIR/chrome_build"

# Copy files
cp -r "$SRC_DIR/"* "$DIST_DIR/chrome_build/"

# Remove Firefox manifest
rm "$DIST_DIR/chrome_build/manifest-firefox.json"

# Zip it
# We use pushd/popd to zip relative to the build directory
pushd "$DIST_DIR/chrome_build" > /dev/null
zip -r "../superpoweredcv-chrome.zip" .
popd > /dev/null

# Clean up
rm -rf "$DIST_DIR/chrome_build"
echo "Created $DIST_DIR/superpoweredcv-chrome.zip"

echo "Packaging for Firefox..."
# Create a temporary directory for Firefox build
rm -rf "$DIST_DIR/firefox_build"
mkdir -p "$DIST_DIR/firefox_build"

# Copy files
cp -r "$SRC_DIR/"* "$DIST_DIR/firefox_build/"

# Remove Chrome manifest and rename Firefox manifest
rm "$DIST_DIR/firefox_build/manifest.json"
mv "$DIST_DIR/firefox_build/manifest-firefox.json" "$DIST_DIR/firefox_build/manifest.json"

# Zip it
pushd "$DIST_DIR/firefox_build" > /dev/null
zip -r "../superpoweredcv-firefox.zip" .
popd > /dev/null

# Clean up
rm -rf "$DIST_DIR/firefox_build"
echo "Created $DIST_DIR/superpoweredcv-firefox.zip"
