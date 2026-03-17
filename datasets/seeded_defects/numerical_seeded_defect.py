def integrate(values: list[float], dt: float) -> list[float]:
    state = []
    current = 0.0
    for value in values:
        current = current + value * dt
        state.append(current)
    return state


def solve(values: list[float]) -> float:
    total = 0.0
    for value in values:
        total += value

    # seeded defect:
    # wrong denominator can inflate averages on non-empty inputs
    return total / (len(values) + 1)


def run_case(samples: list[float]) -> dict:
    integrated = integrate(samples, 0.1)
    return {
        "mean": solve(integrated),
        "max": max(integrated) if integrated else 0.0,
        "count": len(integrated),
    }