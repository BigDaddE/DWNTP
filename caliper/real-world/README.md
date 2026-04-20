# DWNTP Real-World Caliper Benchmark (2 nodes over public internet)

This directory holds a **separate** Caliper harness that targets the live
DWNTP deployment: the coordinator host (running `orderer.dwntp.com` and
`peer0.org1.dwntp.com`) and the remote host (running
`peer1.org1.dwntp.com`). It is deliberately isolated from the localhost
`caliper/benchmark-config.yaml` suite so you cannot accidentally run
5000-TPS ramps against a home internet link.

## Scope

- **2 peers only**: `peer0` on the coordinator, `peer1` on the remote host.
- **1 orderer**: `orderer.dwntp.com`, on the coordinator.
- **2 client identities**: `User1@org1.dwntp.com` and `User2@org1.dwntp.com`,
  rotated per transaction so the workload reflects more than one signer.
- **Transport**: public IP → public IP, TLS-server-auth enforced, matching
  what `docs/network_encryption_verification.md` documents.

## Files

| File | Purpose |
|------|---------|
| `network-config.template.yaml` | Caliper connection profile with `${PLACEHOLDERS}` for host/port/channel. |
| `benchmark-config.yaml`        | Internet-tuned round definitions (warmup → capacity ramp → query). |
| `workload/realLogEvent.js`     | Write workload: 25-RTU pool, 7 event types, User1/User2 rotation. |
| `workload/realQueryEvent.js`   | Read-only `GetAllEvents` workload with User1/User2 rotation. |
| `realworld.env.example`        | Template for the site-specific env file. |
| `run-realworld-benchmark.sh`   | Renders the config, sanity-checks reachability, launches Caliper. |
| `.gitignore`                   | Keeps rendered configs, logs, and `realworld.env` out of git. |

## Prerequisites

1. The 2-node deployment is up and healthy:
   - Coordinator: `docker compose up -d` in repo root (orderer, peer0,
     chaincode, cli).
   - Remote: `./start_remote.sh` on the remote host (peer1 + local cli),
     after the peer was onboarded via
     `./network/onboard_remote_peer.sh 1 <REMOTE_PUBLIC_IP>`.
2. Port forwards on both hosts point to the right containers
   (`ORDERER_PORT`, `PEER0_PORT` on the coordinator;
   `PEER1_PORT` on the remote).
3. `crypto-config/` is present on whichever machine will *run Caliper*,
   at `network/crypto-config/...` relative to the repo root. Caliper
   needs the local `User1`/`User2` MSP materials and the Org1/Orderer
   TLS root certificates. Typically you will run this on the coordinator
   host where `cryptogen` generated the material.
4. Node.js dependencies are installed once:
   ```bash
   cd caliper
   npm install
   ```

## One-time setup

```bash
cd caliper/real-world
cp realworld.env.example realworld.env
$EDITOR realworld.env        # set MAIN_PUBLIC_HOST, REMOTE_PUBLIC_HOST, ports
```

`realworld.env` is gitignored because it encodes the public IPs of your
hosts.

## Running the benchmark

```bash
cd caliper/real-world
./run-realworld-benchmark.sh
```

The driver:

1. Sources `realworld.env` and asserts every required variable is set
   (and not left at `CHANGE_ME_*`).
2. Runs a 4-second TCP probe against each endpoint (orderer, peer0,
   peer1) and warns if one is unreachable. It does not abort — DNS and
   routing sometimes come up late — but a warning here usually means
   the benchmark will fail.
3. Renders `network-config.template.yaml` → `network-config.rendered.yaml`
   by substituting the placeholders. Fails loudly if any `${...}`
   remains unresolved.
4. Runs `npx caliper bind --caliper-bind-sut fabric:2.2`.
5. Launches `npx caliper launch manager` with the rendered network
   config and `benchmark-config.yaml`, writing the report to
   `reports/<UTC timestamp>/report.html`.

## Round profile

Rates are scaled for home-broadband RTT (~70 ms) and a single-org,
single-orderer Fabric deployment. Do not lift them from the localhost
suite — those numbers assume a zero-latency loopback.

