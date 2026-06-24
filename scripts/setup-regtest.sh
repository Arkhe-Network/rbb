#!/bin/bash
set -e

echo "Starting docker-compose..."
cd docker && docker-compose up -d
cd ..

echo "Waiting for bitcoind to be ready..."
sleep 15

# Mine some blocks to start
docker exec -i docker-bitcoind-1 bitcoin-cli -regtest -rpcuser=devuser -rpcpassword=devpass -generate 101

echo "Done setting up regtest environment"
