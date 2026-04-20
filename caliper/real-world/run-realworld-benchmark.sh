#!/bin/bash
# Runs the DWNTP real-world Caliper benchmark against the 2-node deployment.
# Driven entirely by caliper/real-world/realworld.env.
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
CALIPER_DIR=$(cd "$SCRIPT_DIR/.." && pwd)
ENV_FILE="$SCRIPT_DIR/realworld.env"
TEMPLATE="$SCRIPT_DIR/network-config.template.yaml"
RENDERED="$SCRIPT_DIR/network-config.rendered.yaml"
BENCH="$SCRIPT_DIR/benchmark-config.yaml"

if [ ! -f "$ENV_FILE" ]; then
    echo "Missing $ENV_FILE. Copy realworld.env.example and fill in the values:" >&2
    echo "  cp $SCRIPT_DIR/realworld.env.example $ENV_FILE" >&2
    exit 1
fi

# shellcheck source=/dev/null
set -a
source "$ENV_FILE"
set +a

REQUIRED_VARS=(MAIN_PUBLIC_HOST REMOTE_PUBLIC_HOST ORDERER_PORT PEER0_PORT PEER1_PORT CHANNEL CONTRACT_ID)
for var in "${REQUIRED_VARS[@]}"; do
    if [ -z "${!var:-}" ] || [[ "${!var}" == CHANGE_ME_* ]]; then
        echo "Environment variable $var is not set in $ENV_FILE." >&2
        exit 1
    fi
done

sanity_port() {
    local host=$1
    local port=$2
    local label=$3
    if command -v nc >/dev/null 2>&1; then
        if ! nc -z -w 4 "$host" "$port" >/dev/null 2>&1; then
            echo "WARNING: $label ($host:$port) did not respond to a TCP probe within 4s." >&2
            echo "         Benchmark will continue, but expect failures if the endpoint stays unreachable." >&2
        fi
    else
        echo "note: nc not found, skipping reachability probe for $label." >&2
    fi
}

echo "Checking reachability of the 2-node deployment..."
sanity_port "$MAIN_PUBLIC_HOST"   "$ORDERER_PORT" "orderer"
sanity_port "$MAIN_PUBLIC_HOST"   "$PEER0_PORT"   "peer0"
sanity_port "$REMOTE_PUBLIC_HOST" "$PEER1_PORT"   "peer1"

echo "Rendering $RENDERED from $TEMPLATE..."
sed \
    -e "s|\${MAIN_PUBLIC_HOST}|$MAIN_PUBLIC_HOST|g" \
    -e "s|\${REMOTE_PUBLIC_HOST}|$REMOTE_PUBLIC_HOST|g" \
    -e "s|\${ORDERER_PORT}|$ORDERER_PORT|g" \
    -e "s|\${PEER0_PORT}|$PEER0_PORT|g" \
    -e "s|\${PEER1_PORT}|$PEER1_PORT|g" \
    -e "s|\${CHANNEL}|$CHANNEL|g" \
    -e "s|\${CONTRACT_ID}|$CONTRACT_ID|g" \
    "$TEMPLATE" > "$RENDERED"

if grep -q '\${' "$RENDERED"; then
    echo "Rendered config still has unresolved placeholders:" >&2
    grep -n '\${' "$RENDERED" >&2
    exit 1
fi

TIMESTAMP=$(date +%Y%m%d-%H%M%S)
REPORT_DIR="$SCRIPT_DIR/reports/$TIMESTAMP"
mkdir -p "$REPORT_DIR"
REPORT_PATH="$REPORT_DIR/report.html"

echo "Binding Caliper to fabric:2.2..."
(cd "$CALIPER_DIR" && npx caliper bind --caliper-bind-sut fabric:2.2)

echo "Launching Caliper manager. Reports will land in $REPORT_DIR"
(cd "$CALIPER_DIR" && npx caliper launch manager \
    --caliper-workspace ./real-world \
    --caliper-networkconfig network-config.rendered.yaml \
    --caliper-benchconfig benchmark-config.yaml \
    --caliper-report-path "$REPORT_PATH" \
    --caliper-flow-only-test)

echo
echo "Benchmark complete."
echo "Report: $REPORT_PATH"
