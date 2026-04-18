#!/bin/bash
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd "$SCRIPT_DIR/.." && pwd)
# shellcheck source=/dev/null
source "$SCRIPT_DIR/container-runtime.sh"

usage() {
    cat <<EOF
Usage: $0 <peer-index> <remote-public-host> <main-public-host> [ssh-user@remote-host]

Examples:
  $0 1 203.0.113.25 198.51.100.10
  $0 1 203.0.113.25 198.51.100.10 user@203.0.113.25

What it does:
  1. Builds a deployment bundle for the remote peer
  2. Optionally uploads it over SSH/SCP and starts the peer remotely
  3. Prints, or automatically runs, the local onboarding step that joins the peer to the channel
EOF
}

if [ "$#" -lt 3 ] || [ "$#" -gt 4 ]; then
    usage
    exit 1
fi

PEER_INDEX="$1"
REMOTE_PUBLIC_HOST="$2"
MAIN_PUBLIC_HOST="$3"
SSH_DEST="${4:-}"
REMOTE_RUNTIME="${REMOTE_RUNTIME:-podman}"
DEPLOY_DIR="$SCRIPT_DIR/deploy_peer${PEER_INDEX}"
PEER_NAME="peer${PEER_INDEX}.org1.dwntp.com"
PEER_PORT=$((7051 + PEER_INDEX * 10))
CHAINCODE_PORT=$((7052 + PEER_INDEX * 10))
OPERATIONS_PORT=$((9443 + PEER_INDEX * 10))
CRYPTO_SRC="$SCRIPT_DIR/crypto-config/peerOrganizations/org1.dwntp.com/peers/${PEER_NAME}"
CRYPTO_DEST="$DEPLOY_DIR/crypto-config/peerOrganizations/org1.dwntp.com/peers/${PEER_NAME}"

if ! [[ "$PEER_INDEX" =~ ^[0-9]+$ ]]; then
    echo "Peer index must be a non-negative integer."
    exit 1
fi

if [ "$PEER_INDEX" -eq 0 ]; then
    echo "Remote setup is intended for peer1 or higher. peer0 is assumed to run on the coordinator host."
    exit 1
fi

for required_path in \
    "$CRYPTO_SRC" \
    "$SCRIPT_DIR/channel-artifacts/dwntpchannel.block" \
    "$SCRIPT_DIR/chaincode.tar.gz"
do
    if [ ! -e "$required_path" ]; then
        echo "Missing required artifact: $required_path"
        echo "Run ./network/generate.sh and bring up the main node first."
        exit 1
    fi
done

rm -rf "$DEPLOY_DIR"
mkdir -p "$CRYPTO_DEST"

cp -r "$CRYPTO_SRC/msp" "$CRYPTO_DEST/"
cp -r "$CRYPTO_SRC/tls" "$CRYPTO_DEST/"

cat <<EOF > "$DEPLOY_DIR/node.env"
PEER_NAME=$PEER_NAME
PEER_PORT=$PEER_PORT
CHAINCODE_PORT=$CHAINCODE_PORT
OPERATIONS_PORT=$OPERATIONS_PORT
PEER_PUBLIC_HOST=$REMOTE_PUBLIC_HOST
MAIN_PUBLIC_HOST=$MAIN_PUBLIC_HOST
REMOTE_RUNTIME=${REMOTE_RUNTIME}
EOF

cat <<'EOF' > "$DEPLOY_DIR/start_remote.sh"
#!/bin/bash
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=/dev/null
source "$SCRIPT_DIR/node.env"

detect_runtime() {
    if [ -n "${CONTAINER_RUNTIME:-}" ]; then
        echo "$CONTAINER_RUNTIME"
        return 0
    fi

    if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then
        echo "docker"
        return 0
    fi

    if command -v podman >/dev/null 2>&1 && podman info >/dev/null 2>&1; then
        echo "podman"
        return 0
    fi

    if command -v "$REMOTE_RUNTIME" >/dev/null 2>&1; then
        echo "$REMOTE_RUNTIME"
        return 0
    fi

    echo "Neither docker nor podman is reachable on the remote host."
    exit 1
}

RUNTIME=$(detect_runtime)
VOLUME_SUFFIX=""
if [ "$RUNTIME" = "podman" ]; then
    VOLUME_SUFFIX=":z"
fi

"$RUNTIME" rm -f "$PEER_NAME" 2>/dev/null || true
"$RUNTIME" network inspect dwntp-remote >/dev/null 2>&1 || "$RUNTIME" network create dwntp-remote

