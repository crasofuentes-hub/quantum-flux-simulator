def integrate_step(grid, dt):
    next_grid = []
    for row in grid:
        new_row = []
        for value in row:
            new_row.append(value + dt * 0.1)
        next_grid.append(new_row)
    return next_grid

def solve_field(grid, steps, dt):
    current = grid
    for _ in range(steps):
        current = integrate_step(current, dt)
    return current