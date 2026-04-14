#!/bin/bash
set -e

if command -v podman &> /dev/null; then
    DOCKER_CMD="podman"
elif command -v docker &> /dev/null; then
    DOCKER_CMD="docker"
else
    echo "Neither podman nor docker found."
    exit 1
fi

echo "Starting monitoring stack (Prometheus & Grafana)..."

# Cleanup any existing monitoring containers
$DOCKER_CMD rm -f prometheus grafana 2>/dev/null || true

# Ensure network exists
$DOCKER_CMD network inspect dwntp-network >/dev/null 2>&1 || $DOCKER_CMD network create dwntp-network

# Start Prometheus
echo "Starting Prometheus..."
$DOCKER_CMD run -d --name prometheus --network dwntp-network -p 9090:9090 \
  -v $PWD/network/monitoring/prometheus.yml:/etc/prometheus/prometheus.yml:z \
  docker.io/prom/prometheus:latest

# Start Grafana
echo "Starting Grafana..."
$DOCKER_CMD run -d --name grafana --network dwntp-network -p 3000:3000 \
  -e GF_SECURITY_ADMIN_PASSWORD=admin \
  -v $PWD/network/monitoring/grafana/provisioning/datasources:/etc/grafana/provisioning/datasources:z \
  -v $PWD/network/monitoring/grafana/provisioning/dashboards:/etc/grafana/provisioning/dashboards:z \
  docker.io/grafana/grafana:latest

echo "Monitoring stack started successfully!"
echo "---------------------------------------------------"
echo "Prometheus available at: http://localhost:9090"
echo "Grafana available at:    http://localhost:3000 (admin/admin)"
echo "---------------------------------------------------"
