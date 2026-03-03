# install-rust.ps1
# Installs Rust and Cargo on Windows 11

$ErrorActionPreference = "Stop"

Write-Host "Checking for existing Rust installation..."

if (Get-Command rustc -ErrorAction SilentlyContinue) {
    Write-Host "Rust is already installed."
    rustc --version
    exit 0
}

Write-Host "Downloading Rust installer..."

$rustupUrl = "https://win.rustup.rs/x86_64"
$installerPath = "$env:TEMP\rustup-init.exe"

Invoke-WebRequest -Uri $rustupUrl -OutFile $installerPath

Write-Host "Running Rust installer..."

Start-Process -FilePath $installerPath -ArgumentList "-y" -Wait

Write-Host "Adding Cargo to PATH for current session..."

$env:Path += ";$env:USERPROFILE\.cargo\bin"

Write-Host "Verifying installation..."

rustc --version
cargo --version

Write-Host "Rust and Cargo installation complete."