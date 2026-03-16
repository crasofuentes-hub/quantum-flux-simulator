Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

Set-Location (Split-Path -Parent $PSScriptRoot)

Write-Host "`n== CARGO TOML ==" -ForegroundColor Cyan
if (!(Test-Path ".\Cargo.toml")) { throw "Falta Cargo.toml" }

Write-Host "`n== README ==" -ForegroundColor Cyan
if (!(Test-Path ".\README.md")) { throw "Falta README.md" }

Write-Host "`n== LICENSE ==" -ForegroundColor Cyan
if (!(Test-Path ".\LICENSE")) { throw "Falta LICENSE" }

Write-Host "`n== CI WORKFLOW ==" -ForegroundColor Cyan
if (!(Test-Path ".\.github\workflows\ci.yml")) { throw "Falta .github\workflows\ci.yml" }

Write-Host "`n== EXAMPLES ==" -ForegroundColor Cyan
$examples = @(
  ".\examples\my_crypto.py",
  ".\examples\navier_stub.py",
  ".\examples\ml_stub.py"
)
foreach ($e in $examples) {
  if (!(Test-Path $e)) { throw "Falta ejemplo: $e" }
}

Write-Host "`n== TEST FILES ==" -ForegroundColor Cyan
$tests = @(
  ".\tests\cli_smoke.rs",
  ".\tests\analysis_cases.rs",
  ".\tests\json_smoke.rs"
)
foreach ($t in $tests) {
  if (!(Test-Path $t)) { throw "Falta test: $t" }
}

Write-Host "`n== GIT STATUS ==" -ForegroundColor Cyan
git status --short

Write-Host "`nOK: release-check.ps1 PASS" -ForegroundColor Green