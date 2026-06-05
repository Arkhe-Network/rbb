#!/usr/bin/env python3
"""
Kleros Theosis Prometheus Exporter
Connects to the PNKTheosisOracle smart contract via Web3 and exposes juror metrics for Prometheus.
"""

import time
import os
import json
from prometheus_client import start_http_server, Gauge
from web3 import Web3

# Define Prometheus metrics
JUROR_PNK_BALANCE = Gauge('kleros_juror_pnk_balance', 'PNK Balance of the Juror', ['juror_address'])
JUROR_THEOSIS_LEVEL = Gauge('kleros_juror_theosis_level', 'Theosis Level of the Juror (0-10000)', ['juror_address'])
VOTING_WEIGHT = Gauge('kleros_juror_voting_weight', 'Calculated Voting Weight (PNK * Theosis Multiplier)', ['juror_address'])

# Setup Web3
RPC_URL = os.environ.get("RPC_URL", "http://localhost:8545")
CONTRACT_ADDRESS = os.environ.get("ORACLE_CONTRACT_ADDRESS", "0x0000000000000000000000000000000000000000")

# Minimal ABI for PNKTheosisOracle
ORACLE_ABI = json.loads('''[
    {
      "inputs": [
        {
          "internalType": "address",
          "name": "_juror",
          "type": "address"
        }
      ],
      "name": "getJurorMetrics",
      "outputs": [
        {
          "internalType": "uint256",
          "name": "",
          "type": "uint256"
        },
        {
          "internalType": "uint256",
          "name": "",
          "type": "uint256"
        }
      ],
      "stateMutability": "view",
      "type": "function"
    }
]''')

w3 = Web3(Web3.HTTPProvider(RPC_URL))
contract = w3.eth.contract(address=w3.to_checksum_address(CONTRACT_ADDRESS), abi=ORACLE_ABI)

# Set of active jurors to monitor (in a real scenario, this could be populated dynamically from events)
JURORS = [
    "0x1234567890123456789012345678901234567890",
    "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd",
    "0x9999999999999999999999999999999999999999"
]

def fetch_metrics():
    """
    Fetches data from Web3/Smart Contract.
    Uses web3.py to call getJurorMetrics() on the PNKTheosisOracle contract.
    """
    if not w3.is_connected():
        print(f"Failed to connect to Web3 RPC: {RPC_URL}")
        return

    for juror_address in JURORS:
        try:
            checksum_address = w3.to_checksum_address(juror_address)
            # Call the oracle
            pnk_balance, theosis_level = contract.functions.getJurorMetrics(checksum_address).call()

            # Update gauges
            JUROR_PNK_BALANCE.labels(juror_address=juror_address).set(pnk_balance)
            JUROR_THEOSIS_LEVEL.labels(juror_address=juror_address).set(theosis_level)

            # Calculate derived voting weight locally (for metrics)
            # Weight = PNK * (1 + Theosis / 10000)
            theosis_multiplier = 10000 + theosis_level
            voting_weight = (pnk_balance * theosis_multiplier) / 10000

            VOTING_WEIGHT.labels(juror_address=juror_address).set(voting_weight)
        except Exception as e:
            print(f"Error fetching metrics for juror {juror_address}: {e}")

    print(f"Updated metrics for {len(JURORS)} jurors.")

if __name__ == '__main__':
    # Start up the server to expose the metrics.
    port = int(os.environ.get("EXPORTER_PORT", 8000))
    start_http_server(port)
    print(f"Started Kleros Theosis Exporter on port {port}")
    print(f"Monitoring Oracle Contract at: {CONTRACT_ADDRESS}")

    # Generate some requests.
    while True:
        fetch_metrics()
        time.sleep(15) # Fetch every 15 seconds
