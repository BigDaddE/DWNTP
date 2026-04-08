const fs = require('fs');
const path = require('path');

// Get number of peers from command line argument, default to 2
const numPeers = parseInt(process.argv[2], 10) || 2;
const outputPath = path.join(__dirname, 'network-config.yaml');

let peersYaml = '';
for (let i = 0; i < numPeers; i++) {
    const port = 7051 + (i * 10);
    peersYaml += `      - endpoint: localhost:${port}
        tlsCACerts:
          path: "../network/crypto-config/peerOrganizations/org1.dwntp.com/peers/peer${i}.org1.dwntp.com/msp/tlscacerts/tlsca.org1.dwntp.com-cert.pem"
        grpcOptions:
          ssl-target-name-override: peer${i}.org1.dwntp.com\n`;
}

const yamlContent = `name: DWNTP Network
version: "2.0.0"

caliper:
  blockchain: fabric

client:
  connection:
    options:
      discovery:
        enabled: false
        asLocalhost: true

channels:
  - channelName: dwntpchannel
    contracts:
      - id: dwntp

organizations:
  - mspid: Org1MSP
    identities:
      certificates:
        - name: "User1@org1.dwntp.com"
          clientPrivateKey:
            path: "../network/crypto-config/peerOrganizations/org1.dwntp.com/users/User1@org1.dwntp.com/msp/keystore/priv_sk"
          clientSignedCert:
            path: "../network/crypto-config/peerOrganizations/org1.dwntp.com/users/User1@org1.dwntp.com/msp/signcerts/User1@org1.dwntp.com-cert.pem"
    peers:
${peersYaml}`;

try {
    fs.writeFileSync(outputPath, yamlContent.trim() + '\n', 'utf8');
    console.log(`Successfully generated network-config.yaml for ${numPeers} peers at ${outputPath}`);
} catch (err) {
    console.error(`Failed to write network-config.yaml: ${err.message}`);
    process.exit(1);
}
