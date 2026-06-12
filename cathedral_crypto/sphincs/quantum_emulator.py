#!/usr/bin/env python3
# quantum_emulator.py - Emulador do cristal de tempo para testnet RBB Chain
import hashlib
import hmac
import secrets
import struct
import time
import threading
import json
from web3 import Web3
from web3.middleware import ExtraDataToPOAMiddleware
import sphincs_c13

# Configurações
TICK_INTERVAL_NS = 100_000_000          # 100 ns => 10 MHz
TICK_INTERVAL_SEC = TICK_INTERVAL_NS / 1e9
PRIVATE_KEY_HEX = "0x" + "11" * 32      # Chave privada mock (não usar em produção)
ORACLE_ADDRESS = "0x123..."             # Endereço do contrato QuantumTimestampOracle (a ser preenchido)
RPC_URL = "https://rpc.testnet.rbbchain.io"
CHAIN_ID = 12345

# Conecta à testnet
try:
    w3 = Web3(Web3.HTTPProvider(RPC_URL))
    w3.middleware_onion.inject(ExtraDataToPOAMiddleware, layer=0)
    connected = w3.is_connected()
except:
    connected = False

if connected:
    # Carrega o contrato oracle (ABI fornecido na seção anterior)
    try:
        with open("QuantumTimestampOracle.abi", "r") as f:
            oracle_abi = json.load(f)
        oracle = w3.eth.contract(address=ORACLE_ADDRESS, abi=oracle_abi)
        account = w3.eth.account.from_key(PRIVATE_KEY_HEX)
    except:
        oracle = None
        account = None
else:
    oracle = None
    account = None

# Estado interno
current_tick = 0
last_block_hash = b"\x00" * 32
lock = threading.Lock()

# Inicializa o signatário real (usando SPHINCS- C13 implementado)
signer = sphincs_c13.SPHINCS_C13(sk_seed=bytes.fromhex(PRIVATE_KEY_HEX[2:]))
PUBLIC_KEY_HEX = "0x" + signer.pk_root[:16].hex()

def mock_sphincs_sign(message: bytes) -> bytes:
    """
    Gera assinatura SPHINCS- C13 real.
    """
    sig, _, _ = signer.sign(message)
    return sig

def generate_tick():
    """Gera um novo tick, assina e atualiza o oracle."""
    global current_tick, last_block_hash
    while True:
        # Aguarda o próximo intervalo com dither quântico
        dither = secrets.randbelow(20) - 10      # +/- 10 ns
        sleep_sec = TICK_INTERVAL_SEC + (dither * 1e-9)
        time.sleep(sleep_sec)

        # Obtém o hash do bloco mais recente (simulado ou via web3)
        try:
            if connected:
                block = w3.eth.get_block('latest')
                block_hash = block['hash'].hex()
                last_block_hash = bytes.fromhex(block_hash[2:])
            else:
                last_block_hash = secrets.token_bytes(32)
        except:
            last_block_hash = secrets.token_bytes(32)  # fallback

        with lock:
            current_tick += 1
            message = struct.pack(">Q", current_tick) + last_block_hash
            signature = mock_sphincs_sign(message)

            if connected and oracle and account:
                # Envia transação para atualizar o oracle
                try:
                    tx = oracle.functions.updateTick(current_tick, signature).build_transaction({
                        'from': account.address,
                        'nonce': w3.eth.get_transaction_count(account.address),
                        'gas': 200_000,
                        'gasPrice': w3.eth.gas_price,
                        'chainId': CHAIN_ID
                    })
                    signed = account.sign_transaction(tx)
                    tx_hash = w3.eth.send_raw_transaction(signed.rawTransaction)
                    print(f"Tick {current_tick} enviado, tx: {tx_hash.hex()}")
                    # Aguarda confirmação (opcional)
                    w3.eth.wait_for_transaction_receipt(tx_hash, timeout=10)
                except Exception as e:
                    print(f"Erro ao enviar tx: {e}")
            else:
                print(f"Tick {current_tick} simulado localmente. Assinatura gerada ({len(signature)} bytes).")

def main():
    print(f"Iniciando emulador do cristal de tempo... PK Root: {PUBLIC_KEY_HEX}")
    generate_tick()

if __name__ == "__main__":
    main()
