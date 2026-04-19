# Security Test: Forged-Identity Event Submission

This document records a black-box security test executed against a live DWNTP
network running across two hosts over the public internet. The goal was to
verify that Hyperledger Fabric's MSP layer rejects transactions signed by an
identity that is *not* anchored in `Org1MSP`'s trusted root CAs.

## Purpose

The DWNTP threat model assumes that only holders of a valid `Org1MSP`
credential — i.e., an X.509 certificate issued by the organization's CA
generated via `cryptogen` — can submit `LogEvent` transactions to the
`dwntpchannel` ledger. This test attempts to bypass that assumption by
generating a standalone keypair and a self-issued certificate chain, then
submitting a forged `LogEvent` proposal while claiming to be `Org1MSP`.

**Expected outcome:** the endorsing peer rejects the proposal during MSP
validation; no block is written.

## Environment

| Role         | Host                    | Container(s)                        |
|--------------|-------------------------|-------------------------------------|
| Coordinator  | `192.168.109.121`       | `orderer.dwntp.com`, `peer0.org1.dwntp.com`, `dwntp-chaincode`, `cli` |
| Remote peer  | `192.168.105.120`       | `peer1.org1.dwntp.com`, `cli`       |

The test was run from the **remote host**, using its local `cli` container
(built by `network/setup_remote_peer.sh` after commit `4d619cd`). The `cli`
container holds the `hyperledger/fabric-tools:2.5` image with the `peer`
binary and is attached to `dwntp-remote` with host mappings for the
coordinator's public IP.

Before starting, the legitimate ledger contained two events logged by
`User1` and `User2` over the internet:

```json
[
  { "event_name": "InternetTest", "source_mtu": "VXNlcjFAb3JnMS5kd250cC5jb20=", ... },
  { "event_name": "InternetTest", "source_mtu": "VXNlcjJAb3JnMS5kd250cC5jb20=", ... }
]
```

## Threat Model

The attacker in this scenario:

- has shell access to a host that can reach the peer over the network
- does **not** have access to any private key issued by `org1.dwntp.com`'s CA
- does **not** have access to the coordinator's CA signing key
- can freely generate their own keys and X.509 certificates

The attacker's goal is to get *any* forged event into the ledger while
claiming to originate from `Org1MSP`.

## Attack Construction

Hyperledger Fabric's MSP evaluates the `creator` field of a `SignedProposal`
by:

1. Parsing the serialized identity into `(MSPID, cert_bytes)`.
2. Looking up the MSPID's configuration in the channel config.
3. Verifying `cert_bytes` chains up to one of that MSP's trusted root CAs.
4. Applying Node OU rules (e.g., requiring `OU=client` for a client identity).
5. Verifying the proposal's signature using the identity's public key.

A forged cert must therefore be structurally indistinguishable from a real
Fabric end-entity cert (otherwise the `peer` CLI tool refuses to load the
MSP locally, before any network traffic is generated). The realistic attack
path is to build a complete self-rooted chain that *mimics* Fabric's
conventions, sign with it, and let the remote peer reject it on the basis
of the untrusted root CA — which is the only check the attacker cannot
bypass without stealing the real CA key.

Three iterations were needed to get the attack far enough for the peer to
evaluate it properly:

1. **`openssl req -x509` (self-signed)** — rejected locally by the `peer`
   CLI with *"An X509 certificate with Basic Constraint: Certificate
   Authority equals true cannot be used as an identity"*, because the
   default `-x509` flow produces a `CA:TRUE` cert.
2. **CA + end-entity chain, with `extendedKeyUsage=clientAuth`** — rejected
   locally by the `peer` CLI with *"x509: certificate specifies an
   incompatible key usage"*, because real Fabric user certs carry no EKU.
3. **CA + end-entity chain, extensions mirroring real Fabric user certs**
   (`basicConstraints=critical,CA:FALSE`, `keyUsage=critical,digitalSignature`,
   no EKU) — passed local MSP loading, reached `peer0`, and was rejected
   there on the basis of an unknown issuing CA.

