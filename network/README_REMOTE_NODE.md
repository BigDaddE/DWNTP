# Remote Peer / Internet Node Setup

This guide is only for one thing:

- running the DWNTP Fabric network across two different machines over the internet
- where the main machine runs the orderer, chaincode service, and `peer0`
- and a second machine runs `peer1` as a real remote node

It does **not** describe the local benchmark flow or the general project setup.

## What This Setup Does

The remote-node flow gives you a minimal distributed Fabric topology:

- **Coordinator / Main host**
  - runs `orderer.dwntp.com`
  - runs `peer0.org1.dwntp.com`
  - runs `dwntp-chaincode`
  - runs the `cli` container
  - generates crypto material and channel artifacts
  - onboards the remote peer into the channel

- **Remote host**
  - runs `peer1.org1.dwntp.com`
  - runs a local `cli` container bound to `peer1`
  - exposes that peer on the internet

After onboarding, both peers participate in the same channel and see the same ledger state.

## Supported Machines

This flow is intended to work on:

- **Mac + Mac**
- **Mac + Linux**
- **Linux + Linux**

Operationally:

- on **Mac**, the normal runtime is **Docker Desktop**
- on **Linux**, the normal runtime is **Docker** or **Podman**

The helper scripts detect Docker or Podman automatically.

## Scripts Used

This flow uses:

- [generate.sh](/Users/erikrosen/Documents/New%20project/DWNTP/network/generate.sh)
- [redeploy.sh](/Users/erikrosen/Documents/New%20project/DWNTP/network/redeploy.sh)
- [setup_remote_peer.sh](/Users/erikrosen/Documents/New%20project/DWNTP/network/setup_remote_peer.sh)
- [onboard_remote_peer.sh](/Users/erikrosen/Documents/New%20project/DWNTP/network/onboard_remote_peer.sh)

## Network Model

Use this model:

- Main host has a reachable public IP or DNS name
- Remote host has a reachable public IP or DNS name
- The main host exposes:
  - `7050` for the orderer
  - `7053` for the orderer admin API
  - `7051` for `peer0`
  - `9999` for the chaincode service
- The remote host exposes:
  - `7061` for `peer1`

If you are doing NAT/port forwarding, the public ports must map to those internal ports exactly.

## Prerequisites

### Main host

You need:

- Docker or Podman running
- Go installed
- the repo checked out
- ports `7050`, `7053`, `7051`, `9999` reachable from the internet

### Remote host

You need:

- Docker or Podman running
- the repo checked out or at least the generated deployment bundle copied over
- port `7061` reachable from the internet

## Step-by-Step

### 1. Prepare the main host

On the main host:

```bash
cd "/path/to/DWNTP"
./network/generate.sh 2
./network/redeploy.sh 1
```

What this does:

- generates crypto material for 2 peers
- creates the channel block
- starts the orderer
- starts `peer0`
- starts the chaincode service
- commits the chaincode on the main host

At this point, only the local part of the network is up.

### 2. Build the remote peer bundle

On the main host:

```bash
./network/setup_remote_peer.sh 1 <REMOTE_PUBLIC_HOST> <MAIN_PUBLIC_HOST>
```

Example:

```bash
./network/setup_remote_peer.sh 1 203.0.113.25 198.51.100.10
```

This creates:

- [deploy_peer1](/Users/erikrosen/Documents/New%20project/DWNTP/network/deploy_peer1)

That folder contains:

- the peer crypto material for `peer1`
- a `start_remote.sh` script
- a `stop_remote.sh` script
- a `node.env` file with ports and hostnames

### 3. Move the bundle to the remote host

You have two options.

#### Option A: Manual copy

Copy `network/deploy_peer1` to the remote host by `scp`, `rsync`, or similar.

#### Option B: SSH-assisted setup

If the remote host is reachable by SSH, run this from the main host:

```bash
./network/setup_remote_peer.sh 1 <REMOTE_PUBLIC_HOST> <MAIN_PUBLIC_HOST> user@<REMOTE_PUBLIC_HOST>
```

This does three things:

- uploads the bundle
- starts the remote peer
- then runs the onboarding step from the main host

If this path works, you can skip steps 4 and 5 below.

### 4. Start the remote peer

On the remote host, inside the copied bundle directory:

```bash
./start_remote.sh
```

This starts:

- `peer1.org1.dwntp.com`
- `cli`

It also injects host mappings so the remote peer can reach:

- `orderer.dwntp.com`
- `peer0.org1.dwntp.com`
- `dwntp-chaincode`

through the main host's public address.

The local `cli` container is mounted with the org user MSP and orderer CA from the coordinator bundle.
That means `dwntp-client` can be run on the remote host as well, as long as you use the freshly generated bundle.

### 5. Onboard the remote peer

Back on the main host:

```bash
./network/onboard_remote_peer.sh 1 <REMOTE_PUBLIC_HOST>
```

Example:

```bash
./network/onboard_remote_peer.sh 1 203.0.113.25
```

This step:

- waits for the remote peer to become reachable
- joins `peer1` to the channel
- installs the chaincode package on `peer1`

After that, the remote peer is a real participant in the Fabric channel.

## Test the Network

After the remote peer is onboarded, test from the main host:

```bash
cargo run --bin dwntp-client -- --user "User1" log-event \
  --rtu-id "RTU-A1" \
  --event-name "InternetTest" \
  --event-desc "Test over the internet"
```

Then:

```bash
cargo run --bin dwntp-client -- get-all-events
```

If the remote peer is connected correctly, it should replicate the same ledger data.

## Verify Containers

### Main host

Run:

```bash
docker ps
```

or:

```bash
podman ps
```

Expected containers:

- `orderer.dwntp.com`
- `peer0.org1.dwntp.com`
- `cli`
- `dwntp-chaincode`

### Remote host

Expected container:

- `peer1.org1.dwntp.com`

## Ports Summary

### Main host

- `7050` -> orderer
- `7053` -> orderer admin
- `7051` -> peer0
- `9999` -> external chaincode

### Remote host

- `7061` -> peer1

## Important Limitations

This is still a minimal topology.

It is:

- one orderer
- one organization
- one remote peer
- suitable for an internet-based functional test

It is not:

- a production deployment
- multi-org governance
- hardened internet-facing Fabric infrastructure

## Troubleshooting

### `Neither docker nor podman is reachable`

Start Docker Desktop on Mac, or start Docker/Podman on Linux.

### Remote peer never onboards

Check:

- the remote host really exposes `7061`
- the main host can reach `<REMOTE_PUBLIC_HOST>:7061`
- the remote peer container is actually running

### Chaincode works locally but remote peer does not stay connected

Check:

- main host ports `7050`, `7053`, `7051`, and `9999`
- remote host port `7061`
- host firewall rules
- router/NAT forwarding rules

### TLS or hostname errors

Use stable public DNS names if possible instead of changing raw IPs repeatedly.
