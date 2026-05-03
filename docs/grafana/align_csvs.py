#!/usr/bin/env python3
import csv
import glob
import os
import re
import sys
from datetime import datetime

# ── Configuration ─────────────────────────────────────────────────────────────

TIMESTAMPS_FILE = "run_timestamps.csv"


# ── Helpers ───────────────────────────────────────────────────────────────────


def get_metric_name(filename):
    """Derive a clean metric name from a Grafana export filename.

    Grafana appends a long suffix like '-data-as-joinbyfield-2026-04-20_...'
    to every export. We strip that and normalize the remainder.
    """
    name = filename.split("-data-")[0]
    return name.lower().replace(" ", "_").replace("/", "_").replace("-", "_")


# ── Main ──────────────────────────────────────────────────────────────────────


def main():
    base_dir = os.path.dirname(os.path.abspath(__file__))

    # 1. Read run_timestamps.csv -----------------------------------------------

    timestamps_path = os.path.join(base_dir, TIMESTAMPS_FILE)
    if not os.path.exists(timestamps_path):
        print(f"Could not find {TIMESTAMPS_FILE}.")
        print("Please run benchmarks via run-multi-benchmarks.sh to generate it.")
        sys.exit(1)

    entries = []
    with open(timestamps_path, "r") as f:
        reader = csv.DictReader(f)
        for row in reader:
            try:
                entries.append(
                    {
                        "nodes": int(row["nodes"]),
                        "phase": row["phase"],
                        "time": datetime.strptime(
                            row["start_time"], "%Y-%m-%d %H:%M:%S"
                        ),
                    }
                )
            except (ValueError, KeyError) as e:
                print(f"Could not parse row in {TIMESTAMPS_FILE}: {row} ({e})")
                sys.exit(1)

    if not entries:
        print(f"{TIMESTAMPS_FILE} is empty.")
        sys.exit(1)

    base_time = entries[0]["time"]
    last_time = entries[-1]["time"]

    # Convert absolute timestamps to relative seconds from base_time
    for e in entries:
        e["t"] = int((e["time"] - base_time).total_seconds())

    # Group phases by node count, preserving order
    runs = {}  # nodes -> [{"phase": str, "t": int}, ...]
    for e in entries:
        runs.setdefault(e["nodes"], []).append({"phase": e["phase"], "t": e["t"]})

    # Build (start_t, end_t) for every (nodes, phase) pair.
    all_phases = [
        (nodes, p["phase"], p["t"]) for nodes, phases in runs.items() for p in phases
    ]

    phase_ranges = {}  # (nodes, phase) -> (start_t, end_t)
    for i, (nodes, phase, start_t) in enumerate(all_phases):
        end_t = all_phases[i + 1][2] - 1 if i + 1 < len(all_phases) else float("inf")
        phase_ranges[(nodes, phase)] = (start_t, end_t)

    # Output folder
    folder_name = (
        f"{base_time.strftime('%Y%m%d_%H%M%S')}"
        f"-to-"
        f"{last_time.strftime('%Y%m%d_%H%M%S')}"
    )
    target_dir = os.path.join(base_dir, folder_name)
    os.makedirs(target_dir, exist_ok=True)

    print(f"Base time: {base_time}")
    print(f"Output directory: {folder_name}")

    # 2. Find and clean all Grafana CSVs ---------------------------------------

    csv_files = glob.glob(os.path.join(base_dir, "*.csv"))
    csv_files = [f for f in csv_files if os.path.basename(f) != TIMESTAMPS_FILE]

    clean_files = []

    for f_path in csv_files:
        filename = os.path.basename(f_path)

        if "data" not in filename:
            print(f"Skipping (no 'data' in filename): {filename}")
            continue

        metric_name = get_metric_name(filename)
        out_dir = os.path.join(target_dir, metric_name)
        os.makedirs(out_dir, exist_ok=True)

        with open(f_path, "r") as infile:
            reader = csv.reader(infile)
            headers = next(reader, None)

            if headers is None:
                print(f"Skipping (empty file): {filename}")
                continue

            if "Time" not in headers:
                print(f"Skipping (no 'Time' column): {filename}")
                continue

            time_idx = headers.index("Time")

            # Rename CPU columns
            if metric_name == "fabric_process_cpu_usage":
                for i, h in enumerate(headers):
                    if "fabric-orderer" in h:
                        headers[i] = "OrdererCPU"
                    elif "fabric-peers" in h:
                        headers[i] = "PeersCPU"
                    elif "node_exporter" in h:
                        headers[i] = "HostCPU"

            cleaned_rows = []
            units_map = {}  # column index -> unit

            for row in reader:
                if not row or len(row) <= time_idx or not row[time_idx]:
                    continue

                try:
                    time_str = row[time_idx]
                    try:
                        ts = float(time_str)
                        if ts > 2e10:  # Handle milliseconds
                            ts /= 1000.0
                        row_time = datetime.fromtimestamp(ts)
                    except ValueError:
                        row_time = datetime.strptime(time_str, "%Y-%m-%d %H:%M:%S")
                        
                    diff = int((row_time - base_time).total_seconds())
                    if diff < 0:
                        continue
                    row[time_idx] = str(diff)
                except ValueError:
                    continue

                for i in range(len(row)):
                    if i != time_idx:
                        val = row[i].strip().replace(",", "")

                        # Handle undefined / null
                        if val.lower() in ["undefined", "null", "none", ""]:
                            row[i] = ""
                            continue

                        # Handle %
                        if val.endswith("%"):
                            val = val[:-1]
                            units_map.setdefault(i, "%")

                        # Extract numeric + unit
                        match = re.match(r"^([-+]?\d*\.?\d+)(?:\s*([a-zA-Z]+))?$", val)
                        if match:
                            number, unit = match.groups()
                            row[i] = number
                            if unit:
                                units_map.setdefault(i, unit)
                        else:
                            row[i] = val

                cleaned_rows.append(row)

        # Append units to headers
        for idx, unit in units_map.items():
            if unit and f"({unit})" not in headers[idx]:
                headers[idx] = f"{headers[idx]} ({unit})"

        clean_files.append((metric_name, out_dir, headers, cleaned_rows, time_idx))
        print(f"Processed: {filename} -> {metric_name}/")

    print(f"\nAligned {len(clean_files)} files to start time 0.")

    # 3. Split by run and phase ------------------------------------------------

    for metric_name, out_dir, headers, rows, time_idx in clean_files:
        for nodes, phases in runs.items():
            node_dir = os.path.join(out_dir, f"{nodes}_nodes")
            os.makedirs(node_dir, exist_ok=True)

            for p in phases:
                phase = p["phase"]
                start_t, end_t = phase_ranges[(nodes, phase)]

                out_path = os.path.join(node_dir, f"{phase}.csv")
                with open(out_path, "w", newline="") as out_f:
                    writer = csv.writer(out_f)
                    writer.writerow(headers)

                    for row in rows:
                        try:
                            t = int(row[time_idx])
                        except ValueError:
                            continue
                        if start_t <= t <= end_t:
                            mod_row = list(row)
                            mod_row[time_idx] = str(t - start_t)
                            writer.writerow(mod_row)

        print(f"  Split: {metric_name}")

    print("\nAll done! Separated datasets are ready for pgfplots.")


if __name__ == "__main__":
    main()
