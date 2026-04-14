#!/bin/bash
set -e

# Automatically detect docker or podman
if command -v podman &> /dev/null; then
    export DOCKER_HOST=unix:///run/user/$(id -u)/podman/podman.sock
elif command -v docker &> /dev/null; then
    export DOCKER_HOST=unix:///var/run/docker.sock
else
    echo "Neither podman nor docker found."
    exit 1
fi

# Export Prometheus scrape port to avoid collision with Grafana on port 3000
export CALIPER_OBSERVER_PROMETHEUS_SCRAPEPORT=3001

# Grafana configuration for annotations
GRAFANA_URL=${GRAFANA_URL:-"http://localhost:3000"}
GRAFANA_AUTH=${GRAFANA_AUTH:-"admin:admin"}

# Ensure we are executing from the DWNTP/caliper directory
cd "$(dirname "$0")"

# Create the reports directory
mkdir -p reports

# Define the node configurations to benchmark
PEER_COUNTS=(2 4 8 16)

echo "Starting Multi-Node Benchmarks..."
echo "Node counts to test: ${PEER_COUNTS[*]}"
echo "Reports will be saved to: $(pwd)/reports"

for PEERS in "${PEER_COUNTS[@]}"; do
    echo ""
    echo "=="
    echo "  Setting up Network and Benchmarking $PEERS Nodes"
    echo "=="
    echo ""

    # 1. Generate crypto materials and artifacts for $PEERS
    echo "[1/4] Generating network artifacts for $PEERS peers..."
    (cd .. && ./network/generate.sh $PEERS)

    # 2. Redeploy the network with $PEERS
    echo "[2/4] Deploying network for $PEERS peers... (This may take a moment)"
    (cd .. && ./network/redeploy.sh $PEERS)

    # 3. Generate Caliper network config for $PEERS
    echo "[3/4] Generating Caliper connection profile for $PEERS peers..."
    node generate-config.js $PEERS

    # 4. Run the benchmark
    echo "[4/4] Running Caliper benchmark..."

    # Record start time for Grafana annotation (in milliseconds)
    START_TIME=$(date +%s%3N)

    # The benchmark script runs the Caliper flow
    # Allow failure without exiting the script so annotations and next runs still happen
    npm run benchmark || true

    # Record end time for Grafana annotation (in milliseconds)
    END_TIME=$(date +%s%3N)

    # Post region annotation to Grafana
    echo ">>> Posting region annotation to Grafana..."
    curl -s -X POST "$GRAFANA_URL/api/annotations" \
      -H "Content-Type: application/json" \
      -H "Accept: application/json" \
      -u "$GRAFANA_AUTH" \
      -d "{
        \"time\": $START_TIME,
        \"timeEnd\": $END_TIME,
        \"tags\": [\"benchmark\", \"${PEERS}-nodes\"],
        \"text\": \"Benchmark Run: ${PEERS} Nodes\"
      }" > /dev/null || echo ">>> WARNING: Failed to post annotation to Grafana."

    # 5. Save the report
    REPORT_FILE="reports/report-${PEERS}-nodes.html"
    if [ -f "report.html" ]; then
        mv report.html "$REPORT_FILE"
        echo ">>> Successfully saved report for $PEERS nodes to $REPORT_FILE"
    else
        echo ">>> WARNING: report.html not found after benchmark run for $PEERS nodes!"
    fi

    echo "Waiting 10 seconds before tearing down and moving to the next configuration..."
    sleep 10
done

echo ""
echo "=="

echo "  All Benchmarks Completed Successfully!"
echo "  Check the DWNTP/caliper/reports/ directory for results."
echo "=="

echo ""
echo "Cleaning up final network containers..."
# Kill all potential fabric nodes and chaincode containers
$(command -v podman || command -v docker) rm -f -v orderer.dwntp.com cli dwntp-chaincode 2>/dev/null || true
# Kill up to the maximum number of peers we might have started (16 in this case)

for i in $(seq 0 15); do
    $(command -v podman || command -v docker) rm -f -v "peer${i}.org1.dwntp.com" 2>/dev/null || true
done
echo "Cleanup complete."
