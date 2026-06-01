# Loom Development Environment Setup for Windows 11
# Run this script as Administrator

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Loom Development Environment Setup" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# 1. Install Chocolatey (package manager for Windows)
Write-Host "`n[1/7] Installing Chocolatey..." -ForegroundColor Yellow
Set-ExecutionPolicy Bypass -Scope Process -Force
[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))

# 2. Install Rust (via rustup)
Write-Host "`n[2/7] Installing Rust..." -ForegroundColor Yellow
iex ((New-Object System.Net.WebClient).DownloadString('https://win.rustup.rs/x86_64'))
# Wait for rustup to finish (user will need to press Enter)
Write-Host "Press Enter after Rust installation completes..." -ForegroundColor Cyan
Read-Host

# Refresh environment
refreshenv

# 3. Install Git
Write-Host "`n[3/7] Installing Git..." -ForegroundColor Yellow
choco install git -y

# 4. Install Python (required for some helper scripts)
Write-Host "`n[4/7] Installing Python..." -ForegroundColor Yellow
choco install python -y

# 5. Install GnuCOBOL (via MSYS2)
Write-Host "`n[5/7] Installing GnuCOBOL (requires MSYS2)..." -ForegroundColor Yellow
choco install msys2 -y

Write-Host "Installing COBOL compiler via pacman..." -ForegroundColor Yellow
refreshenv
& "C:\tools\msys64\msys2_shell.cmd" -defterm -no-start -here -use-full-path -c "pacman -S --noconfirm mingw-w64-x86_64-gcc mingw-w64-x86_64-gnu-cobol"

# Add MSYS2 MinGW to PATH
$mingwPath = "C:\tools\msys64\mingw64\bin"
if (Test-Path $mingwPath) {
    [Environment]::SetEnvironmentVariable("Path", $env:Path + ";$mingwPath", [EnvironmentVariableTarget]::Machine)
}

# 6. Install VS Code (optional but recommended)
Write-Host "`n[6/7] Installing Visual Studio Code..." -ForegroundColor Yellow
choco install vscode -y

# 7. Install cargo-watch (for development)
Write-Host "`n[7/7] Installing cargo-watch..." -ForegroundColor Yellow
cargo install cargo-watch

# Verify installations
Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  Verifying Installations" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

$versions = @(
    @{Name="Rust"; Command="rustc --version"},
    @{Name="Cargo"; Command="cargo --version"},
    @{Name="Git"; Command="git --version"},
    @{Name="Python"; Command="python --version"},
    @{Name="MSYS2"; Command="where msys2_shell.cmd"},
    @{Name="GnuCOBOL"; Command="cobc --version"},
    @{Name="VS Code"; Command="code --version"}
)

foreach ($item in $versions) {
    $result = & $item.Command 2>$null
    if ($result) {
        Write-Host "$($item.Name): OK - $($result.Split("`n")[0])" -ForegroundColor Green
    } else {
        Write-Host "$($item.Name): NOT FOUND" -ForegroundColor Red
    }
}

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  Next Steps" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "1. Restart your computer to complete PATH updates." -ForegroundColor Yellow