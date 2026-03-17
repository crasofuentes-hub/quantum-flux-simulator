def relu(x: float) -> float:
    return x if x > 0.0 else 0.0


def softmax(logits: list[float]) -> list[float]:
    total = sum(logits) if logits else 1.0
    if total == 0.0:
        return [0.0 for _ in logits]
    return [value / total for value in logits]


def loss(prediction: list[float], target: list[float]) -> float:
    error = 0.0
    for pred, tgt in zip(prediction, target):
        error += abs(pred - tgt)
    return error


def optimizer(weights: list[float], gradient: list[float], lr: float) -> list[float]:
    updated = []
    for weight, grad in zip(weights, gradient):
        updated.append(weight - lr * grad)
    return updated


def train_epoch(tensor: list[float], weights: list[float]) -> dict:
    gradient = []
    activations = []
    for value, weight in zip(tensor, weights):
        activations.append(relu(value * weight))
        gradient.append((value * weight) - 1.0)

    prediction = softmax(activations)
    target = [1.0 / len(prediction) for _ in prediction] if prediction else []
    epoch_loss = loss(prediction, target)
    new_weights = optimizer(weights, gradient, 0.01)

    return {
        "loss": epoch_loss,
        "weights": new_weights,
        "gradient_norm": sum(abs(g) for g in gradient),
    }