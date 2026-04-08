#!/bin/bash
set -e

NUM_PEERS=${1:-1}
cd $(dirname $0)
echo "Setting Peer nodes count to $NUM_PEERS in crypto-config.yaml..."
sed -i "s/Count: .*/Count: $NUM_PEERS/g" crypto-config.yaml

rm -rf channel-artifacts crypto-config
mkdir -p channel-artifacts
mkdir -p crypto-config

echo "Pulling hyperledger/fabric-tools:2.5 image (if not present)..."
podman pull docker.io/hyperledger/fabric-tools:2.5

echo "Generating crypto materials..."
podman run --rm -v $PWD:/config:z docker.io/hyperledger/fabric-tools:2.5 \
    cryptogen generate --config=/config/crypto-config.yaml --output=/config/crypto-config

echo "Generating dwntpchannel.block..."
podman run --rm -v $PWD:/config:z -e FABRIC_CFG_PATH=/config docker.io/hyperledger/fabric-tools:2.5 \
    configtxgen -profile DwntpApplicationGenesis -channelID dwntpchannel -outputBlock /config/channel-artifacts/dwntpchannel.block

echo "Done!"
