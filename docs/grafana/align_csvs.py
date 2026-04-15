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

    processed_count = 0
    for f_path in csv_files:
        filename = os.path.basename(f_path)
        if "clean" in filename or "nodes.csv" in filename: 
            continue
        if "data" not in filename: 
            continue
        
        metric_name = sanitize_name(filename)
        out_dir = os.path.join(base_dir, metric_name)
        os.makedirs(out_dir, exist_ok=True)
        
        out_file = os.path.join(out_dir, f"{metric_name}_clean.csv")
        
        with open(f_path, 'r') as infile, open(out_file, 'w', newline='') as outfile:
            reader = csv.reader(infile)
            writer = csv.writer(outfile)
            
            headers = next(reader)
            
            # Rename headers for cpu to match old scripts if needed
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
        processed_count += 1
        
    print(f"Finished aligning {processed_count} files to start time 0.")

if __name__ == "__main__":
    main()
