# Falcon Example Sweep — Test Harness
# Builds all .fc examples for userland profile, captures results.
# Usage: powershell -File scripts/run_examples.ps1

param(
    [string]$ExampleDir = "..\examples",
    [string]$Profile = "userland"
)

$ErrorActionPreference = "Continue"
$compiler = "cargo"
$compilerArgs = @("run", "--features", "llvm", "--")

$pass = 0
$fail = 0
$skip = 0
$results = @()

$examples = Get-ChildItem -Path $ExampleDir -Filter "*.fc" -File | Sort-Object Name

Write-Host "========================================" -ForegroundColor Cyan
Write-Host " Falcon Example Sweep — Profile: $Profile" -ForegroundColor Cyan
Write-Host " Found $($examples.Count) .fc files" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

foreach ($fc in $examples) {
    $name = $fc.Name
    $fullPath = $fc.FullName
    
    # Skip known non-userland examples
    if ($Profile -eq "userland" -and ($name -match "baremetal|kernel_main")) {
        Write-Host "  SKIP  $name (non-userland entrypoint)" -ForegroundColor Yellow
        $skip++
        $results += [PSCustomObject]@{ File = $name; Status = "SKIP"; Detail = "non-userland" }
        continue
    }

    # Build
    $buildArgs = $compilerArgs + @("build", $fullPath, "--profile", $Profile)
    $proc = Start-Process -FilePath $compiler -ArgumentList $buildArgs `
        -WorkingDirectory (Split-Path $PSScriptRoot -Parent | Join-Path -ChildPath "compiler") `
        -NoNewWindow -Wait -PassThru -RedirectStandardError "$env:TEMP\falcon_sweep_err.txt" `
        -RedirectStandardOutput "$env:TEMP\falcon_sweep_out.txt" 2>$null

    if ($proc.ExitCode -eq 0) {
        Write-Host "  PASS  $name" -ForegroundColor Green
        $pass++
        $results += [PSCustomObject]@{ File = $name; Status = "PASS"; Detail = "" }
    }
    else {
        $errText = if (Test-Path "$env:TEMP\falcon_sweep_err.txt") {
            (Get-Content "$env:TEMP\falcon_sweep_err.txt" -Raw).Trim()
        }
        else { "unknown error" }
        # Truncate error for display
        $shortErr = if ($errText.Length -gt 120) { $errText.Substring(0, 120) + "..." } else { $errText }
        Write-Host "  FAIL  $name — $shortErr" -ForegroundColor Red
        $fail++
        $results += [PSCustomObject]@{ File = $name; Status = "FAIL"; Detail = $shortErr }
    }
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host " Results: $pass PASS / $fail FAIL / $skip SKIP (of $($examples.Count) total)" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# Write results to CSV
$csvPath = Join-Path $PSScriptRoot "sweep_results.csv"
$results | Export-Csv -Path $csvPath -NoTypeInformation -Force
Write-Host "  Results saved to: $csvPath"

exit $fail