Iteration 3 is the interesting case and is the script recorded below.

## Script

Run from the remote host (with a working local `cli` container):

```bash
set -e
W=/tmp/malicious_msp
rm -rf "$W"
mkdir -p "$W"/keystore "$W"/signcerts "$W"/cacerts "$W"/admincerts

# 1. Attacker's self-rooted CA
openssl ecparam -name prime256v1 -genkey -noout -out "$W/ca_key.pem"
openssl req -new -x509 -key "$W/ca_key.pem" -out "$W/cacerts/cert.pem" -days 365 \
  -subj "/C=US/ST=California/L=San Francisco/O=org1.dwntp.com/CN=ca.evil.dwntp.com"

# 2. Attacker user keypair + CSR
openssl ecparam -name prime256v1 -genkey -noout -out "$W/keystore/priv_sk"
openssl req -new -key "$W/keystore/priv_sk" -out "$W/user.csr" \
  -subj "/C=US/ST=California/L=San Francisco/OU=client/CN=malicious@org1.dwntp.com"

# 3. Sign CSR with the fake CA, with extensions matching real Fabric users
cat > "$W/ext.cnf" <<'EOF'
basicConstraints=critical,CA:FALSE
keyUsage=critical,digitalSignature
EOF

openssl x509 -req -in "$W/user.csr" -CA "$W/cacerts/cert.pem" -CAkey "$W/ca_key.pem" \
  -CAcreateserial -out "$W/signcerts/cert.pem" -days 365 -extfile "$W/ext.cnf"

cp "$W/signcerts/cert.pem" "$W/admincerts/cert.pem"

# 4. NodeOU config (so MSP treats OU=client as a client role)
cat > "$W/config.yaml" <<EOF
NodeOUs:
  Enable: true
  ClientOUIdentifier:
    Certificate: cacerts/cert.pem
    OrganizationalUnitIdentifier: client
EOF

# 5. Upload the fake MSP into the cli container
docker exec cli rm -rf /tmp/malicious_msp
docker exec cli mkdir -p /tmp/malicious_msp/keystore /tmp/malicious_msp/signcerts \
                         /tmp/malicious_msp/cacerts /tmp/malicious_msp/admincerts
docker cp "$W/keystore/priv_sk"     cli:/tmp/malicious_msp/keystore/priv_sk
docker cp "$W/signcerts/cert.pem"   cli:/tmp/malicious_msp/signcerts/cert.pem
docker cp "$W/cacerts/cert.pem"     cli:/tmp/malicious_msp/cacerts/cert.pem
docker cp "$W/admincerts/cert.pem"  cli:/tmp/malicious_msp/admincerts/cert.pem
docker cp "$W/config.yaml"          cli:/tmp/malicious_msp/config.yaml

# 6. Attempt the forged LogEvent, claiming MSPID=Org1MSP
TS=$(date +%s%3N)
docker exec -e CORE_PEER_MSPCONFIGPATH=/tmp/malicious_msp \
            -e CORE_PEER_LOCALMSPID=Org1MSP cli \
  bash -c "peer chaincode invoke \
    -o orderer.dwntp.com:7050 --tls --cafile \$ORDERER_CA \
    -C dwntpchannel -n dwntp \
    -c '{\"function\":\"LogEvent\",\"Args\":[\"RTU-EVIL\",\"MaliciousAction\",\"Forged by unauth attacker\",\"$TS\"]}'"
```

## Observed Result

Forged certificate (successfully built):

```
Issuer : C=US, ST=California, L=San Francisco, O=org1.dwntp.com, CN=ca.evil.dwntp.com
Subject: C=US, ST=California, L=San Francisco, OU=client, CN=malicious@org1.dwntp.com
X509v3 Basic Constraints: critical  CA:FALSE
X509v3 Key Usage:         critical  Digital Signature
```

