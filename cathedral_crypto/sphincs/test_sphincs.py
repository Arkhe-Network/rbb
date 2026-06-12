import sphincs_c13
from web3 import Web3

def test_sphincs_signature_length():
    signer = sphincs_c13.SPHINCS_C13(sk_seed=b'\x42'*16)
    sig, pk, msg = signer.sign(b"Test Message")
    assert len(sig) == sphincs_c13.SIG_SIZE
    assert len(pk) == 32
