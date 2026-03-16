Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $repoRoot

if (!(Test-Path ".\Cargo.toml")) {
  throw "No se encontrÃ³ Cargo.toml en la raÃ­z del repo: $repoRoot"
}

$venvPath = Join-Path $repoRoot ".venv-radon"
$pythonExe = Join-Path $venvPath "Scripts\python.exe"
$targetDir = Join-Path $repoRoot "target"

Write-Host "`n== REPO ROOT ==" -ForegroundColor Cyan
Write-Host $repoRoot

Write-Host "`n== PREPARE VENV ==" -ForegroundColor Cyan
if (!(Test-Path $pythonExe)) {
  python -m venv $venvPath
}

& $pythonExe -m pip install --upgrade pip
& $pythonExe -m pip install radon

if (!(Test-Path $targetDir)) {
  New-Item -ItemType Directory -Force -Path $targetDir | Out-Null
}

Write-Host "`n== RADON CC JSON ==" -ForegroundColor Cyan
& $pythonExe -m radon cc .\benchmarks\dataset -j -s | Out-File -Encoding utf8 (Join-Path $targetDir "radon-cc.json")

Write-Host "`n== RADON MI JSON ==" -ForegroundColor Cyan
& $pythonExe -m radon mi .\benchmarks\dataset -j | Out-File -Encoding utf8 (Join-Path $targetDir "radon-mi.json")

Write-Host "`n== FLUX-SIM BENCHMARK ==" -ForegroundColor Cyan
cargo run -- benchmark benchmarks\dataset --quantum-noise 0.01 --relativistic 0.2c --target-temp 77K --json-out target\benchmark-report.json --seed 42

Write-Host "`n== WRITE COMPARISON SCRIPT ==" -ForegroundColor Cyan
$comparisonScriptPath = Join-Path $targetDir "build_comparison.py"
$comparisonScript = @"
import json
from pathlib import Path

root = Path(r"$($repoRoot.Path)")
radon_cc = json.loads((root / "target" / "radon-cc.json").read_text(encoding="utf-8-sig"))
radon_mi = json.loads((root / "target" / "radon-mi.json").read_text(encoding="utf-8-sig"))
flux = json.loads((root / "target" / "benchmark-report.json").read_text(encoding="utf-8-sig"))

def normalize_path(p: str) -> str:
    return p.replace("/", "\\\\").lstrip(".\\\\")

cc_index = {}
for path, items in radon_cc.items():
    norm = normalize_path(path)
    complexities = []
    for item in items:
        value = item.get("complexity")
        if isinstance(value, (int, float)):
            complexities.append(float(value))
    cc_index[norm] = {
        "blocks": len(items),
        "max_cc": max(complexities) if complexities else 0.0,
        "mean_cc": (sum(complexities) / len(complexities)) if complexities else 0.0,
    }

mi_index = {}
for path, payload in radon_mi.items():
    norm = normalize_path(path)
    if isinstance(payload, dict):
        mi_index[norm] = {
            "mi_rank": payload.get("rank", ""),
            "mi_score": float(payload.get("mi", 0.0)),
        }
    else:
        mi_index[norm] = {
            "mi_rank": "",
            "mi_score": 0.0,
        }

entries = []
for item in flux["entries"]:
    norm = normalize_path(item["path"])
    cc = cc_index.get(norm, {"blocks": 0, "max_cc": 0.0, "mean_cc": 0.0})
    mi = mi_index.get(norm, {"mi_rank": "", "mi_score": 0.0})
    entries.append({
        "path": norm,
        "expected_class": item["expected_class"],
        "detected_class": item["detected_class"],
        "class_match": item["class_match"],
        "baseline_structural_score": item["baseline"]["structural_score"],
        "baseline_risk": item["baseline"]["baseline_risk"],
        "baseline_stability": item["baseline"]["baseline_stability"],
        "flux_stability_score": item["stability_score"],
        "flux_singularity_risk": item["singularity_risk"],
        "flux_collapse_probability": item["collapse_probability"],
        "radon_blocks": cc["blocks"],
        "radon_max_cc": cc["max_cc"],
        "radon_mean_cc": cc["mean_cc"],
        "radon_mi_score": mi["mi_score"],
        "radon_mi_rank": mi["mi_rank"],
    })

def mean(values):
    values = list(values)
    return sum(values) / len(values) if values else 0.0

comparison = {
    "benchmark_source": "synthetic_dataset_v0",
    "seed": flux["seed"],
    "aggregate": {
        "files_analyzed": len(entries),
        "class_accuracy": flux["aggregate"]["class_accuracy"],
        "mean_flux_stability": mean(e["flux_stability_score"] for e in entries),
        "mean_flux_singularity_risk": mean(e["flux_singularity_risk"] for e in entries),
        "mean_flux_collapse_probability": mean(e["flux_collapse_probability"] for e in entries),
        "mean_baseline_stability": mean(e["baseline_stability"] for e in entries),
        "mean_baseline_risk": mean(e["baseline_risk"] for e in entries),
        "mean_radon_max_cc": mean(e["radon_max_cc"] for e in entries),
        "mean_radon_mean_cc": mean(e["radon_mean_cc"] for e in entries),
        "mean_radon_mi_score": mean(e["radon_mi_score"] for e in entries),
    },
    "entries": entries,
}

(root / "target" / "comparison-report.json").write_text(
    json.dumps(comparison, indent=2, sort_keys=True),
    encoding="utf-8-sig"
)

lines = []
lines.append("# External Baseline Comparison")
lines.append("")
lines.append("| File | Expected | Detected | Match | Flux Stability | Flux Risk | Collapse | Radon Max CC | Radon Mean CC | Radon MI |")
lines.append("|---|---|---|---:|---:|---:|---:|---:|---:|---:|")
for e in entries:
    lines.append(
        f"| {e['path']} | {e['expected_class']} | {e['detected_class']} | "
        f"{'yes' if e['class_match'] else 'no'} | "
        f"{e['flux_stability_score']:.3f} | {e['flux_singularity_risk']:.3f} | {e['flux_collapse_probability']:.3f} | "
        f"{e['radon_max_cc']:.3f} | {e['radon_mean_cc']:.3f} | {e['radon_mi_score']:.3f} |"
    )

(root / "target" / "comparison-summary.md").write_text("\\n".join(lines) + "\\n", encoding="utf-8-sig")
print("comparison report written")
"@

[System.IO.File]::WriteAllText($comparisonScriptPath, $comparisonScript, (New-Object System.Text.UTF8Encoding($false)))

Write-Host "`n== RUN COMPARISON SCRIPT ==" -ForegroundColor Cyan
& $pythonExe $comparisonScriptPath
$pyExit = $LASTEXITCODE
if ($pyExit -ne 0) {
  throw "build_comparison.py falló con exit code $pyExit"
}

Write-Host "`n== OUTPUTS ==" -ForegroundColor Cyan
Get-Item (Join-Path $targetDir "radon-cc.json"),
         (Join-Path $targetDir "radon-mi.json"),
         (Join-Path $targetDir "benchmark-report.json"),
         (Join-Path $targetDir "comparison-report.json"),
         (Join-Path $targetDir "comparison-summary.md") | Format-Table Name, Length

Write-Host "`n== PREVIEW ==" -ForegroundColor Cyan
Get-Content (Join-Path $targetDir "comparison-summary.md") -TotalCount 20