#!/bin/bash
set -e

echo "Starting monitoring stack (Prometheus & Grafana)..."

# Cleanup any existing monitoring containers
$(command -v podman || command -v docker) rm -f prometheus grafana 2>/dev/null || true

# Ensure network exists
$(command -v podman || command -v docker) network inspect dwntp-network >/dev/null 2>&1 || $(command -v podman || command -v docker) network create dwntp-network

# Start Prometheus
echo "Starting Prometheus..."
$(command -v podman || command -v docker) run -d --name prometheus --network dwntp-network -p 9090:9090 \
  -v $PWD/network/monitoring/prometheus.yml:/etc/prometheus/prometheus.yml:z \
  docker.io/prom/prometheus:latest

# Start Grafana
echo "Starting Grafana..."
$(command -v podman || command -v docker) run -d --name grafana --network dwntp-network -p 3000:3000 \
  -e GF_SECURITY_ADMIN_PASSWORD=admin \
  -v $PWD/network/monitoring/grafana/provisioning/datasources:/etc/grafana/provisioning/datasources:z \
  -v $PWD/network/monitoring/grafana/provisioning/dashboards:/etc/grafana/provisioning/dashboards:z \
  docker.io/grafana/grafana:latest

echo "Monitoring stack started successfully!"
echo "---------------------------------------------------"
echo "Prometheus available at: http://localhost:9090"
echo "Grafana available at:    http://localhost:3000 (admin/admin)"
echo "---------------------------------------------------"
