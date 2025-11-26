#!/bin/bash
set -e

# Get the directory of the script
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
EXTENSION_DIR="$SCRIPT_DIR/../extension"
MANIFEST_PATH="$EXTENSION_DIR/manifest.json"

REQUIRED_FILES=(
    "manifest.json"
    "src/content/index.js"
    "src/popup/index.html"
    "src/popup/index.js"
    "src/popup/styles.css"
    "readme.md"
)

echo -e "\033[0;36mChecking extension package in: $EXTENSION_DIR\033[0m"

# 1. Check for required files
MISSING_FILES=()
for file in "${REQUIRED_FILES[@]}"; do
    FILE_PATH="$EXTENSION_DIR/$file"
    if [ ! -f "$FILE_PATH" ]; then
        MISSING_FILES+=("$file")
        echo -e "\033[0;31m[-] Missing: $file\033[0m"
    else
        echo -e "\033[0;32m[+] Found: $file\033[0m"
    fi
done

if [ ${#MISSING_FILES[@]} -gt 0 ]; then
    echo -e "\033[0;31mError: Missing required files.\033[0m"
    exit 1
fi

# 2. Validate manifest.json
if command -v jq &> /dev/null; then
    if jq -e . "$MANIFEST_PATH" >/dev/null 2>&1; then
        echo -e "\033[0;32m[+] manifest.json is valid JSON\033[0m"
        
        VERSION=$(jq -r .version "$MANIFEST_PATH")
        if [ "$VERSION" != "null" ]; then
            echo -e "\033[0;37m    Version: $VERSION\033[0m"
        else
            echo -e "\033[0;33m[-] Warning: No version field in manifest.json\033[0m"
        fi

        MANIFEST_VERSION=$(jq -r .manifest_version "$MANIFEST_PATH")
        if [ "$MANIFEST_VERSION" == "3" ]; then
            echo -e "\033[0;37m    Manifest Version: 3 (Correct)\033[0m"
        else
            echo -e "\033[0;33m[-] Warning: manifest_version is not 3 (Found: $MANIFEST_VERSION)\033[0m"
        fi
    else
        echo -e "\033[0;31m[-] Error: manifest.json is invalid JSON\033[0m"
        exit 1
    fi
else
    echo -e "\033[0;33m[!] jq is not installed. Skipping JSON validation.\033[0m"
    # Basic check if file exists and is not empty
    if [ -s "$MANIFEST_PATH" ]; then
         echo -e "\033[0;32m[+] manifest.json exists and is not empty\033[0m"
    else
         echo -e "\033[0;31m[-] Error: manifest.json is empty or missing\033[0m"
         exit 1
    fi
fi

echo -e "\033[0;32mExtension package check passed!\033[0m"
exit 0
