#!/bin/bash
# Script to scaffold a remote Hyperledger Fabric Peer deployment

set -e

if [ "$#" -lt 4 ]; then
    echo "Usage: $0 <PEER_INDEX> <PUBLIC_IP> <PEER_PORT> <CHAINCODE_PORT>"
    echo "Example: $0 2 198.51.100.1 8051 8052"
    echo ""
    echo "This script generates a 'docker-compose-remote-peerX.yml' file."
    echo "You must manually copy this file and the corresponding crypto-config material to the remote host."
    exit 1
fi

PEER_INDEX=$1
PUBLIC_IP=$2
PEER_PORT=$3
CC_PORT=$4

PEER_NAME="peer${PEER_INDEX}.org1.dwntp.com"
COMPOSE_FILE="docker-compose-remote-peer${PEER_INDEX}.yml"

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
    working_dir: /opt/gopath/src/github.com/hyperledger/fabric/peer
    command: peer node start
    volumes:
      # YOU MUST RSYNC THIS DIRECTORY FROM THE MAIN HOST TO THIS MACHINE
      - ./crypto-config/peerOrganizations/org1.dwntp.com/peers/${PEER_NAME}/msp:/etc/hyperledger/fabric/msp:z
      - ./crypto-config/peerOrganizations/org1.dwntp.com/peers/${PEER_NAME}/tls:/etc/hyperledger/fabric/tls:z
      - /var/run/docker.sock:/host/var/run/docker.sock:z
    ports:
      - "${PEER_PORT}:${PEER_PORT}"
      - "${CC_PORT}:${CC_PORT}"
    networks:
      - dwntp-remote
    extra_hosts:
      # You need to manually map the Orderer and bootstrap peer to their public IPs
      - "orderer.dwntp.com:<INSERT_ORDERER_PUBLIC_IP>"
      - "peer0.org1.dwntp.com:<INSERT_PEER0_PUBLIC_IP>"
YAML

echo "Successfully generated $COMPOSE_FILE"
echo ""
echo "NEXT STEPS:"
echo "1. On the central/main machine, edit 'network/crypto-config.yaml' to add '$PUBLIC_IP' under SANS."
echo "2. Re-run cryptogen to update certificates with the new SAN."
echo "3. Rsync/scp the generated '$COMPOSE_FILE' and the specific peer's crypto folder to the remote machine."
echo "4. On the remote machine, edit the 'extra_hosts' section of '$COMPOSE_FILE' to point to the other nodes' public IPs."
echo "5. Run: docker-compose -f $COMPOSE_FILE up -d"
