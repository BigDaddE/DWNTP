#!/bin/bash
set -e

NUM_PEERS=${1:-1}

echo "Compiling Go chaincode..."
(cd chaincode && CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -o dwntp-chaincode main.go)

echo "Building dwntp-chaincode Docker image..."
TMPDIR=~/tmp $(command -v podman || command -v docker) build --no-cache -t dwntp-chaincode:latest -f chaincode/Dockerfile chaincode

# Cleanup any existing containers
$(command -v podman || command -v docker) rm -f -v orderer.dwntp.com cli dwntp-chaincode $(for i in $(seq 0 $((NUM_PEERS-1))); do echo "peer${i}.org1.dwntp.com"; done) 2>/dev/null || true

# Create network if it doesn't exist
$(command -v podman || command -v docker) network inspect dwntp-network >/dev/null 2>&1 || $(command -v podman || command -v docker) network create dwntp-network

# Start Orderer (No System Channel approach, Fabric v2.3+)
echo "Starting Orderer..."
$(command -v podman || command -v docker) run -d --name orderer.dwntp.com --network dwntp-network -p 7050:7050 -p 7053:7053 -p 8443:8443 \
  -e FABRIC_LOGGING_SPEC=INFO \
  -e ORDERER_OPERATIONS_LISTENADDRESS=0.0.0.0:8443 \
  -e ORDERER_METRICS_PROVIDER=prometheus \
  -e ORDERER_GENERAL_LISTENADDRESS=0.0.0.0 \
  -e ORDERER_GENERAL_LISTENPORT=7050 \
  -e ORDERER_GENERAL_LOCALMSPID=OrdererMSP \
  -e ORDERER_GENERAL_LOCALMSPDIR=/var/hyperledger/orderer/msp \
  -e ORDERER_GENERAL_TLS_ENABLED=true \
  -e ORDERER_GENERAL_TLS_PRIVATEKEY=/var/hyperledger/orderer/tls/server.key \
  -e ORDERER_GENERAL_TLS_CERTIFICATE=/var/hyperledger/orderer/tls/server.crt \
  -e ORDERER_GENERAL_TLS_ROOTCAS=/var/hyperledger/orderer/tls/ca.crt \
  -e ORDERER_GENERAL_CLUSTER_CLIENTCERTIFICATE=/var/hyperledger/orderer/tls/server.crt \
  -e ORDERER_GENERAL_CLUSTER_CLIENTPRIVATEKEY=/var/hyperledger/orderer/tls/server.key \
  -e ORDERER_GENERAL_CLUSTER_ROOTCAS=/var/hyperledger/orderer/tls/ca.crt \
  -e ORDERER_GENERAL_BOOTSTRAPMETHOD=none \
  -e ORDERER_CHANNELPARTICIPATION_ENABLED=true \
  -e ORDERER_ADMIN_TLS_ENABLED=true \
  -e ORDERER_ADMIN_TLS_CERTIFICATE=/var/hyperledger/orderer/tls/server.crt \
  -e ORDERER_ADMIN_TLS_PRIVATEKEY=/var/hyperledger/orderer/tls/server.key \
  -e ORDERER_ADMIN_TLS_ROOTCAS=/var/hyperledger/orderer/tls/ca.crt \
  -e ORDERER_ADMIN_TLS_CLIENTROOTCAS=/var/hyperledger/orderer/tls/ca.crt \
  -e ORDERER_ADMIN_TLS_CLIENTAUTHREQUIRED=true \
  -e ORDERER_ADMIN_LISTENADDRESS=0.0.0.0:7053 \
  -v $PWD/network/channel-artifacts:/var/hyperledger/orderer/channel-artifacts:z \
  -v $PWD/network/crypto-config/ordererOrganizations/dwntp.com/orderers/orderer.dwntp.com/msp:/var/hyperledger/orderer/msp:z \
  -v $PWD/network/crypto-config/ordererOrganizations/dwntp.com/orderers/orderer.dwntp.com/tls/:/var/hyperledger/orderer/tls:z \
  docker.io/hyperledger/fabric-orderer:2.5 orderer

