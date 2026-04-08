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
    cli bash -c "peer lifecycle chaincode install /opt/gopath/src/github.com/hyperledger/fabric/peer/network/chaincode.tar.gz")
  echo "$INSTALL_OUTPUT"
done
CC_PACKAGE_ID="dwntp_1.0:ed1d1f6719cea1de44ac17f6e571d85727d55f208f9ad7f2209ea20a8777c470"

echo "Package ID: $CC_PACKAGE_ID"

echo "Approving and Committing Chaincode..."
podman exec cli bash -c "peer lifecycle chaincode approveformyorg -o orderer.dwntp.com:7050 --ordererTLSHostnameOverride orderer.dwntp.com --tls --cafile \$ORDERER_CA --channelID dwntpchannel --name dwntp --version 1.0 --sequence 1 --package-id $CC_PACKAGE_ID"
PEER_ARGS=""
for i in $(seq 0 $((NUM_PEERS-1))); do
  PEER_NAME="peer${i}.org1.dwntp.com"
  PEER_ARGS="$PEER_ARGS --peerAddresses ${PEER_NAME}:7051 --tlsRootCertFiles /opt/gopath/src/github.com/hyperledger/fabric/peer/crypto/peerOrganizations/org1.dwntp.com/peers/${PEER_NAME}/tls/ca.crt"
done

podman exec cli bash -c "peer lifecycle chaincode commit -o orderer.dwntp.com:7050 --ordererTLSHostnameOverride orderer.dwntp.com --tls --cafile \$ORDERER_CA --channelID dwntpchannel --name dwntp --version 1.0 --sequence 1 $PEER_ARGS"

echo "Done!"
