#!/bin/bash
set -e

./network/start_network.sh

echo "Waiting for network to boot..."
sleep 5

echo "Joining Channel..."
podman exec cli bash -c "osnadmin channel join --channelID dwntpchannel --config-block /opt/gopath/src/github.com/hyperledger/fabric/peer/channel-artifacts/dwntpchannel.block -o orderer.dwntp.com:7053 --ca-file \$ORDERER_CA --client-cert /opt/gopath/src/github.com/hyperledger/fabric/peer/crypto/ordererOrganizations/dwntp.com/users/Admin@dwntp.com/tls/client.crt --client-key /opt/gopath/src/github.com/hyperledger/fabric/peer/crypto/ordererOrganizations/dwntp.com/users/Admin@dwntp.com/tls/client.key"
podman exec cli bash -c "peer channel join -b /opt/gopath/src/github.com/hyperledger/fabric/peer/channel-artifacts/dwntpchannel.block"

echo "Waiting for Raft leader election on orderer..."
sleep 5

echo "Installing Chaincode..."
INSTALL_OUTPUT=$(podman exec cli bash -c "peer lifecycle chaincode install /opt/gopath/src/github.com/hyperledger/fabric/peer/network/chaincode.tar.gz")
echo "$INSTALL_OUTPUT"
CC_PACKAGE_ID=$(echo "$INSTALL_OUTPUT" | grep -oP '(?<=Chaincode code package identifier: )dwntp_1.0:[a-f0-9]+' || echo "dwntp_1.0:ed1d1f6719cea1de44ac17f6e571d85727d55f208f9ad7f2209ea20a8777c470")

echo "Package ID: $CC_PACKAGE_ID"

echo "Approving and Committing Chaincode..."
podman exec cli bash -c "peer lifecycle chaincode approveformyorg -o orderer.dwntp.com:7050 --ordererTLSHostnameOverride orderer.dwntp.com --tls --cafile \$ORDERER_CA --channelID dwntpchannel --name dwntp --version 1.0 --sequence 1 --package-id $CC_PACKAGE_ID"
podman exec cli bash -c "peer lifecycle chaincode commit -o orderer.dwntp.com:7050 --ordererTLSHostnameOverride orderer.dwntp.com --tls --cafile \$ORDERER_CA --channelID dwntpchannel --name dwntp --version 1.0 --sequence 1 --peerAddresses peer0.org1.dwntp.com:7051 --tlsRootCertFiles /opt/gopath/src/github.com/hyperledger/fabric/peer/crypto/peerOrganizations/org1.dwntp.com/peers/peer0.org1.dwntp.com/tls/ca.crt"

echo "Done!"
