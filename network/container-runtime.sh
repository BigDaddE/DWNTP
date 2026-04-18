#!/bin/bash

set -euo pipefail

detect_container_runtime() {
    if [ -n "${CONTAINER_RUNTIME:-}" ]; then
        if command -v "$CONTAINER_RUNTIME" >/dev/null 2>&1; then
            if "$CONTAINER_RUNTIME" info >/dev/null 2>&1; then
                echo "$CONTAINER_RUNTIME"
                return 0
            fi

            echo "Configured container runtime '$CONTAINER_RUNTIME' is installed but not reachable."
            exit 1
        fi

        echo "Configured container runtime '$CONTAINER_RUNTIME' was not found."
        exit 1
    fi

    if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then
        echo "docker"
        return 0
    fi

    if command -v podman >/dev/null 2>&1 && podman info >/dev/null 2>&1; then
        echo "podman"
        return 0
    fi

    if command -v docker >/dev/null 2>&1; then
        echo "Docker is installed but not reachable. Start Docker Desktop or the Docker daemon."
        exit 1
    fi

    if command -v podman >/dev/null 2>&1; then
        echo "Podman is installed but not reachable."
        exit 1
    fi

    echo "Neither docker nor podman was found."
    exit 1
}

container_volume_suffix() {
    local runtime="$1"

    if [ "$runtime" = "podman" ]; then
        echo ":z"
    else
        echo ""
    fi
}
