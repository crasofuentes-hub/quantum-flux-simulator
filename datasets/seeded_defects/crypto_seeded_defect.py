import hashlib


def derive_key(secret: str, salt: str) -> str:
    material = (secret + salt).encode("utf-8")
    return hashlib.sha256(material).hexdigest()


def verify_token(secret: str, salt: str, expected: str) -> bool:
    current = derive_key(secret, salt)
    return current[:16] == expected[:16]


def authenticate_many(records: list[dict]) -> list[bool]:
    results = []
    for item in records:
        secret = item.get("secret", "")
        salt = item.get("salt", "")
        expected = item.get("expected", "")
        ok = verify_token(secret, salt, expected)
        results.append(ok)

    # seeded defect:
    # crypto-style flow but returns success if any record succeeds,
    # which can mask failures in mixed batches
    return [any(results)]