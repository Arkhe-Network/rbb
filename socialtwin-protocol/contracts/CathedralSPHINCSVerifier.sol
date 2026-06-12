// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.24;

/**
 * @title CathedralSPHINCSVerifier
 * @dev Verifies SPHINCS- C13 (WOTS+C / FORS+C) post-quantum signatures on-chain.
 *      Parameters: n=16, w=8, l=43, k=8, a=16, d=2, h=24.
 *      Public key is a Merkle root (16 bytes, stored in bytes32).
 *      Signature size: 3704 bytes.
 */
contract CathedralSPHINCSVerifier {
    // ------------------------------------------------------------
    // Constants (C13)
    // ------------------------------------------------------------
    uint256 internal constant N = 16;            // hash output bytes
    uint256 internal constant W = 8;             // Winternitz base
    uint256 internal constant L = 43;            // number of WOTS+ chains (16 * 8 / 3 ≈ 43)
    uint256 internal constant K = 8;             // FORS trees
    uint256 internal constant A = 16;            // FORS tree height (2^A leaves)
    uint256 internal constant D = 2;             // hypertree layers
    uint256 internal constant H_TOTAL = 24;      // total hypertree height
    uint256 internal constant H_PER_LAYER = 12;  // H_TOTAL / D

    // Sizes
    uint256 internal constant SIG_SIZE = 3952;

    // ------------------------------------------------------------
    // Core verification
    // ------------------------------------------------------------
    /**
     * @dev Verifies a SPHINCS- C13 signature.
     * @param message Hash of the message (32 bytes)
     * @param signature Raw signature bytes (exactly 3704 bytes)
     * @param publicKeyRoot The root of the hypertree (16 bytes, left-aligned in bytes32)
     * @return true if signature is valid, false otherwise
     */
    function verifySPHINCS(
        bytes32 message,
        bytes calldata signature,
        bytes32 publicKeyRoot
    ) external pure returns (bool) {
        if (signature.length != SIG_SIZE) {
            return false;
        }

        uint256 offset = 0;

        // 1. Randomizer (N bytes)
        bytes32 randomizer;
        assembly {
            randomizer := and(calldataload(signature.offset), 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF00000000000000000000000000000000)
        }
        offset += N;

        // 2. Message digest and indices
        bytes32 md = keccak256(abi.encodePacked(randomizer, message));

        // Deriving indices: idx_tree and idx_leaf
        // In full SPHINCS+, these come from hashing the message with the public seed and R.
        // For our simplified Keccak-based test verifier, we derive them directly from the digest.
        uint64 idx_tree = uint64((uint256(md) >> 128) % (1 << (H_TOTAL - H_PER_LAYER))); // Layer 0 tree index
        uint32 idx_leaf = uint32(uint256(md) % (1 << H_PER_LAYER)); // Leaf index within the tree

        // 3. FORS signatures (K * (N + A*N))
        uint256 forsTotalSize = K * (N + A * N);
        bytes32 forsPK = _reconstructFORSPublicKey(
            signature[offset:offset + forsTotalSize],
            md
        );
        offset += forsTotalSize;

        // 4. First WOTS+ layer (bottom layer)
        uint256 wotsSize = L * N;
        bytes32 layer0Node = _verifyWOTSC(
            signature[offset:offset + wotsSize],
            forsPK // The "message" being signed by the bottom WOTS+ is the FORS PK
        );
        offset += wotsSize;

        // 5. First Merkle path
        uint256 merkleAuthSizeLayer0 = H_PER_LAYER * N;
        layer0Node = _verifyMerklePath(
            layer0Node,
            signature[offset:offset + merkleAuthSizeLayer0],
            idx_leaf
        );
        offset += merkleAuthSizeLayer0;

        // 6. Second WOTS+ layer (top layer)
        bytes32 layer1Node = _verifyWOTSC(
            signature[offset:offset + wotsSize],
            layer0Node // The "message" being signed by the top WOTS+ is the root of layer 0
        );
        offset += wotsSize;

        // 7. Second Merkle path to public key root
        uint256 merkleAuthSizeLayer1 = H_PER_LAYER * N;
        bytes32 computedRoot = _verifyMerklePath(
            layer1Node,
            signature[offset:offset + merkleAuthSizeLayer1],
            idx_tree
        );
        offset += merkleAuthSizeLayer1;

        // Ensure computed root matches provided public key (first 16 bytes)
        bytes32 mask = bytes32(uint256(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF) << 128);
        return (computedRoot & mask) == (publicKeyRoot & mask);
    }

    // ------------------------------------------------------------
    // FORS reconstruction
    // ------------------------------------------------------------
    function _reconstructFORSPublicKey(
        bytes calldata forsData,
        bytes32 md
    ) internal pure returns (bytes32) {
        bytes32[] memory forsRoots = new bytes32[](K);
        uint256 offset = 0;

        for (uint256 i = 0; i < K; i++) {
            // Leaf value
            bytes32 leaf;
            assembly {
                leaf := and(calldataload(add(forsData.offset, offset)), 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF00000000000000000000000000000000)
            }
            offset += N;

            // Auth path
            bytes32[] memory authPath = new bytes32[](A);
            for (uint256 j = 0; j < A; j++) {
                bytes32 node;
                assembly {
                    node := and(calldataload(add(forsData.offset, offset)), 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF00000000000000000000000000000000)
                }
                authPath[j] = node;
                offset += N;
            }

            // Reconstruct the root of this FORS tree
            // Leaf index derivation: uses part of the message digest
            uint256 leafIdx = uint256(keccak256(abi.encodePacked(md, i))) % (1 << A);

            bytes32 root = leaf;
            for (uint256 j = 0; j < A; j++) {
                if ((leafIdx >> j) & 1 == 0) {
                    root = keccak256(abi.encodePacked(root, authPath[j]));
                } else {
                    root = keccak256(abi.encodePacked(authPath[j], root));
                }
            }
            forsRoots[i] = root;
        }

        return keccak256(abi.encodePacked(forsRoots));
    }

    // ------------------------------------------------------------
    // WOTS+C verification
    // ------------------------------------------------------------
    function _verifyWOTSC(
        bytes calldata wotsSig,
        bytes32 message
    ) internal pure returns (bytes32) {
        // Simplified: The generator applies the hashing logic and places the final values in the signature
        // so we just reconstruct the public key.
        // A complete implementation would derive base-w digits and apply remaining hashes.

        // Implement true base-w (w=8) digit derivation for N=16 bytes
        // 16 bytes = 128 bits. With w=8, we consume 3 bits per digit.
        // L = 43 digits.
        uint8[] memory digits = new uint8[](L);
        uint256 bits = 0;
        uint256 total = 0;
        uint256 inIdx = 0;

        for (uint256 i = 0; i < L; i++) {
            if (bits < 3) {
                // message is bytes32, but we only use the first 16 bytes.
                // bytes32 is big-endian, so the first 16 bytes are the highest 128 bits.
                // message[inIdx] gives the byte.
                total = (total << 8) | uint8(message[inIdx]);
                inIdx++;
                bits += 8;
            }
            bits -= 3;
            digits[i] = uint8((total >> bits) & 7); // 7 is W - 1
        }

        bytes32[] memory chains = new bytes32[](L);
        uint256 offset = 0;
        for (uint256 i = 0; i < L; i++) {
            bytes32 chainVal;
            assembly {
                chainVal := and(calldataload(add(wotsSig.offset, offset)), 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF00000000000000000000000000000000)
            }

            uint8 digit = digits[i];

            // Apply remaining hashes
            for (uint8 j = digit; j < W - 1; j++) {
                chainVal = keccak256(abi.encodePacked(chainVal));
            }

            chains[i] = chainVal;
            offset += N;
        }

        return keccak256(abi.encodePacked(chains));
    }

    // ------------------------------------------------------------
    // Merkle path verification
    // ------------------------------------------------------------
    function _verifyMerklePath(
        bytes32 leaf,
        bytes calldata authPath,
        uint256 leafIndex
    ) internal pure returns (bytes32) {
        bytes32 node = leaf;
        uint256 idx = leafIndex;
        uint256 pathLen = authPath.length / N;

        for (uint256 i = 0; i < pathLen; i++) {
            bytes32 sibling;
            assembly {
                sibling := and(calldataload(add(authPath.offset, mul(i, N))), 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF00000000000000000000000000000000)
            }

            if ((idx >> i) & 1 == 0) {
                node = keccak256(abi.encodePacked(node, sibling));
            } else {
                node = keccak256(abi.encodePacked(sibling, node));
            }
        }
        return node;
    }
}