for i in $(seq 0 $((NUM_PEERS-1))); do
  PEER_NAME="peer${i}.org1.dwntp.com"
  PEER_PORT=$((7051 + i * 10))
  CHAINCODE_PORT=$((7052 + i * 10))
  OPERATIONS_PORT=$((9443 + i * 10))

  echo "Starting ${PEER_NAME}..."
  $(command -v podman || command -v docker) run -d --name ${PEER_NAME} --network dwntp-network -p ${PEER_PORT}:7051 -p ${CHAINCODE_PORT}:7052 -p ${OPERATIONS_PORT}:9443 \
    --add-host host.containers.internal:host-gateway \
    -e FABRIC_LOGGING_SPEC=INFO \
    -e CORE_OPERATIONS_LISTENADDRESS=0.0.0.0:9443 \
    -e CORE_METRICS_PROVIDER=prometheus \
    -e CORE_PEER_ID=${PEER_NAME} \
    -e CORE_PEER_ADDRESS=${PEER_NAME}:7051 \
    -e CORE_PEER_LISTENADDRESS=0.0.0.0:7051 \
    -e CORE_PEER_CHAINCODEADDRESS=${PEER_NAME}:7052 \
    -e CORE_PEER_CHAINCODELISTENADDRESS=0.0.0.0:7052 \
    -e CORE_PEER_GOSSIP_BOOTSTRAP=peer0.org1.dwntp.com:7051 \
    -e CORE_PEER_GOSSIP_EXTERNALENDPOINT=${PEER_NAME}:7051 \
    -e CORE_PEER_LOCALMSPID=Org1MSP \
    -e CORE_PEER_MSPCONFIGPATH=/etc/hyperledger/fabric/msp \
    -e CORE_PEER_TLS_ENABLED=true \
    -e CORE_PEER_TLS_CERT_FILE=/etc/hyperledger/fabric/tls/server.crt \
    -e CORE_PEER_TLS_KEY_FILE=/etc/hyperledger/fabric/tls/server.key \
    -e CORE_PEER_TLS_ROOTCERT_FILE=/etc/hyperledger/fabric/tls/ca.crt \
    -e CORE_VM_ENDPOINT= \
    -v $PWD/network/crypto-config/peerOrganizations/org1.dwntp.com/peers/${PEER_NAME}/msp:/etc/hyperledger/fabric/msp:z \
    -v $PWD/network/crypto-config/peerOrganizations/org1.dwntp.com/peers/${PEER_NAME}/tls:/etc/hyperledger/fabric/tls:z \
    docker.io/hyperledger/fabric-peer:2.5 peer node start
done


# Start CLI
echo "Starting CLI..."
$(command -v podman || command -v docker) run -d -it --name cli --network dwntp-network \
  -e GOPATH=/opt/gopath \
  -e FABRIC_LOGGING_SPEC=INFO \
  -e CORE_PEER_ID=cli \
  -e CORE_PEER_ADDRESS=peer0.org1.dwntp.com:7051 \
  -e CORE_PEER_LOCALMSPID=Org1MSP \
  -e CORE_PEER_TLS_ENABLED=true \
  -e CORE_PEER_TLS_CERT_FILE=/opt/gopath/src/github.com/hyperledger/fabric/peer/crypto/peerOrganizations/org1.dwntp.com/peers/peer0.org1.dwntp.com/tls/server.crt \
  -e CORE_PEER_TLS_KEY_FILE=/opt/gopath/src/github.com/hyperledger/fabric/peer/crypto/peerOrganizations/org1.dwntp.com/peers/peer0.org1.dwntp.com/tls/server.key \
  -e CORE_PEER_TLS_ROOTCERT_FILE=/opt/gopath/src/github.com/hyperledger/fabric/peer/crypto/peerOrganizations/org1.dwntp.com/peers/peer0.org1.dwntp.com/tls/ca.crt \
  -e CORE_PEER_MSPCONFIGPATH=/opt/gopath/src/github.com/hyperledger/fabric/peer/crypto/peerOrganizations/org1.dwntp.com/users/Admin@org1.dwntp.com/msp \
  -e ORDERER_CA=/opt/gopath/src/github.com/hyperledger/fabric/peer/crypto/ordererOrganizations/dwntp.com/orderers/orderer.dwntp.com/tls/ca.crt \
  -e ORDERER_ADMIN_TLS_SIGN_CERT=/opt/gopath/src/github.com/hyperledger/fabric/peer/crypto/ordererOrganizations/dwntp.com/orderers/orderer.dwntp.com/tls/server.crt \
  -e ORDERER_ADMIN_TLS_PRIVATE_KEY=/opt/gopath/src/github.com/hyperledger/fabric/peer/crypto/ordererOrganizations/dwntp.com/orderers/orderer.dwntp.com/tls/server.key \
  -v $PWD/network/crypto-config:/opt/gopath/src/github.com/hyperledger/fabric/peer/crypto:z \
  -v $PWD/network/channel-artifacts:/opt/gopath/src/github.com/hyperledger/fabric/peer/channel-artifacts:z \
  -v $PWD/network:/opt/gopath/src/github.com/hyperledger/fabric/peer/network:z \
  docker.io/hyperledger/fabric-tools:2.5 /bin/bash

echo "Network started!"
