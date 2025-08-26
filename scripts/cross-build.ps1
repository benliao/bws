# Cross-compilation build script for BWS (PowerShell)
# This script builds BWS for multiple target platforms

param(
    [switch]$Help,
    [string[]]$Targets = @(),
    [switch]$SkipTests,
    [switch]$Verbose
)

# Show help
if ($Help) {
    Write-Host "BWS Cross-Compilation Build Script" -ForegroundColor Blue
    Write-Host "=================================" -ForegroundColor Blue
    Write-Host ""
    Write-Host "Usage: .\cross-build.ps1 [options]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -Help           Show this help message"
    Write-Host "  -Targets        Specify targets to build (comma-separated)"
    Write-Host "  -SkipTests      Skip running tests"
    Write-Host "  -Verbose        Enable verbose output"
    Write-Host ""
    Write-Host "Examples:"
    Write-Host "  .\cross-build.ps1"
    Write-Host "  .\cross-build.ps1 -Targets x86_64-unknown-linux-musl,aarch64-unknown-linux-musl"
    Write-Host "  .\cross-build.ps1 -SkipTests -Verbose"
    exit 0
}

# Colors for output
$colors = @{
    Red = "Red"
    Green = "Green" 
    Yellow = "Yellow"
    Blue = "Blue"
    Cyan = "Cyan"
}

# Build information
try {
    $version = (cargo pkgid) -replace '.*#', ''
    $buildDate = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    try {
        $gitCommit = git rev-parse --short HEAD 2>$null
        if (-not $gitCommit) { $gitCommit = "unknown" }
    } catch {
        $gitCommit = "unknown"
    }
} catch {
    Write-Error "Failed to get build information. Make sure you're in the BWS project directory."
    exit 1
}

Write-Host "🔨 BWS Cross-Compilation Build Script" -ForegroundColor Blue
Write-Host "=====================================" -ForegroundColor Blue
Write-Host "Version: $version"
Write-Host "Build Date: $buildDate" 
Write-Host "Git Commit: $gitCommit"
Write-Host ""

# Export build variables
$env:BWS_VERSION = $version
$env:BUILD_DATE = $buildDate
$env:GIT_COMMIT = $gitCommit

# Define default targets
$defaultTargets = @(
    "x86_64-unknown-linux-musl",      # Linux x64 static
    "aarch64-unknown-linux-musl",     # Linux ARM64 static
    "x86_64-unknown-linux-gnu",       # Linux x64 dynamic
    "aarch64-unknown-linux-gnu",      # Linux ARM64 dynamic
    "armv7-unknown-linux-musleabihf", # ARMv7 static
    "x86_64-pc-windows-msvc"          # Windows x64
)

# Use specified targets or default
$targetList = if ($Targets.Count -gt 0) { $Targets } else { $defaultTargets }

# Function to build for a specific target
function Invoke-TargetBuild {
    param([string]$Target)
    
    Write-Host "🏗️  Building for $Target..." -ForegroundColor Yellow
    
    # Determine which tool to use
    $usesCross = (Get-Command cross -ErrorAction SilentlyContinue) -and ($Target -ne "x86_64-pc-windows-msvc")
    
    try {
        if ($usesCross) {
            Write-Host "Using cross for $Target"
            $buildArgs = @("build", "--release", "--target", $Target, "--bin", "bws")
            if ($Verbose) { $buildArgs += "--verbose" }
            & cross @buildArgs
            
            $buildArgs = @("build", "--release", "--target", $Target, "--bin", "bws-ctl")
            if ($Verbose) { $buildArgs += "--verbose" }
            & cross @buildArgs
        } else {
            Write-Host "Using cargo for $Target"
            $buildArgs = @("build", "--release", "--target", $Target, "--bin", "bws")
            if ($Verbose) { $buildArgs += "--verbose" }
            & cargo @buildArgs
            
            $buildArgs = @("build", "--release", "--target", $Target, "--bin", "bws-ctl")
            if ($Verbose) { $buildArgs += "--verbose" }
            & cargo @buildArgs
        }
        
        if ($LASTEXITCODE -eq 0) {
            Write-Host "✅ Successfully built for $Target" -ForegroundColor Green
            
            # Get binary info
            $binaryPath = "target\$Target\release\bws"
            $ctlBinaryPath = "target\$Target\release\bws-ctl"
            if ($Target -like "*windows*") {
                $binaryPath = "target\$Target\release\bws.exe"
                $ctlBinaryPath = "target\$Target\release\bws-ctl.exe"
            }
            
            if (Test-Path $binaryPath) {
                $size = (Get-Item $binaryPath).Length
                $sizeStr = if ($size -gt 1MB) { "{0:N1} MB" -f ($size / 1MB) } else { "{0:N1} KB" -f ($size / 1KB) }
                Write-Host "   📦 BWS binary size: $sizeStr"
                Write-Host "   📍 BWS location: $binaryPath"
            }
            
            if (Test-Path $ctlBinaryPath) {
                $size = (Get-Item $ctlBinaryPath).Length
                $sizeStr = if ($size -gt 1MB) { "{0:N1} MB" -f ($size / 1MB) } else { "{0:N1} KB" -f ($size / 1KB) }
                Write-Host "   📦 BWS-CTL binary size: $sizeStr"
                Write-Host "   📍 BWS-CTL location: $ctlBinaryPath"
            }
            Write-Host ""
            return $true
        } else {
            Write-Host "❌ Failed to build for $Target" -ForegroundColor Red
            return $false
        }
    } catch {
        Write-Host "❌ Exception building for $Target : $_" -ForegroundColor Red
        return $false
    }
}

