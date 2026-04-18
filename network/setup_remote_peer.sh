#!/bin/bash
# Script to scaffold and optionally deploy a remote Hyperledger Fabric Peer

set -e

if [ "$#" -lt 5 ]; then
    echo "Usage: $0 <PEER_INDEX> <REMOTE_PUBLIC_IP> <PEER_PORT> <CHAINCODE_PORT> <MAIN_HOST_PUBLIC_IP> [SSH_USER@REMOTE_HOST]"
    echo "Example (Scaffold only): $0 1 4.3.2.1 7051 7052 1.2.3.4"
    echo "Example (Scaffold & Deploy): $0 1 4.3.2.1 7051 7052 1.2.3.4 root@4.3.2.1"
    echo ""
    echo "This script generates a full deployment folder for the remote peer,"
    echo "containing the docker-compose file and the necessary crypto material."
    echo "If an SSH destination is provided, it will automatically scp the folder to the remote host."
    exit 1
fi

PEER_INDEX=$1
PUBLIC_IP=$2
PEER_PORT=$3
CC_PORT=$4
MAIN_IP=$5
SSH_DEST=$6

PEER_NAME="peer${PEER_INDEX}.org1.dwntp.com"
DEPLOY_DIR="deploy_peer${PEER_INDEX}"
COMPOSE_FILE="$DEPLOY_DIR/docker-compose.yml"
CRYPTO_SRC="crypto-config/peerOrganizations/org1.dwntp.com/peers/${PEER_NAME}"
CRYPTO_DEST="$DEPLOY_DIR/crypto-config/peerOrganizations/org1.dwntp.com/peers/${PEER_NAME}"

# 1. Verify Crypto Material exists
if [ ! -d "$CRYPTO_SRC" ]; then
    echo "Error: Crypto material for $PEER_NAME not found at $CRYPTO_SRC"
    echo "Did you remember to run ./generate.sh first?"
    exit 1
fi

echo "==> Creating deployment package for $PEER_NAME in $DEPLOY_DIR/"
mkdir -p "$CRYPTO_DEST"

# 2. Copy the crypto material to the deployment directory
cp -r "$CRYPTO_SRC/msp" "$CRYPTO_DEST/"
cp -r "$CRYPTO_SRC/tls" "$CRYPTO_DEST/"

# 3. Generate the Docker Compose file directly in the deployment directory
cat <<YAML > "$COMPOSE_FILE"
version: "3.7"

networks:
  dwntp-remote:
    name: dwntp-network

services:
  ${PEER_NAME}:
    image: docker.io/hyperledger/fabric-peer:2.5
    container_name: ${PEER_NAME}
    environment:
      - CORE_VM_ENDPOINT=unix:///host/var/run/docker.sock
      - CORE_VM_DOCKER_HOSTCONFIG_NETWORKMODE=dwntp-network
      - FABRIC_LOGGING_SPEC=INFO
      - CORE_PEER_TLS_ENABLED=true
      - CORE_PEER_PROFILE_ENABLED=false
      - CORE_PEER_TLS_CERT_FILE=/etc/hyperledger/fabric/tls/server.crt
      - CORE_PEER_TLS_KEY_FILE=/etc/hyperledger/fabric/tls/server.key
      - CORE_PEER_TLS_ROOTCERT_FILE=/etc/hyperledger/fabric/tls/ca.crt
      - CORE_PEER_ID=${PEER_NAME}
      - CORE_PEER_ADDRESS=${PUBLIC_IP}:${PEER_PORT}
      - CORE_PEER_LISTENADDRESS=0.0.0.0:${PEER_PORT}
      - CORE_PEER_CHAINCODEADDRESS=0.0.0.0:${CC_PORT}
      - CORE_PEER_CHAINCODELISTENADDRESS=0.0.0.0:${CC_PORT}
      # Gossip config for multi-host internet routing
      - CORE_PEER_GOSSIP_EXTERNALENDPOINT=${PUBLIC_IP}:${PEER_PORT}
      - CORE_PEER_GOSSIP_BOOTSTRAP=peer0.org1.dwntp.com:7051 # Point to a known peer
      - CORE_PEER_LOCALMSPID=Org1MSP
      # Disable local docker socket reliance for Chaincode-as-a-Service approach
      - CORE_VM_ENDPOINT=
    working_dir: /opt/gopath/src/github.com/hyperledger/fabric/peer
    command: peer node start
    volumes:
      # Mount the relative crypto material
      - ./crypto-config/peerOrganizations/org1.dwntp.com/peers/${PEER_NAME}/msp:/etc/hyperledger/fabric/msp:z
      - ./crypto-config/peerOrganizations/org1.dwntp.com/peers/${PEER_NAME}/tls:/etc/hyperledger/fabric/tls:z
    ports:
      - "${PEER_PORT}:${PEER_PORT}"
      - "${CC_PORT}:${CC_PORT}"
    networks:
      - dwntp-remote
    extra_hosts:
      # Map the Orderer and bootstrap peer to the Main Host Public IP
      - "orderer.dwntp.com:${MAIN_IP}"
      - "peer0.org1.dwntp.com:${MAIN_IP}"
YAML

# 4. Generate a helper start script
cat <<'SCRIPT' > "$DEPLOY_DIR/start_remote.sh"
#!/bin/bash
echo "Starting Remote Peer..."
docker-compose up -d
echo "Peer is running!"
SCRIPT
chmod +x "$DEPLOY_DIR/start_remote.sh"

echo "==> Deployment package ready: $DEPLOY_DIR/"

# 5. Push to remote server if SSH provided
if [ -n "$SSH_DEST" ]; then
    echo "==> Pushing to remote server: $SSH_DEST"
    scp -r "$DEPLOY_DIR" "$SSH_DEST:~/"
    
    echo ""
    echo "=========================================================================="
    echo "SUCCESS: Files have been transferred!"
    echo ""
    echo "To start the peer, SSH into the remote machine and run:"
    echo "  ssh $SSH_DEST"
    echo "  cd ~/$DEPLOY_DIR"
    echo "  ./start_remote.sh"
    echo "=========================================================================="
else
    echo ""
    echo "=========================================================================="
    echo "SUCCESS: Deployment folder generated."
    echo ""
    echo "To start the peer, manually transfer the '$DEPLOY_DIR' folder to the remote host."
    echo "Once on the remote host, navigate to the folder and run:"
    echo "  docker-compose up -d"
    echo "=========================================================================="
fi
