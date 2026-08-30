$ErrorActionPreference = "Stop"

$Project = "C:\Users\tayeb\Documents\loom"

Set-Location $Project

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "        LOOM DEVELOPMENT TEST" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

if (-not (Test-Path "$Project\Cargo.toml")) {
    Write-Host "ERROR: Cargo.toml not found." -ForegroundColor Red
    exit 1
}

Write-Host "[1/4] cargo check --all-targets" -ForegroundColor Yellow
cargo check --all-targets
if ($LASTEXITCODE -ne 0) {
    Write-Host "cargo check FAILED." -ForegroundColor Red
    exit $LASTEXITCODE
}

Write-Host ""
Write-Host "[2/4] cargo test --lib" -ForegroundColor Yellow
cargo test --lib
if ($LASTEXITCODE -ne 0) {
    Write-Host "Library tests FAILED." -ForegroundColor Red
    exit $LASTEXITCODE
}

Write-Host ""
Write-Host "[3/4] cargo test --tests" -ForegroundColor Yellow
cargo test --tests
if ($LASTEXITCODE -ne 0) {
    Write-Host "Integration tests FAILED." -ForegroundColor Red
    exit $LASTEXITCODE
}

Write-Host ""
Write-Host "[4/4] cargo test --all-targets" -ForegroundColor Yellow
cargo test --all-targets
if ($LASTEXITCODE -ne 0) {
    Write-Host "Full test suite FAILED." -ForegroundColor Red
    exit $LASTEXITCODE
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "       ALL LOOM TESTS PASSED" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""

Write-Host "Git status:" -ForegroundColor Cyan
git status --short

Write-Host ""
Write-Host "No commit or push was performed." -ForegroundColor DarkGray
