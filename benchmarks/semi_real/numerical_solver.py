def integrate(initial: float, rate: float, steps: int, dt: float) -> list[float]:
    values = [initial]
    current = initial
    for _ in range(steps):
        current = current + rate * dt
        values.append(current)
    return values


def solve(values: list[float]) -> float:
    if not values:
        return 0.0
    total = 0.0
    for value in values:
        total += value
    return total / len(values)


def rk4_like_step(x: float, velocity: float, dt: float) -> float:
    k1 = velocity
    k2 = velocity + 0.5 * dt
    k3 = velocity + 0.5 * dt
    k4 = velocity + dt
    return x + (dt / 6.0) * (k1 + 2.0 * k2 + 2.0 * k3 + k4)


def run_simulation() -> dict:
    trajectory = integrate(initial=1.0, rate=0.25, steps=32, dt=0.1)
    smoothed = []
    for value in trajectory:
        smoothed.append(rk4_like_step(value, 0.15, 0.05))
    return {
        "mean": solve(smoothed),
        "max": max(smoothed),
        "min": min(smoothed),
        "samples": len(smoothed),
    }