$ErrorActionPreference = "Stop"

$scriptDir = $PSScriptRoot
$projectRoot = Join-Path $scriptDir ".."
$srcDir = Join-Path $projectRoot "extension"
$distDir = Join-Path $projectRoot "dist"
$browsers = @("chrome", "firefox", "edge", "safari")

Write-Host "Packaging extension from: $srcDir" -ForegroundColor Cyan

# Ensure we are in the project root context
if (-not (Test-Path -Path $srcDir)) {
    Write-Error "Error: 'extension' directory not found at $srcDir"
    exit 1
}

# Run Lint and Tests
Write-Host "Running Lint and Tests..." -ForegroundColor Cyan
Push-Location $srcDir
try {
    npm install
    npm run lint
    if ($LASTEXITCODE -ne 0) { throw "Linting failed" }
    npm run test
    if ($LASTEXITCODE -ne 0) { throw "Tests failed" }
}
catch {
    Write-Error "Pre-build checks failed: $_"
    Pop-Location
    exit 1
}
Pop-Location

# Create dist directory
if (Test-Path -Path $distDir) { 
    Write-Host "Cleaning up old dist directory..." -ForegroundColor Yellow
    Remove-Item -Path $distDir -Recurse -Force 
}
New-Item -ItemType Directory -Path $distDir | Out-Null

foreach ($browser in $browsers) {
    Write-Host "Packaging for $browser..." -ForegroundColor Green
    $buildDir = Join-Path $distDir "${browser}_build"
    New-Item -ItemType Directory -Path $buildDir | Out-Null

    # Copy src directory
    $srcSource = Join-Path $srcDir "src"
    Copy-Item -Path $srcSource -Destination $buildDir -Recurse

    # Copy manifest
    # Priority: 1. manifests/$browser.json, 2. manifest.json (default)
    $manifestSpecific = Join-Path $srcDir "manifests" "$browser.json"
    $manifestDefault = Join-Path $srcDir "manifest.json"
    
    if (Test-Path -Path $manifestSpecific) {
        Copy-Item -Path $manifestSpecific -Destination (Join-Path $buildDir "manifest.json")
        Write-Host "  Using specific manifest: $browser.json" -ForegroundColor Gray
    }
    elseif (Test-Path -Path $manifestDefault) {
        Copy-Item -Path $manifestDefault -Destination (Join-Path $buildDir "manifest.json")
        Write-Host "  Using default manifest.json" -ForegroundColor Gray
    }
    else {
        Write-Warning "  No manifest found for $browser"
    }

    # Zip it
    $zipFile = Join-Path $distDir "superpoweredcv-$browser.zip"
    
    # Compress-Archive can be finicky with paths, so we use relative paths if possible or ensure full paths
    # To avoid including the root folder in the zip, we need to be careful.
    # PowerShell's Compress-Archive puts the folder inside if you zip the folder.
    # We want the contents.
    
    $filesToZip = Get-ChildItem -Path $buildDir
    Compress-Archive -Path $filesToZip.FullName -DestinationPath $zipFile -Force
    
    Write-Host "  Created $zipFile" -ForegroundColor Cyan
}
