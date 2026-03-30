#!/bin/bash
set -e

# Cleanup any existing containers
podman rm -f orderer.dwntp.com peer0.org1.dwntp.com cli 2>/dev/null || true

# Create network if it doesn't exist
podman network inspect dwntp-network >/dev/null 2>&1 || podman network create dwntp-network

# Start Orderer (No System Channel approach, Fabric v2.3+)
echo "Starting Orderer..."
podman run -d --name orderer.dwntp.com --network dwntp-network -p 7050:7050 -p 7053:7053 \
  -e FABRIC_LOGGING_SPEC=INFO \
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

# Start Peer
echo "Starting Peer..."
podman run -d --name peer0.org1.dwntp.com --network dwntp-network -p 7051:7051 -p 7052:7052 \
  --add-host host.containers.internal:host-gateway \
  -e FABRIC_LOGGING_SPEC=INFO \
  -e CORE_PEER_ID=peer0.org1.dwntp.com \
  -e CORE_PEER_ADDRESS=peer0.org1.dwntp.com:7051 \
  -e CORE_PEER_LISTENADDRESS=0.0.0.0:7051 \
  -e CORE_PEER_CHAINCODEADDRESS=peer0.org1.dwntp.com:7052 \
  -e CORE_PEER_CHAINCODELISTENADDRESS=0.0.0.0:7052 \
  -e CORE_PEER_GOSSIP_BOOTSTRAP=peer0.org1.dwntp.com:7051 \
  -e CORE_PEER_GOSSIP_EXTERNALENDPOINT=peer0.org1.dwntp.com:7051 \
  -e CORE_PEER_LOCALMSPID=Org1MSP \
  -e CORE_PEER_MSPCONFIGPATH=/etc/hyperledger/fabric/msp \
  -e CORE_PEER_TLS_ENABLED=true \
  -e CORE_PEER_TLS_CERT_FILE=/etc/hyperledger/fabric/tls/server.crt \
  -e CORE_PEER_TLS_KEY_FILE=/etc/hyperledger/fabric/tls/server.key \
  -e CORE_PEER_TLS_ROOTCERT_FILE=/etc/hyperledger/fabric/tls/ca.crt \
  -e CORE_VM_ENDPOINT= \
  -v $PWD/network/crypto-config/peerOrganizations/org1.dwntp.com/peers/peer0.org1.dwntp.com/msp:/etc/hyperledger/fabric/msp:z \
  -v $PWD/network/crypto-config/peerOrganizations/org1.dwntp.com/peers/peer0.org1.dwntp.com/tls:/etc/hyperledger/fabric/tls:z \
  docker.io/hyperledger/fabric-peer:2.5 peer node start

# Start CLI
echo "Starting CLI..."
podman run -d -it --name cli --network dwntp-network \
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
