import os
import math
from web3 import Web3

N = 16
W = 8
LOG2W = 3
L = 43
K = 8
A = 16
D = 2
H_TOTAL = 24
H_PER_LAYER = 12
WOTS_MAX_STEP = W - 1

SIG_SIZE = 3952

def keccak256(data):
    return Web3.keccak(bytes(data))[:N] + bytes(32 - N)

def keccak256_full(data):
    return Web3.keccak(bytes(data))

def _compute_winternitz_digits(msg_hash, leaf_idx, tree_idx):
    encoded = msg_hash + leaf_idx.to_bytes(32, 'big') + tree_idx.to_bytes(32, 'big') + bytes([0])
    expanded = keccak256_full(encoded)

    digits = []
    bits_available = 256
    bit_pos = 0
    for i in range(L):
        if bits_available < LOG2W:
            expanded = keccak256_full(expanded + bytes([i]))
            bits_available = 256
            bit_pos = 0

        digit = (int.from_bytes(expanded, 'big') >> bit_pos) & ((1 << LOG2W) - 1)
        digits.append(digit)
        bit_pos += LOG2W
        bits_available -= LOG2W

    return digits

def gen_wots_sk(sk_seed, leaf_idx, tree_idx, chain_idx):
    return keccak256_full(sk_seed + leaf_idx.to_bytes(32, 'big') + tree_idx.to_bytes(32, 'big') + chain_idx.to_bytes(32, 'big'))[:N] + bytes(32 - N)

def wots_pk_from_sk(sk_seed, leaf_idx, tree_idx):
    chain_values = []
    for i in range(L):
        current = gen_wots_sk(sk_seed, leaf_idx, tree_idx, i)
        for _ in range(WOTS_MAX_STEP):
            current = keccak256(current)
        chain_values.append(current)
    return keccak256(b''.join(chain_values))

def compute_merkle_tree(sk_seed, tree_idx, height):
    num_leaves = 1 << height
    leaves = [wots_pk_from_sk(sk_seed, i, tree_idx) for i in range(num_leaves)]

    tree = [leaves]
    for level in range(height):
        current_layer = tree[-1]
        next_layer = []
        for i in range(0, len(current_layer), 2):
            left = current_layer[i]
            right = current_layer[i+1]
            next_layer.append(keccak256(left + right))
        tree.append(next_layer)
    return tree