Response from the endorsing peer (`peer0.org1.dwntp.com`):

```
Error: error endorsing invoke: rpc error: code = Unknown desc = error validating proposal:
access denied: channel [dwntpchannel] creator org unknown, creator is malformed
 - proposal response: <nil>
```

The transaction never reached the orderer and no block was written.

Post-attack ledger state, queried via the legitimate Rust client:

```bash
cargo run --bin dwntp-client -- --user Admin get-all-events
```

```json
[
  { "rtu_id": "RTU-A1", "event_name": "InternetTest", "source_mtu": "VXNlcjFAb3JnMS5kd250cC5jb20=", ... },
  { "rtu_id": "RTU-A1", "event_name": "InternetTest", "source_mtu": "VXNlcjJAb3JnMS5kd250cC5jb20=", ... }
]
```

No `RTU-EVIL` event is present. `source_mtu` values correspond to
legitimate `User1@org1.dwntp.com` and `User2@org1.dwntp.com` base64-encoded
CNs. The attack left no on-chain footprint.

## Interpretation

The rejection message — `creator org unknown, creator is malformed` — is
emitted by Fabric's MSP after it fails to build a certification chain from
the creator cert up to any of `Org1MSP`'s trusted root CAs (the real
`ca.org1.dwntp.com`, baked into the channel configuration by
`configtxgen` at genesis time). Because the attacker's cert was issued by
`ca.evil.dwntp.com`, which no one on the channel trusts, the identity is
treated as belonging to an unknown organization.

This confirms the core integrity property of the system: **event
submission requires possession of a private key whose corresponding
certificate chains up to a CA that was already recognized by the channel
at genesis**. Generating new keys locally is cheap; getting them
recognized by an already-running MSP is not — it requires either stealing
the real CA private key or pushing a successful channel configuration
update.

## Residual Attack Vectors

This test validates one specific attack path. It does not cover:

1. **Theft of a valid private key.** All identities in
   `crypto-config/.../users/` are real, valid MSPs. Any process that can
   read those files can log events under that user's identity. The
   current `setup_remote_peer.sh` copies the *entire* `users/` tree into
   the remote bundle, which means the remote host holds private keys for
   `Admin`, `User1`, and `User2` even if only one of them is supposed to
   operate there. Mitigation: scope the bundle to a single user.
2. **Compromise of the organization CA.** The CA private key under
   `network/crypto-config/peerOrganizations/org1.dwntp.com/ca/` can
   issue arbitrary new identities that *will* be accepted. It should
   never be present on a remote host and ideally should be moved offline
   after initial material generation.
3. **Channel configuration update with an attacker's CA.** An actor
   holding a channel-admin-role identity could, in principle, sign a
   config update that adds a new root CA to `Org1MSP`. Mitigation: keep
   admin keys off remote hosts; audit config-update transactions.
4. **TLS-layer attacks.** This test used the real Org1 TLS root to
   establish the connection to the peer. Attacks on the TLS trust
   material (e.g., a stolen TLS CA key) are a separate failure mode not
   exercised here.

## Cleanup

After the test, the fake MSP was removed from both the host and the
`cli` container:

```bash
rm -rf /tmp/malicious_msp
docker exec cli rm -rf /tmp/malicious_msp
```

## Summary

| Step                                            | Result                |
|-------------------------------------------------|-----------------------|
| Generate standalone EC keypair                  | ok                    |
| Build fake CA + end-entity cert chain           | ok                    |
| Load fake MSP into `cli` container              | ok                    |
| Submit `LogEvent` proposal claiming `Org1MSP`   | rejected by `peer0`   |
| Malicious event written to ledger               | **no**                |

The MSP-based identity verification in DWNTP behaves correctly: an
attacker without access to `Org1MSP`'s real CA cannot forge events, even
when they produce a structurally valid Fabric-style certificate. The
test reinforces that the practical attack surface lies in key custody
(protecting existing private keys and the CA key) rather than in the
cryptographic identity checks themselves.