# Function to verify binary
function Test-Binary {
    param([string]$Target)
    
    $binaryPath = "target\$Target\release\bws"
    if ($Target -like "*windows*") {
        $binaryPath = "target\$Target\release\bws.exe"
    }
    
    if (Test-Path $binaryPath) {
        Write-Host "🔍 Verifying $Target binary..." -ForegroundColor Blue
        
        # Check file info
        $fileInfo = Get-Item $binaryPath
        Write-Host "   Size: $($fileInfo.Length) bytes"
        Write-Host "   Created: $($fileInfo.CreationTime)"
        
        # Try to get version info for Windows binaries
        if ($Target -like "*windows*") {
            try {
                $versionInfo = [System.Diagnostics.FileVersionInfo]::GetVersionInfo($binaryPath)
                if ($versionInfo.FileVersion) {
                    Write-Host "   Version: $($versionInfo.FileVersion)"
                }
            } catch {
                # Ignore version info errors
            }
        }
        Write-Host ""
    }
}

# Clean previous builds
Write-Host "🧹 Cleaning previous builds..." -ForegroundColor Yellow
cargo clean
Write-Host ""

# Add required targets
Write-Host "🎯 Adding required targets..." -ForegroundColor Yellow
foreach ($target in $targetList) {
    Write-Host "Adding target: $target"
    rustup target add $target 2>$null
}
Write-Host ""

# Run tests first (unless skipped)
if (-not $SkipTests) {
    Write-Host "🧪 Running tests..." -ForegroundColor Yellow
    cargo test --release
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Tests failed!" -ForegroundColor Red
        exit 1
    }
    Write-Host "✅ Tests passed!" -ForegroundColor Green
    Write-Host ""
}

# Main build loop
Write-Host "🚀 Starting cross-compilation builds..." -ForegroundColor Yellow
Write-Host ""

$successfulBuilds = @()
$failedBuilds = @()

foreach ($target in $targetList) {
    if (Invoke-TargetBuild $target) {
        $successfulBuilds += $target
        Test-Binary $target
    } else {
        $failedBuilds += $target
    }
}

# Build summary
Write-Host "📊 Build Summary" -ForegroundColor Blue
Write-Host "===============" -ForegroundColor Blue
Write-Host ""

if ($successfulBuilds.Count -gt 0) {
    Write-Host "✅ Successful builds ($($successfulBuilds.Count)):" -ForegroundColor Green
    foreach ($target in $successfulBuilds) {
        Write-Host "   ✓ $target"
    }
    Write-Host ""
}

if ($failedBuilds.Count -gt 0) {
    Write-Host "❌ Failed builds ($($failedBuilds.Count)):" -ForegroundColor Red
    foreach ($target in $failedBuilds) {
        Write-Host "   ✗ $target"
    }
    Write-Host ""
}

# Create distribution directory
if ($successfulBuilds.Count -gt 0) {
    Write-Host "📦 Creating distribution packages..." -ForegroundColor Yellow
    
    $distDir = "dist"
    if (-not (Test-Path $distDir)) {
        New-Item -ItemType Directory -Path $distDir | Out-Null
    }
    
    foreach ($target in $successfulBuilds) {
        $binaryPath = "target\$target\release\bws"
        $ctlBinaryPath = "target\$target\release\bws-ctl"
        $packageDir = "$distDir\bws-$version-$target"
        
        if ($target -like "*windows*") {
            $binaryPath = "target\$target\release\bws.exe"
            $ctlBinaryPath = "target\$target\release\bws-ctl.exe"
        }
        
        if (Test-Path $binaryPath) {
            Write-Host "Creating package for $target..."
            
            # Create package directory
            if (Test-Path $packageDir) {
                Remove-Item $packageDir -Recurse -Force
            }
            New-Item -ItemType Directory -Path $packageDir | Out-Null
            
            # Copy binaries
            Copy-Item $binaryPath $packageDir
            if (Test-Path $ctlBinaryPath) {
                Copy-Item $ctlBinaryPath $packageDir
            }
            
            # Copy documentation
            if (Test-Path "README.md") { Copy-Item "README.md" $packageDir }
            if (Test-Path "LICENSE") { Copy-Item "LICENSE" $packageDir }
            
            # Copy example configuration
            if (Test-Path "config.toml") { 
                Copy-Item "config.toml" "$packageDir\config.example.toml" 
            }
            
            # Create zip archive
            $archivePath = "$distDir\bws-$version-$target.zip"
            if (Test-Path $archivePath) {
                Remove-Item $archivePath -Force
            }
            
            Compress-Archive -Path $packageDir -DestinationPath $archivePath
            Write-Host "   📦 Created: $archivePath"
        }
    }
    Write-Host ""
}

# Show final results
Write-Host "🎉 Cross-compilation complete!" -ForegroundColor Blue
Write-Host "Built $($successfulBuilds.Count) out of $($targetList.Count) targets"

if ($failedBuilds.Count -eq 0) {
    Write-Host "All builds successful! 🚀" -ForegroundColor Green
    exit 0
} else {
    Write-Host "Some builds failed. Check the output above for details." -ForegroundColor Yellow
    exit 1
}
