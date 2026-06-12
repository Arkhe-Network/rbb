import os
import hashlib
from web3 import Web3, EthereumTesterProvider
import solcx

from enter_cathedral_bridge.src.sphincs_wrapper import RealSPHINCS

solcx.install_solc('0.8.24')

def test_batch_anchoring():
    w3 = Web3(EthereumTesterProvider())

    verifier_source = """
    // SPDX-License-Identifier: Apache-2.0
    pragma solidity ^0.8.24;

    contract CathedralSPHINCSVerifierYul {
        function verifySPHINCS(bytes calldata message, bytes calldata signature, bytes32 publicKeyRoot) external pure returns (bool) {
            if (signature.length != 3952) return false;
            return true;
        }
    }
    """

    oracle_source = """
    // SPDX-License-Identifier: Apache-2.0
    pragma solidity ^0.8.24;

    contract QuantumTimestampOracleMock {
        uint64 public latestTick = 1;

        function getTimestamp() external view returns (uint64 tick, bytes memory signature) {
            return (latestTick, new bytes(0));
        }

        function publicKeyRoot() external pure returns (bytes32) {
            return keccak256("dummy_root");
        }

        function incrementTick() external {
            latestTick += 1;
        }
    }
    """

    with open('enter_cathedral_bridge/contracts/EnterEvidenceAnchor.sol', 'r') as f:
        anchor_source = f.read()

    compiled_sol = solcx.compile_standard(
        {
            "language": "Solidity",
            "sources": {
                "CathedralSPHINCSVerifierYul.sol": {"content": verifier_source},
                "QuantumTimestampOracleMock.sol": {"content": oracle_source},
                "EnterEvidenceAnchor.sol": {"content": anchor_source}
            },
            "settings": {
                "outputSelection": {
                    "*": {
                        "*": ["abi", "metadata", "evm.bytecode", "evm.sourceMap"]
                    }
                }
            }
        },
        solc_version='0.8.24',
    )

    verifier_interface = compiled_sol['contracts']['CathedralSPHINCSVerifierYul.sol']['CathedralSPHINCSVerifierYul']
    oracle_interface = compiled_sol['contracts']['QuantumTimestampOracleMock.sol']['QuantumTimestampOracleMock']
    anchor_interface = compiled_sol['contracts']['EnterEvidenceAnchor.sol']['EnterEvidenceAnchor']

    w3.eth.default_account = w3.eth.accounts[0]
    authorized_submitter = w3.eth.accounts[2]

    # Deploy Verifier
    Verifier = w3.eth.contract(abi=verifier_interface['abi'], bytecode=verifier_interface['evm']['bytecode']['object'])
    tx_hash = Verifier.constructor().transact()
    tx_receipt = w3.eth.wait_for_transaction_receipt(tx_hash)
    verifier_address = tx_receipt.contractAddress

    # Deploy Oracle
    Oracle = w3.eth.contract(abi=oracle_interface['abi'], bytecode=oracle_interface['evm']['bytecode']['object'])
    tx_hash = Oracle.constructor().transact()
    tx_receipt = w3.eth.wait_for_transaction_receipt(tx_hash)
    oracle_address = tx_receipt.contractAddress

    # Deploy Anchor
    Anchor = w3.eth.contract(abi=anchor_interface['abi'], bytecode=anchor_interface['evm']['bytecode']['object'])
    tx_hash = Anchor.constructor(verifier_address, oracle_address, authorized_submitter).transact()
    tx_receipt = w3.eth.wait_for_transaction_receipt(tx_hash)
    anchor_address = tx_receipt.contractAddress

    contract = w3.eth.contract(address=anchor_address, abi=anchor_interface['abi'])
    oracle = w3.eth.contract(address=oracle_address, abi=oracle_interface['abi'])

    agent = RealSPHINCS(seed=os.urandom(16))

    # 1. Coletar 50 evidências (simuladas)
    evidences = [f"Evidence {i}".encode() for i in range(50)]
    leaf_hashes = [hashlib.sha3_256(e).digest() for e in evidences]

    def merkle_root(leaves):
        level = leaves
        while len(level) > 1:
            level = [hashlib.sha3_256(level[i] + level[i+1]).digest() if i+1<len(level) else level[i] for i in range(0,len(level),2)]
        return level[0]

    root_hash = merkle_root(leaf_hashes)

    oracle.functions.incrementTick().transact()

    # 2. Obter tick quântico
    tick, _ = oracle.functions.getTimestamp().call()
    block_hash = w3.eth.get_block('latest')['hash'].hex()

    # 3. Assinar mensagem
    # Use b'' instead of trying to format block_hash string because block_hash might have '0x' prefix
    block_hash_bytes = w3.eth.get_block('latest')['hash']
    msg = root_hash + tick.to_bytes(8, 'big') + block_hash_bytes
    signature = agent.sign(msg)

    # 4. Enviar transação
    tx_hash = contract.functions.anchorBatch(root_hash, tick, block_hash_bytes, signature).transact({'from': authorized_submitter})
    tx_receipt = w3.eth.wait_for_transaction_receipt(tx_hash)

    print("Batch anchored successfully!")

    # Verify event was emitted
    logs = contract.events.BatchAnchored().process_receipt(tx_receipt)
    assert len(logs) == 1
    assert logs[0].args.rootHash == root_hash
    assert logs[0].args.tick == tick
    assert logs[0].args.submitter == authorized_submitter

if __name__ == "__main__":
    test_batch_anchoring()
