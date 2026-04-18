#!/bin/bash
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=/dev/null
source "$SCRIPT_DIR/container-runtime.sh"

RUNTIME=$(detect_container_runtime)
VOLUME_SUFFIX=$(container_volume_suffix "$RUNTIME")
NUM_PEERS=${1:-1}
GENERATED_CRYPTO_CONFIG="$SCRIPT_DIR/.crypto-config.generated.yaml"

cleanup() {
    rm -f "$GENERATED_CRYPTO_CONFIG"
}

trap cleanup EXIT

echo "Setting Peer nodes count to $NUM_PEERS in crypto-config.yaml..."
sed "s/Count: .*/Count: $NUM_PEERS/g" "$SCRIPT_DIR/crypto-config.yaml" > "$GENERATED_CRYPTO_CONFIG"

rm -rf "$SCRIPT_DIR/channel-artifacts" "$SCRIPT_DIR/crypto-config"
mkdir -p "$SCRIPT_DIR/channel-artifacts"
mkdir -p "$SCRIPT_DIR/crypto-config"

echo "Pulling hyperledger/fabric-tools:2.5 image (if not present)..."
"$RUNTIME" pull docker.io/hyperledger/fabric-tools:2.5

echo "Generating crypto materials..."
"$RUNTIME" run --rm -v "$SCRIPT_DIR:/config$VOLUME_SUFFIX" docker.io/hyperledger/fabric-tools:2.5 \
    cryptogen generate --config=/config/.crypto-config.generated.yaml --output=/config/crypto-config

echo "Generating dwntpchannel.block..."
"$RUNTIME" run --rm -v "$SCRIPT_DIR:/config$VOLUME_SUFFIX" -e FABRIC_CFG_PATH=/config docker.io/hyperledger/fabric-tools:2.5 \
    configtxgen -profile DwntpApplicationGenesis -channelID dwntpchannel -outputBlock /config/channel-artifacts/dwntpchannel.block

echo "Packaging external chaincode..."
ROOT_CERT=$(awk 'NF {sub(/\r/, ""); printf "%s\\n",$0;}' "$SCRIPT_DIR/crypto-config/peerOrganizations/org1.dwntp.com/tlsca/tlsca.org1.dwntp.com-cert.pem")
cat << JSON_EOF > "$SCRIPT_DIR/chaincode-external/connection.json"
{
  "address": "dwntp-chaincode:9999",
  "dial_timeout": "10s",
  "tls_required": true,
  "client_auth_required": false,
  "root_cert": "$ROOT_CERT"
}
JSON_EOF

cd "$SCRIPT_DIR/chaincode-external"
tar cfz code.tar.gz connection.json
tar cfz ../chaincode.tar.gz metadata.json code.tar.gz

echo "Chaincode packaged!"
