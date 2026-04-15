#!/usr/bin/env python3
import os
import glob
import csv
from datetime import datetime
import sys

def main():
    base_dir = os.path.dirname(os.path.abspath(__file__))
    
    # 1. Find the TPS file to get the base time (from Cumulative Transactions)
    tps_files = glob.glob(os.path.join(base_dir, "**/Cumulative Transactions Over Time*.csv"), recursive=True)
    if not tps_files:
        print("No 'Cumulative Transactions Over Time' CSV found to determine base time.")
        print("Please ensure you export it from Grafana.")
        sys.exit(1)

    tps_file = tps_files[0]
    with open(tps_file, 'r') as f:
        reader = csv.DictReader(f)
        try:
            first_row = next(reader)
        except StopIteration:
            print("TPS file is empty.")
            sys.exit(1)
            
        base_time_str = first_row['Time']
        try:
            base_time = datetime.strptime(base_time_str, "%Y-%m-%d %H:%M:%S")
        except ValueError:
            print(f"Could not parse time format in {tps_file}: {base_time_str}")
            sys.exit(1)

    print(f"Base time determined from TPS file: {base_time}")

    # 2. Find all CSVs
    csv_files = glob.glob(os.path.join(base_dir, "**/*.csv"), recursive=True)

    def sanitize_name(name):
        name = name.split('-data-')[0]
        name = name.lower().replace(' ', '_').replace('/', '_').replace('-', '_')
        if name == "cumulative_transactions_over_time":
            return "tps"
        if name == "fabric_process_cpu_usage":
            return "cpu"
        if name == "cpu":
            return "cpu_overall"
        return name

    clean_files_generated = []

    for f_path in csv_files:
        # Skip already cleaned files or manually split files (e.g., tps_2_nodes.csv)
        filename = os.path.basename(f_path)
        if "clean" in filename or "nodes.csv" in filename: 
            continue
        if "data" not in filename: 
            continue
        
        metric_name = sanitize_name(filename)
        out_dir = os.path.join(base_dir, metric_name)
        os.makedirs(out_dir, exist_ok=True)
        
        out_file = os.path.join(out_dir, f"{metric_name}_clean.csv")
        clean_files_generated.append((metric_name, out_file))
        
        with open(f_path, 'r') as infile, open(out_file, 'w', newline='') as outfile:
            reader = csv.reader(infile)
            writer = csv.writer(outfile)
            
            headers = next(reader)
            
            # Optional: Rename headers for cpu to match old scripts if needed
            if metric_name == "cpu":
                for i, h in enumerate(headers):
                    if "fabric-orderer" in h: headers[i] = "OrdererCPU"
                    elif "fabric-peers" in h: headers[i] = "PeersCPU"
                    elif "node_exporter" in h: headers[i] = "HostCPU"
            
            writer.writerow(headers)
            
            try:
                time_idx = headers.index('Time')
            except ValueError:
                print(f"Skipping {filename}: no 'Time' column found.")
                continue
                
            for row in reader:
                if not row or len(row) <= time_idx or not row[time_idx]: 
                    continue
                try:
                    row_time = datetime.strptime(row[time_idx], "%Y-%m-%d %H:%M:%S")
                    diff = int((row_time - base_time).total_seconds())
                    
                    # Skip rows that occurred BEFORE the base_time (TPS start)
                    if diff < 0:
                        continue
                        
                    row[time_idx] = str(diff)
                except ValueError:
                    pass # Leave time as is if it can't parse
                
                # Clean up percentage signs or byte units from other columns
                for i in range(len(row)):
                    if i != time_idx:
                        val = row[i].strip()
                        if val.endswith('%'):
                            val = val[:-1]
                        val = val.replace(',', '')
                        row[i] = val
                writer.writerow(row)
                
        print(f"Processed: {filename} -> {metric_name}/{metric_name}_clean.csv")
        
    print(f"Finished aligning {len(clean_files_generated)} files to start time 0.")
    
    # 3. Analyze the TPS clean file to find natural splits (resets)
    tps_clean_path = os.path.join(base_dir, "tps", "tps_clean.csv")
    if not os.path.exists(tps_clean_path):
        print("Could not find tps_clean.csv to determine splits.")
        sys.exit(1)

    splits = []
    current_split_start = 0
    prev_val = -1

    with open(tps_clean_path, 'r') as f:
        reader = csv.DictReader(f)
        for row in reader:
            try:
                t = int(row['Time'])
                # Prefer 'Total Transactions (Received)'
                col_name = 'Total Transactions (Received)' if 'Total Transactions (Received)' in row else reader.fieldnames[1]
                val = float(row[col_name])
            except (ValueError, KeyError):
                continue
                
            # A huge drop in cumulative transactions means the network restarted for the next benchmark run.
            # Example: 50,000 transactions suddenly drops to 5.
            if prev_val != -1 and val < prev_val * 0.5: 
                splits.append((current_split_start, t - 1)) # End the previous split right before this time
                current_split_start = t # Start the next split right now
            
            prev_val = val
            
    # Cap the final split at infinity
    splits.append((current_split_start, float('inf')))
    
    node_counts = [2, 4, 8, 16] # Expected topologies
    
    print(f"\nDetected {len(splits)} individual test runs by observing TPS drops:")
    for i, (start_t, end_t) in enumerate(splits):
        nodes = node_counts[i] if i < len(node_counts) else "unknown"
        print(f"  Run {i+1} ({nodes} nodes): Time {start_t} to {end_t}")
        
    if len(splits) > len(node_counts):
        print(f"Warning: Expected {len(node_counts)} test runs but found {len(splits)}. Will only process the first {len(node_counts)}.")
        splits = splits[:len(node_counts)]
        
    print("\nSplitting all aligned CSVs into separate node runs...")
    
    # 4. Apply these splits to all generated _clean.csv files
    for metric_name, clean_file in clean_files_generated:
        if not os.path.exists(clean_file):
            continue
            
        with open(clean_file, 'r') as f:
            reader = csv.reader(f)
            headers = next(reader)
            time_idx = headers.index('Time')
            rows = list(reader)
            
            for i, (start_t, end_t) in enumerate(splits):
                if i >= len(node_counts):
                    break
                nodes = node_counts[i]
                
                out_path = os.path.join(base_dir, metric_name, f"{metric_name}_{nodes}_nodes.csv")
                
                with open(out_path, 'w', newline='') as out_f:
                    writer = csv.writer(out_f)
                    writer.writerow(headers)
                    
                    for row in rows:
                        t = int(row[time_idx])
                        if start_t <= t <= end_t:
                            # Shift the time so each run segment ALSO starts cleanly at 0 seconds
                            mod_row = list(row)
                            mod_row[time_idx] = str(t - start_t)
                            writer.writerow(mod_row)
                            
        print(f"  Created separate runs for: {metric_name}")

    print("\nAll done! Separated datasets are ready for pgfplots.")

if __name__ == "__main__":
    main()
