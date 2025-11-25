$ErrorActionPreference = "Stop"

$srcDir = "extension"
$distDir = "dist"

# Ensure we are in the project root
if (-not (Test-Path -Path $srcDir)) {
    Write-Error "Error: 'extension' directory not found. Please run this script from the project root."
    exit 1
}

# Create dist directory
if (-not (Test-Path -Path $distDir)) {
    New-Item -ItemType Directory -Path $distDir | Out-Null
}

Write-Host "Packaging for Chrome..."
$chromeBuildDir = Join-Path $distDir "chrome_build"
if (Test-Path -Path $chromeBuildDir) { Remove-Item -Path $chromeBuildDir -Recurse -Force }
New-Item -ItemType Directory -Path $chromeBuildDir | Out-Null

# Copy files
Copy-Item -Path "$srcDir\*" -Destination $chromeBuildDir -Recurse

# Remove Firefox manifest
Remove-Item -Path "$chromeBuildDir\manifest-firefox.json" -ErrorAction SilentlyContinue

# Zip it
$chromeZip = Join-Path $distDir "superpoweredcv-chrome.zip"
if (Test-Path -Path $chromeZip) { Remove-Item -Path $chromeZip -Force }
Compress-Archive -Path "$chromeBuildDir\*" -DestinationPath $chromeZip

# Clean up
Remove-Item -Path $chromeBuildDir -Recurse -Force
Write-Host "Created $chromeZip"

Write-Host "Packaging for Firefox..."
$firefoxBuildDir = Join-Path $distDir "firefox_build"
if (Test-Path -Path $firefoxBuildDir) { Remove-Item -Path $firefoxBuildDir -Recurse -Force }
New-Item -ItemType Directory -Path $firefoxBuildDir | Out-Null

# Copy files
Copy-Item -Path "$srcDir\*" -Destination $firefoxBuildDir -Recurse

# Remove Chrome manifest and rename Firefox manifest
Remove-Item -Path "$firefoxBuildDir\manifest.json" -ErrorAction SilentlyContinue
Rename-Item -Path "$firefoxBuildDir\manifest-firefox.json" -NewName "manifest.json"

# Zip it
$firefoxZip = Join-Path $distDir "superpoweredcv-firefox.zip"
if (Test-Path -Path $firefoxZip) { Remove-Item -Path $firefoxZip -Force }
Compress-Archive -Path "$firefoxBuildDir\*" -DestinationPath $firefoxZip

# Clean up
Remove-Item -Path $firefoxBuildDir -Recurse -Force
Write-Host "Created $firefoxZip"
