def aes_round(block, key):
    out = []
    for i, value in enumerate(block):
        out.append(value ^ key[i % len(key)])
    return out

def rsa_transform(m, e, n):
    return pow(m, e, n)

def sha_mix(words):
    acc = 0
    for w in words:
        acc ^= w
    return acc