param(
    [switch]$Force
)

$ErrorActionPreference = "Stop"

$compilerDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $compilerDir

$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
$cargoExe = Join-Path $cargoBin "cargo.exe"

if (-not (Test-Path $cargoExe)) {
    throw "cargo.exe not found at $cargoExe. Install Rust first: https://rustup.rs/"
}

$sessionPathParts = $env:Path -split ';' | Where-Object { $_ -ne "" }
if (-not ($sessionPathParts -contains $cargoBin)) {
    $env:Path = "$cargoBin;$env:Path"
}

$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
$userPathParts = @()
if ($userPath) {
    $userPathParts = $userPath -split ';' | Where-Object { $_ -ne "" }
}
if (-not ($userPathParts -contains $cargoBin)) {
    Write-Host "Adding $cargoBin to user PATH..."
    $newUserPath = if ($userPath) { "$userPath;$cargoBin" } else { $cargoBin }
    [Environment]::SetEnvironmentVariable("Path", $newUserPath, "User")
}

$installArgs = @(
    "install",
    "--path", ".",
    "--features", "llvm",
    "--bin", "falcon"
)

if ($Force) {
    $installArgs += "--force"
}

Write-Host "Installing Falcon CLI globally..."
& $cargoExe @installArgs

$falconExe = Join-Path $cargoBin "falcon.exe"
if (-not (Test-Path $falconExe)) {
    throw "Installation completed but $falconExe was not found."
}

Write-Host ""
Write-Host "Done. Falcon is installed at:"
Write-Host "  $falconExe"
if (-not (Get-Command falcon -ErrorAction SilentlyContinue)) {
    Write-Host "Current shell does not resolve 'falcon' yet. Open a new terminal."
}
Write-Host "Run:"
Write-Host "  falcon --help"
