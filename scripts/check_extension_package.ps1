# Check Extension Package Script

$extensionDir = Join-Path $PSScriptRoot "..\extension"
$manifestPath = Join-Path $extensionDir "manifest.json"
$requiredFiles = @(
    "manifest.json",
    "src/content/index.js",
    "src/popup/index.html",
    "src/popup/index.js",
    "src/popup/styles.css",
    "readme.md"
)

Write-Host "Checking extension package in: $extensionDir" -ForegroundColor Cyan

# 1. Check for required files
$missingFiles = @()
foreach ($file in $requiredFiles) {
    $filePath = Join-Path $extensionDir $file
    if (-not (Test-Path $filePath)) {
        $missingFiles += $file
        Write-Host "[-] Missing: $file" -ForegroundColor Red
    } else {
        Write-Host "[+] Found: $file" -ForegroundColor Green
    }
}

if ($missingFiles.Count -gt 0) {
    Write-Host "Error: Missing required files." -ForegroundColor Red
    exit 1
}

# 2. Validate manifest.json
try {
    $manifestContent = Get-Content $manifestPath -Raw | ConvertFrom-Json
    Write-Host "[+] manifest.json is valid JSON" -ForegroundColor Green
    
    # Check version
    if ($manifestContent.version) {
        Write-Host "    Version: $($manifestContent.version)" -ForegroundColor Gray
    } else {
        Write-Host "[-] Warning: No version field in manifest.json" -ForegroundColor Yellow
    }

    # Check manifest_version
    if ($manifestContent.manifest_version -eq 3) {
        Write-Host "    Manifest Version: 3 (Correct)" -ForegroundColor Gray
    } else {
        Write-Host "[-] Warning: manifest_version is not 3 (Found: $($manifestContent.manifest_version))" -ForegroundColor Yellow
    }

} catch {
    Write-Host "[-] Error: manifest.json is invalid JSON" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    exit 1
}

# 3. Check Dist Packages
$distDir = Join-Path $PSScriptRoot "..\dist"
$browsers = @("chrome", "firefox", "edge", "safari")

if (Test-Path $distDir) {
    Write-Host "`nChecking distribution packages in: $distDir" -ForegroundColor Cyan
    
    Add-Type -AssemblyName System.IO.Compression.FileSystem

    foreach ($browser in $browsers) {
        $zipName = "superpoweredcv-$browser.zip"
        $zipPath = Join-Path $distDir $zipName
        
        if (Test-Path $zipPath) {
            Write-Host "[+] Found package: $zipName" -ForegroundColor Green
            
            try {
                $zip = [System.IO.Compression.ZipFile]::OpenRead($zipPath)
                $entries = $zip.Entries | Select-Object -ExpandProperty FullName
                
                # Check for critical files inside zip
                # Note: Zip entries usually use forward slashes
                $criticalZipFiles = @(
                    "manifest.json",
                    "src/content/index.js",
                    "src/popup/index.html"
                )
                
                $missingInZip = @()
                foreach ($crit in $criticalZipFiles) {
                    # Normalize slashes for comparison just in case
                    $found = $entries | Where-Object { $_ -replace '\\', '/' -match "$crit$" }
                    if (-not $found) {
                        $missingInZip += $crit
                    }
                }
                
                if ($missingInZip.Count -eq 0) {
                    Write-Host "    [+] Contents verified (manifest, content script, popup)" -ForegroundColor Gray
                } else {
                    Write-Host "    [-] Missing files in zip: $($missingInZip -join ', ')" -ForegroundColor Red
                }
                
                $zip.Dispose()
            } catch {
                Write-Host "    [-] Error reading zip file: $_" -ForegroundColor Red
            }
        } else {
            Write-Host "[-] Missing package: $zipName" -ForegroundColor Yellow
        }
    }
} else {
    Write-Host "`n[-] Dist directory not found. Run package_extension.ps1 first." -ForegroundColor Yellow
}

Write-Host "`nExtension package check passed!" -ForegroundColor Green
exit 0
