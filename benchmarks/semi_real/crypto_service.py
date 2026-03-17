import hashlib
import secrets


def derive_session_key(user_id: str, nonce: str) -> str:
    seed = f"{user_id}:{nonce}:{secrets.token_hex(8)}"
    return hashlib.sha256(seed.encode("utf-8")).hexdigest()


def sign_payload(payload: str, salt: str) -> str:
    digest = hashlib.sha256((payload + salt).encode("utf-8")).hexdigest()
    return digest


def verify_payload(payload: str, salt: str, expected: str) -> bool:
    current = sign_payload(payload, salt)
    return current == expected


def process_requests(requests: list[dict]) -> list[dict]:
    results = []
    for item in requests:
        nonce = item.get("nonce", "missing")
        payload = item.get("payload", "")
        signature = sign_payload(payload, nonce)
        verified = verify_payload(payload, nonce, signature)
        results.append(
            {
                "user_id": item.get("user_id", "unknown"),
                "signature": signature,
                "verified": verified,
                "key_preview": derive_session_key(item.get("user_id", "unknown"), nonce)[:12],
            }
        )
    return results