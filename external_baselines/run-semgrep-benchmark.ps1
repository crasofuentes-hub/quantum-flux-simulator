Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $repoRoot

if (!(Test-Path ".\Cargo.toml")) {
  throw "No se encontró Cargo.toml en la raíz del repo: $repoRoot"
}

$venvPath = Join-Path $repoRoot ".venv-semgrep"
$pythonExe = Join-Path $venvPath "Scripts\python.exe"
$targetDir = Join-Path $repoRoot "target"
$datasetDir = Join-Path $repoRoot "datasets\seeded_defects"

if (!(Test-Path $datasetDir)) {
  throw "No existe datasets/seeded_defects: $datasetDir"
}

Write-Host "`n== REPO ROOT ==" -ForegroundColor Cyan
Write-Host $repoRoot

Write-Host "`n== PREPARE VENV ==" -ForegroundColor Cyan
if (!(Test-Path $pythonExe)) {
  python -m venv $venvPath
}

& $pythonExe -m pip install --upgrade pip
& $pythonExe -m pip install semgrep

if (!(Test-Path $targetDir)) {
  New-Item -ItemType Directory -Force -Path $targetDir | Out-Null
}

$semgrepJson = Join-Path $targetDir "semgrep-results.json"
$summaryJson = Join-Path $targetDir "semgrep-summary.json"

Write-Host "`n== RUN SEMGREP ==" -ForegroundColor Cyan
& $pythonExe -m semgrep scan `
  --config auto `
  --json `
  --output $semgrepJson `
  $datasetDir

if (!(Test-Path $semgrepJson)) {
  throw "Semgrep no produjo semgrep-results.json"
}

Write-Host "`n== BUILD SUMMARY ==" -ForegroundColor Cyan
$builderPath = Join-Path $targetDir "build_semgrep_summary.py"
$builder = @"
import json
from pathlib import Path

root = Path(r"$($repoRoot.Path)")
results_path = root / "target" / "semgrep-results.json"
summary_path = root / "target" / "semgrep-summary.json"

payload = json.loads(results_path.read_text(encoding="utf-8-sig"))
results = payload.get("results", [])

by_file = {}
for item in results:
    path = item.get("path", "")
    check_id = item.get("check_id", "")
    severity = item.get("extra", {}).get("severity", "")
    entry = by_file.setdefault(path, {"findings": 0, "checks": [], "severities": []})
    entry["findings"] += 1
    if check_id:
        entry["checks"].append(check_id)
    if severity:
        entry["severities"].append(severity)

summary = {
    "dataset": "seeded_defects_v1",
    "tool": "semgrep",
    "total_findings": len(results),
    "files_scanned": len(by_file),
    "files": by_file,
}

summary_path.write_text(json.dumps(summary, indent=2, sort_keys=True), encoding="utf-8-sig")
print("semgrep summary written")
"@

[System.IO.File]::WriteAllText($builderPath, $builder, (New-Object System.Text.UTF8Encoding($false)))
& $pythonExe $builderPath

if (!(Test-Path $summaryJson)) {
  throw "No se produjo semgrep-summary.json"
}

Write-Host "`n== OUTPUTS ==" -ForegroundColor Cyan
Get-Item $semgrepJson, $summaryJson | Format-Table Name, Length

Write-Host "`n== PREVIEW ==" -ForegroundColor Cyan
Get-Content $summaryJson -TotalCount 40