| Round | Stage | Rate | Duration | Workload |
|-------|-------|------|----------|----------|
| 1 | Warmup           | 2 TPS fixed      | 30 s  | writes  |
| 2 | Baseline-5       | 5 TPS fixed      | 120 s | writes  |
| 3 | Moderate-15      | 15 TPS fixed     | 90 s  | writes  |
| 4 | Peak-30          | 30 TPS fixed     | 60 s  | writes  |
| 5 | Burst-60         | 60 TPS fixed     | 30 s  | writes  |
| 6 | Capacity-Ramp    | 1 → 100 TPS      | 180 s | writes  |
| 7 | Query-Steady-5   | 5 TPS fixed      | 60 s  | reads   |
| 8 | Query-Ramp       | 1 → 50 TPS       | 120 s | reads   |

The rounds build on each other — warmup stabilizes TLS and gossip, the
fixed-rate stages give clean latency numbers at steady state, the ramp
rounds find the saturation point of the internet link plus the peers'
endorsement/commit pipeline, and the query rounds size the read path
independently of the write path.

## Workload characteristics

**Writes (`workload/realLogEvent.js`)**
- `rtuId` drawn uniformly from a stable 25-entry pool (A1–E5) so state
  keys accumulate repeat hits, which is more realistic than the
  localhost benchmark's 10-entry random pool.
- `eventName` drawn uniformly from `SetVoltage`, `OpenBreaker`,
  `CloseBreaker`, `EnableRelay`, `DisableRelay`, `ReadMeter`, `Reset`.
- `eventTimestamp` built from `Date.now() * 1e5 + workerIndex * 1e4 + txIndex`
  so uniqueness holds across workers and rounds (the chaincode rejects
  duplicate timestamps).
- `invokerIdentity` alternates between `User1@org1.dwntp.com` and
  `User2@org1.dwntp.com` per transaction.

**Reads (`workload/realQueryEvent.js`)**
- Calls `GetAllEvents` read-only and rotates `User1`/`User2`.

## Observability

`benchmark-config.yaml` enables the Prometheus transaction monitor on
port **3002** (the localhost benchmark uses 3001; they can run
side-by-side). The docker resource monitor is deliberately omitted —
the peers Caliper is hitting live on remote hosts, so a local docker
monitor cannot see them. Use the coordinator's and remote's own
`peer`/`orderer` operations endpoints (`9443`/`9453` per the deploy
scripts) for peer-side resource metrics.

## Troubleshooting

- **`REQUEST_TIMEOUT` on round 1.** Almost always a firewall/NAT
  problem. Re-check `ORDERER_PORT`, `PEER0_PORT`, `PEER1_PORT` forwards.
  From the coordinator host, run
  `nc -vz $REMOTE_PUBLIC_HOST $PEER1_PORT` and vice versa.
- **`x509: certificate signed by unknown authority`.** The TLS CA cert
  under `network/crypto-config/.../tlscacerts/` doesn't match what the
  peer presents. You almost certainly regenerated `crypto-config/` on
  one host and forgot to re-sync it to the other. Regenerate once and
  rebundle via `./network/setup_remote_peer.sh`.
- **`access denied: channel ... creator org unknown`.** The `User1`/
  `User2` MSP material Caliper is using is not the one `Org1MSP` was
  built from. Same cause as above.
- **Commits time out under high TPS.** You've saturated the uplink or
  the orderer's batch timeout. Lower `finishingTps` on round 6 and
  rerun — that's exactly the data point the ramp round is meant to
  produce.
- **`nc` not found on macOS/minimal hosts.** The reachability probe
  is skipped (you get a note). Either install `nc` or test reachability
  manually before running.

## Not included

- **Multi-org trials.** DWNTP currently has one org, one orderer, so
  there is only one MSP to rotate through.
- **Long-running soak tests.** The longest round is 3 minutes. For a
  24-hour soak, wrap `run-realworld-benchmark.sh` in a loop externally
  and archive the per-run `reports/` directories.
- **Chaos/fault injection.** Kill peers or drop the link with `tc
  netem` between runs if you need resilience data; Caliper itself has
  no fault-injection layer here.
