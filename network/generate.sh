#!/bin/bash
set -e

cd $(dirname $0)
mkdir -p channel-artifacts
mkdir -p crypto-config

echo "Pulling hyperledger/fabric-tools:2.5 image (if not present)..."
podman pull docker.io/hyperledger/fabric-tools:2.5

echo "Generating crypto materials..."
podman run --rm -v $PWD:/config:z docker.io/hyperledger/fabric-tools:2.5 \
    cryptogen generate --config=/config/crypto-config.yaml --output=/config/crypto-config

echo "Generating genesis block..."
podman run --rm -v $PWD:/config:z -e FABRIC_CFG_PATH=/config docker.io/hyperledger/fabric-tools:2.5 \
    configtxgen -profile DwntpApplicationGenesis -channelID system-channel -outputBlock /config/channel-artifacts/genesis.block

echo "Done!"