"$RUNTIME" run -d --restart unless-stopped --name "$PEER_NAME" --network dwntp-remote \
  --add-host "orderer.dwntp.com:${MAIN_PUBLIC_HOST}" \
  --add-host "peer0.org1.dwntp.com:${MAIN_PUBLIC_HOST}" \
  --add-host "dwntp-chaincode:${MAIN_PUBLIC_HOST}" \
  -p "${PEER_PORT}:7051" \
  -p "${CHAINCODE_PORT}:7052" \
  -p "${OPERATIONS_PORT}:9443" \
  -e FABRIC_LOGGING_SPEC=INFO \
  -e CORE_OPERATIONS_LISTENADDRESS=0.0.0.0:9443 \
  -e CORE_METRICS_PROVIDER=prometheus \
  -e CORE_PEER_ID="$PEER_NAME" \
  -e CORE_PEER_ADDRESS="${PEER_NAME}:7051" \
  -e CORE_PEER_LISTENADDRESS=0.0.0.0:7051 \
  -e CORE_PEER_CHAINCODEADDRESS="${PEER_NAME}:7052" \
  -e CORE_PEER_CHAINCODELISTENADDRESS=0.0.0.0:7052 \
  -e CORE_PEER_GOSSIP_BOOTSTRAP="peer0.org1.dwntp.com:7051" \
  -e CORE_PEER_GOSSIP_EXTERNALENDPOINT="${PEER_NAME}:${PEER_PORT}" \
  -e CORE_PEER_LOCALMSPID=Org1MSP \
  -e CORE_PEER_MSPCONFIGPATH=/etc/hyperledger/fabric/msp \
  -e CORE_PEER_TLS_ENABLED=true \
  -e CORE_PEER_TLS_CERT_FILE=/etc/hyperledger/fabric/tls/server.crt \
  -e CORE_PEER_TLS_KEY_FILE=/etc/hyperledger/fabric/tls/server.key \
  -e CORE_PEER_TLS_ROOTCERT_FILE=/etc/hyperledger/fabric/tls/ca.crt \
  -e CORE_VM_ENDPOINT= \
  -v "$SCRIPT_DIR/crypto-config/peerOrganizations/org1.dwntp.com/peers/${PEER_NAME}/msp:/etc/hyperledger/fabric/msp${VOLUME_SUFFIX}" \
  -v "$SCRIPT_DIR/crypto-config/peerOrganizations/org1.dwntp.com/peers/${PEER_NAME}/tls:/etc/hyperledger/fabric/tls${VOLUME_SUFFIX}" \
  docker.io/hyperledger/fabric-peer:2.5 peer node start

echo "Remote peer started:"
echo "  $PEER_NAME"
echo "  mapped public endpoint: ${PEER_PUBLIC_HOST}:${PEER_PORT}"
EOF

cat <<'EOF' > "$DEPLOY_DIR/stop_remote.sh"
#!/bin/bash
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=/dev/null
source "$SCRIPT_DIR/node.env"

if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then
    RUNTIME="docker"
elif command -v podman >/dev/null 2>&1 && podman info >/dev/null 2>&1; then
    RUNTIME="podman"
else
    echo "Neither docker nor podman is reachable."
    exit 1
fi

"$RUNTIME" rm -f "$PEER_NAME" 2>/dev/null || true
echo "Stopped $PEER_NAME"
EOF

cat <<EOF > "$DEPLOY_DIR/README.txt"
1. Copy this folder to the remote host.
2. Start the peer on the remote host:
   ./start_remote.sh
3. On the coordinator host, onboard the peer:
   ./network/onboard_remote_peer.sh ${PEER_INDEX} ${REMOTE_PUBLIC_HOST}
EOF

chmod +x "$DEPLOY_DIR/start_remote.sh" "$DEPLOY_DIR/stop_remote.sh"

echo "Created remote deployment bundle in: $DEPLOY_DIR"

if [ -n "$SSH_DEST" ]; then
    REMOTE_PARENT="~/$(basename "$REPO_ROOT")-remote"
    echo "Uploading deployment bundle to $SSH_DEST:$REMOTE_PARENT/"
    ssh "$SSH_DEST" "mkdir -p $REMOTE_PARENT"
    scp -r "$DEPLOY_DIR" "$SSH_DEST:$REMOTE_PARENT/"
    ssh "$SSH_DEST" "cd $REMOTE_PARENT/$(basename "$DEPLOY_DIR") && ./start_remote.sh"

    echo "Remote peer started. Onboarding from coordinator..."
    "$SCRIPT_DIR/onboard_remote_peer.sh" "$PEER_INDEX" "$REMOTE_PUBLIC_HOST" "$PEER_PORT"
else
    echo ""
    echo "Manual steps remaining:"
    echo "  1. Copy $DEPLOY_DIR to the remote host"
    echo "  2. Run ./start_remote.sh on the remote host"
    echo "  3. Run: ./network/onboard_remote_peer.sh $PEER_INDEX $REMOTE_PUBLIC_HOST $PEER_PORT"
fi
