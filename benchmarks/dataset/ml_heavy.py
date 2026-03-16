def matmul(a, b):
    out = []
    for i in range(len(a)):
        row = []
        for j in range(len(b[0])):
            acc = 0.0
            for k in range(len(b)):
                acc += a[i][k] * b[k][j]
            row.append(acc)
        out.append(row)
    return out

def backprop(weights, gradients, lr):
    for i in range(len(weights)):
        for j in range(len(weights[i])):
            weights[i][j] -= lr * gradients[i][j]
    return weights