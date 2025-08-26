# BWS Test Runner - Hot Reload Testing Suite (PowerShell)
# Runs both unit tests and integration tests for hot reload functionality

param(
    [switch]$UnitOnly,
    [switch]$IntegrationOnly,
    [switch]$Verbose,
    [switch]$Help
)

if ($Help) {
    Write-Host "BWS Hot Reload Test Runner" -ForegroundColor Blue
    Write-Host "===========================" -ForegroundColor Blue
    Write-Host ""
    Write-Host "Usage: .\run_hot_reload_tests.ps1 [options]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -UnitOnly           Run only unit tests"
    Write-Host "  -IntegrationOnly    Run only integration tests"
    Write-Host "  -Verbose            Enable verbose output"
    Write-Host "  -Help               Show this help message"
    exit 0
}

# Configuration
$TestsDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $TestsDir

# Test statistics
$Script:TotalTests = 0
$Script:PassedTests = 0
$Script:FailedTests = 0

# Logging functions
function Write-Info {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green
}

function Write-TestError {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor Yellow
}

# Run unit tests
function Invoke-UnitTests {
    Write-Info "Running hot reload unit tests..."
    
    Push-Location $ProjectRoot
    
    try {
        $output = cargo test hot_reload --lib 2>&1
        $exitCode = $LASTEXITCODE
        
        if ($Verbose) {
            $output | Write-Host
        }
        
        if ($exitCode -eq 0) {
            # Extract test count from output
            $testCount = 0
            foreach ($line in $output) {
                if ($line -match "(\d+) passed") {
                    $testCount = [int]$matches[1]
                    break
                }
            }
            
            Write-Success "Unit tests passed ($testCount tests)"
            $Script:TotalTests += $testCount
            $Script:PassedTests += $testCount
            return $true
        } else {
            Write-TestError "Unit tests failed"
            $Script:FailedTests += 1
            return $false
        }
    } finally {
        Pop-Location
    }
}

# Run integration tests
function Invoke-IntegrationTests {
    Write-Info "Running hot reload integration tests..."
    
    $integrationScript = Join-Path $TestsDir "test_hot_reload.ps1"
    
    if (-not (Test-Path $integrationScript)) {
        Write-TestError "Integration test script not found: $integrationScript"
        $Script:FailedTests += 1
        return $false
    }
    
    try {
        if ($Verbose) {
            & $integrationScript -Verbose
        } else {
            & $integrationScript
        }
        
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Integration tests passed"
            $Script:TotalTests += 1
            $Script:PassedTests += 1
            return $true
        } else {
            Write-TestError "Integration tests failed"
            $Script:FailedTests += 1
            return $false
        }
    } catch {
        Write-TestError "Exception running integration tests: $($_.Exception.Message)"
        $Script:FailedTests += 1
        return $false
    }
}

# Build BWS binaries
function Build-Bws {
    Write-Info "Building BWS binaries..."
    
    Push-Location $ProjectRoot
    
    try {
        $output = cargo build 2>&1
        $exitCode = $LASTEXITCODE
        
        if ($Verbose) {
            $output | Write-Host
        }
        
        if ($exitCode -eq 0) {
            Write-Success "BWS binaries built successfully"
            return $true
        } else {
            Write-TestError "Failed to build BWS binaries"
            if (-not $Verbose) {
                $output | Write-Host
            }
            return $false
        }
    } finally {
        Pop-Location
    }
}

# Check prerequisites
function Test-Prerequisites {
    Write-Info "Checking prerequisites..."
    
    # Check if cargo is available
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-TestError "cargo is not installed or not in PATH"
        return $false
    }
    
    # Check if we're in the right directory
    if (-not (Test-Path (Join-Path $ProjectRoot "Cargo.toml"))) {
        Write-TestError "Not in BWS project directory"
        return $false
    }
    
    # Check PowerShell version
    if ($PSVersionTable.PSVersion.Major -lt 5) {
        Write-Warning "PowerShell version may be too old. Recommended: 5.1+"
    }
    
    Write-Success "Prerequisites check passed"
    return $true
}

# Main function
function Invoke-MainTests {
    Write-Host "==================================================" -ForegroundColor Blue
    Write-Host "BWS Hot Reload Test Runner (PowerShell)" -ForegroundColor Blue
    Write-Host "==================================================" -ForegroundColor Blue
    
    # Check prerequisites
    if (-not (Test-Prerequisites)) {
        exit 1
    }
    
    # Build BWS
    if (-not (Build-Bws)) {
        exit 1
    }
    
    Write-Host ""
    Write-Host "Running test suite..." -ForegroundColor Blue
    Write-Host ""
    
    $unitResult = $true
    $integrationResult = $true
    
    # Run unit tests (unless integration only)
    if (-not $IntegrationOnly) {
        $unitResult = Invoke-UnitTests
        Write-Host ""
    }
    
    # Run integration tests (unless unit only)
    if (-not $UnitOnly) {
        $integrationResult = Invoke-IntegrationTests
        Write-Host ""
    }
    
    # Print summary
    Write-Host "==================================================" -ForegroundColor Blue
    Write-Host "Test Summary" -ForegroundColor Blue
    Write-Host "==================================================" -ForegroundColor Blue
    
    if (-not $IntegrationOnly) {
        if ($unitResult) {
            Write-Host "✓ Unit Tests: PASSED" -ForegroundColor Green
        } else {
            Write-Host "✗ Unit Tests: FAILED" -ForegroundColor Red
        }
    }
    
    if (-not $UnitOnly) {
        if ($integrationResult) {
            Write-Host "✓ Integration Tests: PASSED" -ForegroundColor Green
        } else {
            Write-Host "✗ Integration Tests: FAILED" -ForegroundColor Red
        }
    }
    
    Write-Host ""
    Write-Host "Total Tests: $($Script:TotalTests)" -ForegroundColor Blue
    Write-Host "Passed: $($Script:PassedTests)" -ForegroundColor Green
    Write-Host "Failed: $($Script:FailedTests)" -ForegroundColor Red
    Write-Host ""
    
    if ($Script:FailedTests -eq 0) {
        Write-Host "🎉 All hot reload tests passed!" -ForegroundColor Green
        exit 0
    } else {
        Write-Host "❌ Some tests failed. Check the output above for details." -ForegroundColor Red
        exit 1
    }
}

# Execute main function
Invoke-MainTests
