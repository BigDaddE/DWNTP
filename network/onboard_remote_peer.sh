#!/bin/bash
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=/dev/null
source "$SCRIPT_DIR/container-runtime.sh"

usage() {
    echo "Usage: $0 <peer-index> <remote-public-host> [peer-port]"
}

if [ "$#" -lt 2 ] || [ "$#" -gt 3 ]; then
    usage
    exit 1
fi

PEER_INDEX="$1"
REMOTE_PUBLIC_HOST="$2"
PEER_PORT="${3:-$((7051 + PEER_INDEX * 10))}"
PEER_NAME="peer${PEER_INDEX}.org1.dwntp.com"
RUNTIME=$(detect_container_runtime)
CONN_TIMEOUT=${PEER_CONN_TIMEOUT:-30s}

if ! "$RUNTIME" ps --format '{{.Names}}' | grep -qx 'cli'; then
    echo "The local CLI container is not running. Start the main network first."
    exit 1
fi

wait_for_endpoint() {
    local host_name="$1"
    local port="$2"
    local attempt=1
    local max_attempts="${3:-60}"

    while [ "$attempt" -le "$max_attempts" ]; do
        if "$RUNTIME" exec cli bash -lc "echo > /dev/tcp/${host_name}/${port}" >/dev/null 2>&1; then
            return 0
        fi

        sleep 1
        attempt=$((attempt + 1))
    done

    echo "Timed out waiting for ${host_name}:${port}"
    exit 1
}

run_peer_cmd() {
    local peer_cmd="$1"
    "$RUNTIME" exec \
      -e CORE_PEER_ADDRESS="${PEER_NAME}:${PEER_PORT}" \
      -e CORE_PEER_TLS_ROOTCERT_FILE="/opt/gopath/src/github.com/hyperledger/fabric/peer/crypto/peerOrganizations/org1.dwntp.com/peers/${PEER_NAME}/tls/ca.crt" \
      cli bash -lc "$peer_cmd"
}

ensure_host_alias() {
    local container_name="$1"
    local host_name="$2"
    local host_ip="$3"

    if ! "$RUNTIME" ps --format '{{.Names}}' | grep -qx "$container_name"; then
        return 0
    fi

    "$RUNTIME" exec "$container_name" bash -lc "grep -Eq '(^|[[:space:]])${host_name}([[:space:]]|\$)' /etc/hosts || echo '${host_ip} ${host_name}' >> /etc/hosts"
}

echo "Waiting for remote peer ${REMOTE_PUBLIC_HOST}:${PEER_PORT}..."
wait_for_endpoint "$REMOTE_PUBLIC_HOST" "$PEER_PORT" 60

ensure_host_alias "cli" "$PEER_NAME" "$REMOTE_PUBLIC_HOST"
ensure_host_alias "peer0.org1.dwntp.com" "$PEER_NAME" "$REMOTE_PUBLIC_HOST"

echo "Joining ${PEER_NAME} to channel..."
set +e
JOIN_OUTPUT=$(run_peer_cmd "peer channel join -b /opt/gopath/src/github.com/hyperledger/fabric/peer/channel-artifacts/dwntpchannel.block --connTimeout ${CONN_TIMEOUT}" 2>&1)
JOIN_STATUS=$?
set -e
echo "$JOIN_OUTPUT"

if [ "$JOIN_STATUS" -ne 0 ] && ! echo "$JOIN_OUTPUT" | grep -Eqi 'already exists|ledger from genesis block'; then
    exit "$JOIN_STATUS"
fi

echo "Installing chaincode on ${PEER_NAME}..."
set +e
INSTALL_OUTPUT=$(run_peer_cmd "peer lifecycle chaincode install /opt/gopath/src/github.com/hyperledger/fabric/peer/network/chaincode.tar.gz --connTimeout ${CONN_TIMEOUT}" 2>&1)
INSTALL_STATUS=$?
set -e
echo "$INSTALL_OUTPUT"

if [ "$INSTALL_STATUS" -ne 0 ] && ! echo "$INSTALL_OUTPUT" | grep -Eqi 'already successfully installed'; then
    exit "$INSTALL_STATUS"
fi

echo "Remote peer ${PEER_NAME} onboarded successfully."
