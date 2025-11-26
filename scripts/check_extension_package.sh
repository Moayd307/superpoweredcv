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

# 3. Check Dist Packages
DIST_DIR="$SCRIPT_DIR/../dist"
BROWSERS=("chrome" "firefox" "edge" "safari")

if [ -d "$DIST_DIR" ]; then
    echo -e "\n\033[0;36mChecking distribution packages in: $DIST_DIR\033[0m"
    
    for browser in "${BROWSERS[@]}"; do
        ZIP_NAME="superpoweredcv-$browser.zip"
        ZIP_PATH="$DIST_DIR/$ZIP_NAME"
        
        if [ -f "$ZIP_PATH" ]; then
            echo -e "\033[0;32m[+] Found package: $ZIP_NAME\033[0m"
            
            if command -v unzip &> /dev/null; then
                # Check contents
                MISSING_IN_ZIP=()
                REQUIRED_IN_ZIP=("manifest.json" "src/content/index.js" "src/popup/index.html")
                
                # List zip contents
                ZIP_CONTENTS=$(unzip -l "$ZIP_PATH")
                
                for req in "${REQUIRED_IN_ZIP[@]}"; do
                    if ! echo "$ZIP_CONTENTS" | grep -q "$req"; then
                        MISSING_IN_ZIP+=("$req")
                    fi
                done
                
                if [ ${#MISSING_IN_ZIP[@]} -eq 0 ]; then
                    echo -e "\033[0;37m    [+] Contents verified (manifest, content script, popup)\033[0m"
                else
                    echo -e "\033[0;31m    [-] Missing files in zip: ${MISSING_IN_ZIP[*]}\033[0m"
                fi
            else
                 echo -e "\033[0;33m    [!] unzip command not found, skipping content check\033[0m"
            fi
        else
            echo -e "\033[0;33m[-] Missing package: $ZIP_NAME\033[0m"
        fi
    done
else
    echo -e "\n\033[0;33m[-] Dist directory not found. Run package_extension.sh first.\033[0m"
fi

echo -e "\n\033[0;32mExtension package check passed!\033[0m"
exit 0
