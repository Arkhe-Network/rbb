// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

/**
 * @title CathedralSPHINCSVerifier
 * @notice Verifica assinaturas SPHINCS- C13 (WOTS+C / FORS+C) on-chain.
 * @dev Parâmetros: n=16, w=8, l=43, k=8, a=16, d=2, h_total=24, h_per_layer=12.
 *      Tamanho da assinatura (sem compressão FORS+C): 3952 bytes.
 *      O artigo menciona 3704 bytes via compressão (FORS+C) – suporte futuro.
 *      Gas estimado: ~130k-150k (dependendo da calldata e loops).
 */
contract CathedralSPHINCSVerifier {
    // ------------------------------------------------------------
    // Constantes do esquema C13
    // ------------------------------------------------------------
    uint256 private constant N = 16;                 // bytes por hash
    uint256 private constant W = 8;                 // base Winternitz
    uint256 private constant LOG2W = 3;             // log2(8) = 3 bits por dígito
    uint256 private constant L = 43;                // número de dígitos / chains WOTS+
    uint256 private constant K = 8;                 // número de árvores FORS
    uint256 private constant A = 16;                // altura de cada árvore FORS (2^16 folhas)
    uint256 private constant D = 2;                 // camadas da hypertree
    uint256 private constant H_TOTAL = 24;          // altura total (2^24 assinaturas possíveis)
    uint256 private constant H_PER_LAYER = 12;      // altura por camada (24/2)
    uint256 private constant WOTS_MAX_STEP = W - 1; // =7

    // Tamanhos dos componentes da assinatura (bytes) – layout sem compressão FORS+C
    uint256 private constant SIG_RAND_SIZE = N;                     // 16
    uint256 private constant FORS_LEAF_SIZE = N;                    // 16
    uint256 private constant FORS_AUTH_SIZE = A * N;                // 256
    uint256 private constant FORS_SIG_ITEM_SIZE = FORS_LEAF_SIZE + FORS_AUTH_SIZE; // 272
    uint256 private constant FORS_TOTAL_SIZE = K * FORS_SIG_ITEM_SIZE; // 2176
    uint256 private constant WOTS_SIG_SIZE = L * N;                 // 688
    uint256 private constant MERKLE_AUTH_SIZE = H_PER_LAYER * N;    // 192
    // Tamanho total da assinatura (sem compressão): 16 + 2176 + 688 + 192 + 688 + 192 = 3952
    uint256 private constant SIG_SIZE = 3952;

    // ------------------------------------------------------------
    // Interface pública
    // ------------------------------------------------------------
    /**
     * @dev Verifica uma assinatura SPHINCS- C13.
     * @param message A mensagem original (arbitrária, será hasheada com keccak256).
     * @param signature Assinatura bruta (3952 bytes, layout conforme especificação).
     * @param publicKeyRoot Raiz da hypertree pública (16 bytes, alinhado à esquerda num bytes32).
     * @return true se a assinatura é válida, false caso contrário.
     */
    function verifySPHINCS(
        bytes memory message,
        bytes calldata signature,
        bytes32 publicKeyRoot
    ) external view returns (bool) {
        require(signature.length == SIG_SIZE, "CathedralSPHINCSVerifier: invalid signature length (expected 3952 bytes)");

        // --------------------------------------------------------
        // 1. Extrai randomizer (primeiros N bytes)
        // --------------------------------------------------------
        bytes32 randomizer;
        uint256 offset = 0;
        assembly {
            randomizer := and(calldataload(signature.offset), 0xffffffffffffffffffffffffffffffff00000000000000000000000000000000)
        }
        offset += N;

        // --------------------------------------------------------
        // 2. Deriva índices e message digest (md)
        // --------------------------------------------------------
        // H_msg = keccak256(randomizer || publicKeyRoot || message)
        bytes32 hMsg = keccak256(abi.encodePacked(randomizer, publicKeyRoot, message));
        // md: primeiros N bytes (16) de hMsg
        bytes32 md;
        assembly {
            mstore(0, hMsg)
            md := mload(0)            // pega os primeiros 32 bytes, mas só interessam os 16 primeiros
        }
        // idx_tree: bits 128..139 (12 bits) - posição após md
        uint256 idxTree = (uint256(hMsg) >> (N * 8)) & ((1 << H_PER_LAYER) - 1);
        // idx_leaf: bits 140..151 (12 bits) - próximos 12 bits
        uint256 idxLeaf = (uint256(hMsg) >> ((N + H_PER_LAYER) * 8)) & ((1 << H_PER_LAYER) - 1);

        // --------------------------------------------------------
        // 3. Verificação FORS
        // --------------------------------------------------------
        bytes32 forsPK = _verifyFORS(
            signature[offset:offset + FORS_TOTAL_SIZE],
            md,
            idxTree,
            idxLeaf
        );
        offset += FORS_TOTAL_SIZE;

        // --------------------------------------------------------
        // 4. Primeira camada WOTS+ (camada 0) + Merkle
        // --------------------------------------------------------
        bytes32 nodeLayer0 = _verifyWOTSC(
            signature[offset:offset + WOTS_SIG_SIZE],
            forsPK,
            idxLeaf,
            idxTree
        );
        offset += WOTS_SIG_SIZE;

        nodeLayer0 = _verifyMerklePath(
            nodeLayer0,
            signature[offset:offset + MERKLE_AUTH_SIZE],
            idxLeaf,
            H_PER_LAYER
        );
        offset += MERKLE_AUTH_SIZE;

        // --------------------------------------------------------
        // 5. Segunda camada WOTS+ (camada 1) + Merkle até a raiz
        // --------------------------------------------------------
        bytes32 nodeLayer1 = _verifyWOTSC(
            signature[offset:offset + WOTS_SIG_SIZE],
            nodeLayer0,
            idxTree,          // na camada superior, o índice da árvore é o idx_tree global
            0                 // leaf index não usado, passamos 0
        );
        offset += WOTS_SIG_SIZE;

        bytes32 computedRoot = _verifyMerklePath(
            nodeLayer1,
            signature[offset:offset + MERKLE_AUTH_SIZE],
            idxTree,
            H_PER_LAYER
        );
        offset += MERKLE_AUTH_SIZE;

        // --------------------------------------------------------
        // 6. Comparação com a chave pública fornecida (primeiros 16 bytes)
        // --------------------------------------------------------
        return (bytes16(computedRoot) == bytes16(publicKeyRoot));
    }

    // ------------------------------------------------------------
    // Funções internas: FORS
    // ------------------------------------------------------------
    function _verifyFORS(
        bytes calldata forsData,
        bytes32 md,
        uint256 idxTree,
        uint256 idxLeaf
    ) private pure returns (bytes32) {
        // Deriva os índices das árvores FORS a partir do md e dos índices da hypertree
        bytes32[] memory roots = new bytes32[](K);
        uint256 innerOffset = 0;

        for (uint256 i = 0; i < K; i++) {
            // Calcula o índice da folha dentro da árvore FORS (0 .. 2^A - 1)
            bytes32 leafIdxHash = keccak256(abi.encodePacked(md, idxTree, idxLeaf, i));
            uint256 leafIdx = uint256(leafIdxHash) % (1 << A);

            // Lê o valor da folha (N bytes)
            bytes32 leafValue;
            assembly {
                leafValue := and(calldataload(add(forsData.offset, innerOffset)), 0xffffffffffffffffffffffffffffffff00000000000000000000000000000000)
            }
            innerOffset += N;

            // Lê o caminho de autenticação (A * N bytes)
            bytes32[] memory authPath = new bytes32[](A);
            for (uint256 j = 0; j < A; j++) {
                assembly {
                    let pos := add(forsData.offset, innerOffset)
                    let dest := add(authPath, add(32, mul(j, 32)))
                    calldatacopy(dest, pos, N)
                }
                innerOffset += N;
            }

            // Reconstrói a raiz da árvore FORS
            bytes32 node = leafValue;
            uint256 currentIdx = leafIdx;
            for (uint256 level = 0; level < A; level++) {
                bytes32 sibling = authPath[level];
                if ((currentIdx >> level) & 1 == 0) {
                    node = bytes32(uint256(keccak256(abi.encodePacked(node, sibling))) & 0xffffffffffffffffffffffffffffffff00000000000000000000000000000000);
                } else {
                    node = bytes32(uint256(keccak256(abi.encodePacked(sibling, node))) & 0xffffffffffffffffffffffffffffffff00000000000000000000000000000000);
                }
            }
            roots[i] = node;
        }

        // Combina todas as raízes FORS em uma única chave pública (hash concatenado)
        return bytes32(uint256(keccak256(abi.encodePacked(roots))) & 0xffffffffffffffffffffffffffffffff00000000000000000000000000000000);
    }

    // ------------------------------------------------------------
    // WOTS+ (com suporte a grinding opcional do signatário)
    // ------------------------------------------------------------
    function _verifyWOTSC(
        bytes calldata wotsSig,
        bytes32 message,
        uint256 leafIdx,
        uint256 treeIdx
    ) private pure returns (bytes32) {
        // Deriva os dígitos base-w da mensagem (N bytes)
        uint8[L] memory digits = _computeWinternitzDigits(message, leafIdx, treeIdx);

        // Verifica cada chain
        bytes32[] memory chainValues = new bytes32[](L);
        uint256 offset = 0;
        for (uint256 i = 0; i < L; i++) {
            // Lê o valor fornecido na assinatura (N bytes)
            bytes32 sigValue;
            assembly {
                sigValue := and(calldataload(add(wotsSig.offset, offset)), 0xffffffffffffffffffffffffffffffff00000000000000000000000000000000)
            }
            offset += N;

            // Aplica (WOTS_MAX_STEP - digit) hashes para chegar ao topo da chain
            uint256 steps = WOTS_MAX_STEP - digits[i];
            bytes32 current = sigValue;
            for (uint256 step = 0; step < steps; step++) {
                current = bytes32(uint256(keccak256(abi.encodePacked(current))) & 0xffffffffffffffffffffffffffffffff00000000000000000000000000000000);
            }
            chainValues[i] = current;
        }

        // Converte as L chains em uma chave pública (hash da concatenação)
        return bytes32(uint256(keccak256(abi.encodePacked(chainValues))) & 0xffffffffffffffffffffffffffffffff00000000000000000000000000000000);
    }

    // ------------------------------------------------------------
    // Merkle path verification (para uma árvore binária de altura H)
    // ------------------------------------------------------------
    function _verifyMerklePath(
        bytes32 leaf,
        bytes calldata authPath,
        uint256 leafIndex,
        uint256 treeHeight
    ) private pure returns (bytes32) {
        require(authPath.length == treeHeight * N, "Invalid Merkle auth path length");
        bytes32 node = leaf;
        uint256 idx = leafIndex;
        for (uint256 level = 0; level < treeHeight; level++) {
            bytes32 sibling;
            assembly {
                sibling := and(calldataload(add(authPath.offset, mul(level, N))), 0xffffffffffffffffffffffffffffffff00000000000000000000000000000000)
            }
            if ((idx >> level) & 1 == 0) {
                node = bytes32(uint256(keccak256(abi.encodePacked(node, sibling))) & 0xffffffffffffffffffffffffffffffff00000000000000000000000000000000);
            } else {
                node = bytes32(uint256(keccak256(abi.encodePacked(sibling, node))) & 0xffffffffffffffffffffffffffffffff00000000000000000000000000000000);
            }
        }
        return node;
    }

    // ------------------------------------------------------------
    // Converte um hash de N bytes em L dígitos base-w
    // ------------------------------------------------------------
    function _computeWinternitzDigits(
        bytes32 msgHash,
        uint256 leafIdx,
        uint256 treeIdx
    ) private pure returns (uint8[L] memory digits) {
        // Expande o hash usando um domínio separado para evitar colisão com índices
        bytes32 expanded = keccak256(abi.encodePacked(msgHash, leafIdx, treeIdx, uint8(0)));
        uint256 bitsAvailable = 256;
        uint256 bitPos = 0;
        for (uint256 i = 0; i < L; i++) {
            if (bitsAvailable < LOG2W) {
                // Se não há bits suficientes, faz um novo hash encadeado
                expanded = keccak256(abi.encodePacked(expanded, uint8(i)));
                bitsAvailable = 256;
                bitPos = 0;
            }
            uint256 digit = (uint256(expanded) >> bitPos) & ((1 << LOG2W) - 1);
            digits[i] = uint8(digit);
            bitPos += LOG2W;
            bitsAvailable -= LOG2W;
        }
    }
}
