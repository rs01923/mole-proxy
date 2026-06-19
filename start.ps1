# Mole Proxy Windows Startup Script

$ErrorActionPreference = "Stop"

# Add common Docker Desktop path to environment PATH for the current session
$dockerBinPath = "C:\Program Files\Docker\Docker\resources\bin"
if (Test-Path $dockerBinPath) {
    $env:Path = "$dockerBinPath;$env:Path"
}

# Check for cargo
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Error: 'cargo' could not be found." -ForegroundColor Red
    Write-Host "Please install Rust and cargo (e.g., via https://rustup.rs/) before running this script." -ForegroundColor Yellow
    exit 1
}

# Check for docker
$dockerCmd = ""
if (Get-Command docker-compose -ErrorAction SilentlyContinue) {
    $dockerCmd = "docker-compose"
} elseif (Get-Command docker -ErrorAction SilentlyContinue) {
    $dockerCmd = "docker compose"
} else {
    Write-Host "Error: Neither 'docker-compose' nor 'docker' could be found." -ForegroundColor Red
    Write-Host "Please install Docker Desktop and ensure it is in your PATH or at the default installation path." -ForegroundColor Yellow
    exit 1
}

# Check if Docker daemon is running
& docker ps >$null 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "Error: Docker daemon is not running." -ForegroundColor Red
    Write-Host "Please open Docker Desktop and make sure the engine is fully started before running this script." -ForegroundColor Yellow
    exit 1
}

Write-Host "Building GUI..." -ForegroundColor Green
try {
    cargo build --release --manifest-path mole-proxy-gui/Cargo.toml
    Copy-Item mole-proxy-gui/target/release/mole-proxy-gui.exe ./gui.exe -Force
} catch {
    Write-Host "Failed to build the GUI: $_" -ForegroundColor Red
    exit 1
}

if (-not (Test-Path "mullvad-config")) {
    Write-Host "Configuration folder not found. Running the initial build..." -ForegroundColor Yellow
    Invoke-Expression "$dockerCmd up -d --build"
} else {
    Write-Host "Configuration found! Skipping the build and starting the proxy..." -ForegroundColor Yellow
    Invoke-Expression "$dockerCmd up -d"
}

Write-Host "All setup! Use the Mole Proxy GUI to interface with the setup." -ForegroundColor Green
Write-Host "To stop the container use `"$dockerCmd down`"" -ForegroundColor Cyan

# Start the GUI
Start-Process .\gui.exe
