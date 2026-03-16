Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

Set-Location (Split-Path -Parent $PSScriptRoot)

function Import-VsDevCmdEnvironment {
  $candidates = @(
    "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat",
    "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\Community\Common7\Tools\VsDevCmd.bat",
    "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\Professional\Common7\Tools\VsDevCmd.bat",
    "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\Enterprise\Common7\Tools\VsDevCmd.bat"
  )

  $vsDevCmd = $candidates | Where-Object { Test-Path $_ } | Select-Object -First 1
  if (-not $vsDevCmd) {
    throw "No se encontró VsDevCmd.bat"
  }

  $cmd = "`"$vsDevCmd`" -arch=x64 -host_arch=x64 && set"
  $output = & cmd.exe /d /s /c $cmd

  foreach ($line in $output) {
    if ($line -match '^(.*?)=(.*)$') {
      Set-Item -Path ("Env:" + $matches[1]) -Value $matches[2]
    }
  }
}

Write-Host "`n== IMPORT MSVC ENV ==" -ForegroundColor Cyan
Import-VsDevCmdEnvironment

Write-Host "`n== FORMAT ==" -ForegroundColor Cyan
cargo fmt --all

Write-Host "`n== CLIPPY ==" -ForegroundColor Cyan
cargo clippy --all-targets --all-features -- -D warnings

Write-Host "`n== TEST ==" -ForegroundColor Cyan
cargo test

Write-Host "`n== BUILD RELEASE ==" -ForegroundColor Cyan
cargo build --release

Write-Host "`n== ANALYZE SMOKE ==" -ForegroundColor Cyan
cargo run -- analyze examples\my_crypto.py --quantum-noise 0.01 --relativistic 0.8c --target-temp 77K --json-out target\verify-report.json

Write-Host "`nOK: verify.ps1 PASS" -ForegroundColor Green