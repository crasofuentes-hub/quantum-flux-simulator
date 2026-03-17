def normalize_rows(rows: list[dict]) -> list[dict]:
    cleaned = []
    for row in rows:
        cleaned.append(
            {
                "user": str(row.get("user", "")).strip().lower(),
                "region": str(row.get("region", "")).strip().upper(),
                "amount": float(row.get("amount", 0.0)),
            }
        )
    return cleaned


def aggregate_by_region(rows: list[dict]) -> dict:
    totals = {}
    for row in rows:
        region = row["region"]
        totals[region] = totals.get(region, 0.0) + row["amount"]
    return totals


def find_hotspots(totals: dict) -> list[str]:
    hotspots = []
    for region, value in totals.items():
        if value >= 1000.0:
            hotspots.append(region)
    return hotspots


def run_pipeline(rows: list[dict]) -> dict:
    normalized = normalize_rows(rows)
    totals = aggregate_by_region(normalized)
    hotspots = find_hotspots(totals)
    return {
        "rows": len(normalized),
        "regions": len(totals),
        "hotspots": hotspots,
    }