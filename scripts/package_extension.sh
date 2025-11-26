#!/bin/bash
set -e

# Get the directory of the script
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$SCRIPT_DIR/.."
SRC_DIR="$PROJECT_ROOT/extension"
DIST_DIR="$PROJECT_ROOT/dist"
BROWSERS=("chrome" "firefox" "edge" "safari")

if [ ! -d "$SRC_DIR" ]; then
    echo "Error: 'extension' directory not found at $SRC_DIR"
    exit 1
fi

rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

for browser in "${BROWSERS[@]}"; do
    echo "Packaging for $browser..."
    BUILD_DIR="$DIST_DIR/${browser}_build"
    mkdir -p "$BUILD_DIR"

    # Copy src directory
    cp -r "$SRC_DIR/src" "$BUILD_DIR/"

    # Copy manifest
    # Priority: 1. manifests/$browser.json, 2. manifest.json (default)
    MANIFEST_SPECIFIC="$SRC_DIR/manifests/$browser.json"
    MANIFEST_DEFAULT="$SRC_DIR/manifest.json"
    
    if [ -f "$MANIFEST_SPECIFIC" ]; then
        cp "$MANIFEST_SPECIFIC" "$BUILD_DIR/manifest.json"
        echo "  Using specific manifest: $browser.json"
    elif [ -f "$MANIFEST_DEFAULT" ]; then
        cp "$MANIFEST_DEFAULT" "$BUILD_DIR/manifest.json"
        echo "  Using default manifest.json"
    else
        echo "  Warning: No manifest found for $browser"
    fi

    # Zip it
    # Use pushd/popd to zip relative to the build dir so we don't get the full path structure
    pushd "$BUILD_DIR" > /dev/null
    zip -r "../superpoweredcv-$browser.zip" .
    popd > /dev/null

    echo "  Created $DIST_DIR/superpoweredcv-$browser.zip"
done
