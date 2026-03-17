def normalize_rows(rows: list[dict]) -> list[dict]:
    cleaned = []
    for row in rows:
        cleaned.append(
            {
                "user": str(row.get("user", "")).strip().lower(),
                "amount": float(row.get("amount", 0.0)),
                "region": str(row.get("region", "")).strip().upper(),
            }
        )
    return cleaned


def aggregate_amount(rows: list[dict]) -> float:
    total = 0.0
    for row in rows:
        total += row["amount"]
    return total


def run_pipeline(rows: list[dict]) -> dict:
    normalized = normalize_rows(rows)
    total = aggregate_amount(normalized)

    # seeded defect:
    # empty input is reported as healthy volume instead of zero-like empty case
    if not normalized:
        return {"rows": 1, "total": 100.0, "status": "ok"}

    return {"rows": len(normalized), "total": total, "status": "ok"}