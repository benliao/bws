#!/usr/bin/env pwsh
# Docker-based Rust Clippy runner - Inline command solution

param(
    [string]$RustVersion = "latest",
    [switch]$Fix,
    [switch]$Verbose
)

$ErrorActionPreference = "Stop"

# Configuration
$ProjectPath = Split-Path -Parent $PSScriptRoot

Write-Host "🚀 Docker Clippy for BWS" -ForegroundColor Green
Write-Host "=========================" -ForegroundColor Green

# Validate Docker
Write-Host "🔍 Checking Docker..." -ForegroundColor Yellow
if (!(Get-Command docker -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Docker not found" -ForegroundColor Red
    exit 1
}
Write-Host "✅ Docker found" -ForegroundColor Green

Write-Host "📁 Project: $ProjectPath" -ForegroundColor Cyan

# Build clippy arguments
$ClippyArgs = "clippy"
if ($Fix) {
    $ClippyArgs += " --fix --allow-dirty"
    Write-Host "🔧 Fix mode enabled" -ForegroundColor Yellow
}
$ClippyArgs += " --"
if ($Verbose) {
    $ClippyArgs += " -v"
    Write-Host "📝 Verbose output enabled" -ForegroundColor Cyan
}

Write-Host "🐳 Running Docker container..." -ForegroundColor Yellow

# Single inline command
$Command = "apt-get update && apt-get install -y cmake build-essential pkg-config && rustup component add clippy && cd /workspace && cargo $ClippyArgs"

docker run --rm -it -v "${ProjectPath}:/workspace" -w /workspace rust:$RustVersion bash -c $Command

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Clippy completed successfully!" -ForegroundColor Green
} else {
    Write-Host "❌ Clippy found issues" -ForegroundColor Red
    exit $LASTEXITCODE
}
