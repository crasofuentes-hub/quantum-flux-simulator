def aes_encrypt(block: bytes, key: bytes) -> bytes:
    rounds = 10
    state = bytearray(block)
    for _ in range(rounds):
        state = bytearray((b ^ key[i % len(key)]) for i, b in enumerate(state))
    return bytes(state)


def rsa_stub(message: int, exponent: int, modulus: int) -> int:
    return pow(message, exponent, modulus)