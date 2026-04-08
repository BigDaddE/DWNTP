#!/bin/bash
set -e

NUM_PEERS=${1:-1}

./network/start_network.sh $NUM_PEERS

echo "Waiting for network to boot..."
sleep 5

echo "Joining Channel for Orderer..."
podman exec cli bash -c "osnadmin channel join --channelID dwntpchannel --config-block /opt/gopath/src/github.com/hyperledger/fabric/peer/channel-artifacts/dwntpchannel.block -o orderer.dwntp.com:7053 --ca-file \$ORDERER_CA --client-cert /opt/gopath/src/github.com/hyperledger/fabric/peer/crypto/ordererOrganizations/dwntp.com/users/Admin@dwntp.com/tls/client.crt --client-key /opt/gopath/src/github.com/hyperledger/fabric/peer/crypto/ordererOrganizations/dwntp.com/users/Admin@dwntp.com/tls/client.key"

for i in $(seq 0 $((NUM_PEERS-1))); do
  PEER_NAME="peer${i}.org1.dwntp.com"
  echo "Joining ${PEER_NAME} to Channel..."
  podman exec -e CORE_PEER_ADDRESS=${PEER_NAME}:7051 \
    -e CORE_PEER_TLS_ROOTCERT_FILE=/opt/gopath/src/github.com/hyperledger/fabric/peer/crypto/peerOrganizations/org1.dwntp.com/peers/${PEER_NAME}/tls/ca.crt \
    cli bash -c "peer channel join -b /opt/gopath/src/github.com/hyperledger/fabric/peer/channel-artifacts/dwntpchannel.block"
done

echo "Waiting for Raft leader election on orderer..."
sleep 5

echo "Installing Chaincode on all peers..."
for i in $(seq 0 $((NUM_PEERS-1))); do
  PEER_NAME="peer${i}.org1.dwntp.com"
  echo "Installing on ${PEER_NAME}..."
  INSTALL_OUTPUT=$(podman exec -e CORE_PEER_ADDRESS=${PEER_NAME}:7051 \
    -e CORE_PEER_TLS_ROOTCERT_FILE=/opt/gopath/src/github.com/hyperledger/fabric/peer/crypto/peerOrganizations/org1.dwntp.com/peers/${PEER_NAME}/tls/ca.crt \
    cli bash -c "peer lifecycle chaincode install /opt/gopath/src/github.com/hyperledger/fabric/peer/network/chaincode.tar.gz" 2>&1)
  echo "$INSTALL_OUTPUT"
  # Extract package ID from the last install
  CC_PACKAGE_ID=$(echo "$INSTALL_OUTPUT" | awk -F 'identifier: ' '{print $2}' | tr -d '\r' | grep -v '^$' || true)
done

echo "Package ID: $CC_PACKAGE_ID"

echo "Starting Chaincode container..."
podman rm -f dwntp-chaincode 2>/dev/null || true
podman run -d --name dwntp-chaincode --network dwntp-network -p 9999:9999 \
  -e CHAINCODE_SERVER_ADDRESS=0.0.0.0:9999 \
  -e CHAINCODE_ID=$CC_PACKAGE_ID \
  -e CHAINCODE_TLS_DISABLED=false \
  -e CHAINCODE_TLS_KEY=/tls/server.key \
  -e CHAINCODE_TLS_CERT=/tls/server.crt \
  -v $PWD/network/crypto-config/peerOrganizations/org1.dwntp.com/peers/peer0.org1.dwntp.com/tls/:/tls:z \
  -e RUST_LOG=info \
  dwntp-chaincode:latest

echo "Approving and Committing Chaincode..."
podman exec cli bash -c "peer lifecycle chaincode approveformyorg -o orderer.dwntp.com:7050 --ordererTLSHostnameOverride orderer.dwntp.com --tls --cafile \$ORDERER_CA --channelID dwntpchannel --name dwntp --version 1.0 --sequence 1 --package-id $CC_PACKAGE_ID"
PEER_ARGS=""
for i in $(seq 0 $((NUM_PEERS-1))); do
  PEER_NAME="peer${i}.org1.dwntp.com"
  PEER_ARGS="$PEER_ARGS --peerAddresses ${PEER_NAME}:7051 --tlsRootCertFiles /opt/gopath/src/github.com/hyperledger/fabric/peer/crypto/peerOrganizations/org1.dwntp.com/peers/${PEER_NAME}/tls/ca.crt"
done

podman exec cli bash -c "peer lifecycle chaincode commit -o orderer.dwntp.com:7050 --ordererTLSHostnameOverride orderer.dwntp.com --tls --cafile \$ORDERER_CA --channelID dwntpchannel --name dwntp --version 1.0 --sequence 1 $PEER_ARGS"

echo "Done!"
