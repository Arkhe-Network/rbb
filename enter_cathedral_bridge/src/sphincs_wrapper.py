import os
import hashlib

class RealSPHINCS:
    def __init__(self, seed=None):
        if seed is None:
            self.seed = os.urandom(16)
        else:
            self.seed = seed

    def sign(self, message: bytes) -> bytes:
        # Mocking SPHINCS+ signature with 3952 bytes according to Cathedral specifications
        # For testing, we ensure the first 32 bytes are the keccak256 hash of the message
        from eth_utils import keccak
        msg_hash = keccak(message)
        # Pad with 0s to reach 3952 bytes
        signature = msg_hash + b'\x00' * (3952 - 32)
        return signature
