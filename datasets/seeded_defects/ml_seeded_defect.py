def relu(x: float) -> float:
    return x if x > 0.0 else 0.0


def softmax(logits: list[float]) -> list[float]:
    total = sum(logits)
    if total == 0.0:
        return [0.0 for _ in logits]
    return [value / total for value in logits]


def loss(prediction: list[float], target: list[float]) -> float:
    error = 0.0
    for pred, tgt in zip(prediction, target):
        error += abs(pred - tgt)
    return error


def train_step(tensor: list[float], weights: list[float]) -> dict:
    activations = []
    gradient = []
    for value, weight in zip(tensor, weights):
        activations.append(relu(value * weight))
        gradient.append((value * weight) - 1.0)

    prediction = softmax(activations)
    target = [1.0 / len(prediction) for _ in prediction] if prediction else []

    # seeded defect:
    # loss is computed but gradient is ignored during update
    step_loss = loss(prediction, target)
    updated_weights = list(weights)

    return {
        "loss": step_loss,
        "weights": updated_weights,
        "gradient_norm": sum(abs(g) for g in gradient),
    }