class SPHINCS_C13:
    def __init__(self, sk_seed=None):
        self.sk_seed = sk_seed or os.urandom(N)
        self.top_tree = compute_merkle_tree(self.sk_seed, 0, H_PER_LAYER)
        self.pk_root = self.top_tree[-1][0]

    def sign(self, message):
        randomizer = os.urandom(N)
        randomizer_32 = randomizer + bytes(32 - N)

        encoded_hmsg = randomizer_32 + self.pk_root + message
        hMsg = keccak256_full(encoded_hmsg)
        md = hMsg

        hMsg_int = int.from_bytes(hMsg, 'big')
        idxTree = (hMsg_int >> (N * 8)) & ((1 << H_PER_LAYER) - 1)
        idxLeaf = (hMsg_int >> ((N + H_PER_LAYER) * 8)) & ((1 << H_PER_LAYER) - 1)

        signature = bytearray()
        signature.extend(randomizer)

        # FORS
        fors_data = bytearray()
        fors_roots = []
        for i in range(K):
            encoded_leafIdxHash = md + idxTree.to_bytes(32, 'big') + idxLeaf.to_bytes(32, 'big') + i.to_bytes(32, 'big')
            leafIdxHash = keccak256_full(encoded_leafIdxHash)
            leafIdx = int.from_bytes(leafIdxHash, 'big') % (1 << A)

            def gen_fors_sk(sk_seed, tree_idx, leaf_idx, i, j):
                return keccak256_full(sk_seed + tree_idx.to_bytes(32, 'big') + leaf_idx.to_bytes(32, 'big') + b'fors' + i.to_bytes(32, 'big') + j.to_bytes(32, 'big'))[:N] + bytes(32 - N)

            def compute_fors_node(level, idx):
                if level == 0:
                    return gen_fors_sk(self.sk_seed, idxTree, idxLeaf, i, idx)
                else:
                    left = compute_fors_node(level-1, 2*idx)
                    right = compute_fors_node(level-1, 2*idx+1)
                    return keccak256(left + right)

            leafValue = compute_fors_node(0, leafIdx)
            fors_data.extend(leafValue[:N])

            authPath = []
            for level in range(A):
                sibling_idx = (leafIdx >> level) ^ 1
                def compute_fors_sub(l, idx):
                    if l == 0:
                        return gen_fors_sk(self.sk_seed, idxTree, idxLeaf, i, idx)
                    left = compute_fors_sub(l-1, 2*idx)
                    right = compute_fors_sub(l-1, 2*idx+1)
                    return keccak256(left + right)

                sibling = compute_fors_sub(level, sibling_idx)
                authPath.append(sibling)
                fors_data.extend(sibling[:N])

            node = leafValue
            currentIdx = leafIdx
            for level in range(A):
                sibling_32 = authPath[level]
                if (currentIdx >> level) & 1 == 0:
                    node = keccak256(node + sibling_32)
                else:
                    node = keccak256(sibling_32 + node)
            fors_roots.append(node)

        fors_pk = keccak256(b''.join(fors_roots))
        signature.extend(fors_data)

        # WOTS+ Layer 0
        wots_sig0 = bytearray()
        digits0 = _compute_winternitz_digits(fors_pk, idxLeaf, idxTree)
        chain_values0 = []
        for i in range(L):
            sk = gen_wots_sk(self.sk_seed, idxLeaf, idxTree, i)
            current = sk
            for _ in range(digits0[i]):
                current = keccak256(current)
            wots_sig0.extend(current[:N])

            for _ in range(WOTS_MAX_STEP - digits0[i]):
                current = keccak256(current)
            chain_values0.append(current)

        nodeLayer0 = keccak256(b''.join(chain_values0))
        signature.extend(wots_sig0)

        # Merkle Layer 0
        tree0 = compute_merkle_tree(self.sk_seed, idxTree, H_PER_LAYER)
        merkle0 = bytearray()
        idx = idxLeaf & ((1 << H_PER_LAYER) - 1)
        for level in range(H_PER_LAYER):
            sibling_idx = idx ^ 1
            sibling = tree0[level][sibling_idx]
            merkle0.extend(sibling[:N])
            idx >>= 1

        signature.extend(merkle0)

        # WOTS+ Layer 1
        wots_sig1 = bytearray()
        digits1 = _compute_winternitz_digits(tree0[-1][0], idxTree, 0)
        chain_values1 = []
        for i in range(L):
            sk = gen_wots_sk(self.sk_seed, idxTree, 0, i)
            current = sk
            for _ in range(digits1[i]):
                current = keccak256(current)
            wots_sig1.extend(current[:N])

            for _ in range(WOTS_MAX_STEP - digits1[i]):
                current = keccak256(current)
            chain_values1.append(current)

        nodeLayer1 = keccak256(b''.join(chain_values1))
        signature.extend(wots_sig1)

        # Merkle Layer 1
        merkle1 = bytearray()
        idx = idxTree & ((1 << H_PER_LAYER) - 1)
        node = nodeLayer1
        for level in range(H_PER_LAYER):
            sibling_idx = idx ^ 1
            sibling = self.top_tree[level][sibling_idx]
            merkle1.extend(sibling[:N])
            if (idx & 1) == 0:
                node = keccak256(node + sibling)
            else:
                node = keccak256(sibling + node)
            idx >>= 1
        signature.extend(merkle1)

        computedRoot = node
        print(f"wots1 from pk: {wots_pk_from_sk(self.sk_seed, idxTree, 0).hex()}")
        print(f"wots1 from sig: {nodeLayer1.hex()}")
        print(f"computedRoot: {computedRoot.hex()}")
        print(f"expectedPk: {self.pk_root.hex()}")
        return signature, self.pk_root, message

if __name__ == '__main__':
    signer = SPHINCS_C13(sk_seed=b'\x42'*16)
    sig, pk, msg = signer.sign(b"Test Message")
    with open('test_data.py', 'w') as f:
        f.write(f"sig = {repr(sig)}\n")
        f.write(f"pk = {repr(pk)}\n")
        f.write(f"msg = {repr(msg)}\n")
        print("Signature saved.")
