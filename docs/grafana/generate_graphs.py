#!/usr/bin/env python
import csv
import os

import sys
import glob

# Will be set in main()
BASE_DIR = ""
OUTPUT_FILE = "generated_graphs.tex"

PHASE_ORDER = [
    "Fixed-Rate-Writes-50",
    "Fixed-Rate-Writes-250",
    "Fixed-Rate-Writes-500",
    "Query-Performance",
]

METRIC_INFO = {
    "transactions_per_second_(tps)": {
        "ylabel": "Transactions Per Second (TPS)",
        "desc": "This graph illustrates the throughput measured in Transactions Per Second (TPS). It highlights the rate at which transactions are processed, endorsed, and committed across the network under the current workload, showing how performance scales with the number of peers."
    },
    "endorsement_latency_over_time": {
        "ylabel": "Latency (s)",
        "desc": "This graph displays the endorsement latency over time. It represents the duration required for peers to simulate and endorse transaction proposals, serving as a key indicator of smart contract responsiveness."
    },
    "fabric_process_cpu_usage": {
        "ylabel": "CPU Usage (\\%)",
        "desc": "This graph shows the CPU utilization of the Fabric processes (Orderers and Peers). Monitoring CPU usage is critical to determining whether the system's throughput is bound by computational limits."
    },
    "fabric_process_memory_usage_(resident_set_size)": {
        "ylabel": "Memory (Bytes)",
        "desc": "This graph plots the Resident Set Size (RSS) memory consumption of the Fabric containers. It provides insights into the memory footprint and potential memory bloat during sustained operational phases."
    },
    "gossip_protocol_traffic": {
        "ylabel": "Messages / Second",
        "desc": "This graph tracks the Gossip protocol message rates. It demonstrates the peer-to-peer communication overhead required for block dissemination and state synchronization, which typically scales exponentially with network size."
    },
    "blockchain_ledger_height_(blocks)": {
        "ylabel": "Blocks",
        "desc": "This graph visualizes the ledger height progression. It serves to verify that blocks are being appended synchronously and at a consistent rate across all participating peers."
    },
    "cumulative_transactions_over_time": {
        "ylabel": "Total Transactions",
        "desc": "This graph displays the cumulative count of transactions processed over the duration of the test phase, providing a macroscopic view of the total workload handled."
    }
}



def latex_escape(s):
    return (
        s.replace("\\", "\\textbackslash{}")
        .replace("_", "\\_")
        .replace("&", "\\&")
        .replace("%", "\\%")
        .replace("#", "\\#")
        .replace("{", "\\{")
        .replace("}", "\\}")
    )


def clean_title(name):
    return name.replace("_", " ").replace("(", "").replace(")", "")


def format_phase_title(name):
    return name.replace("-", " ")


def get_headers(csv_path):
    with open(csv_path, "r") as f:
        reader = csv.reader(f)
        return next(reader)


def build_axis(headers, metric_dir, phase):
    import os
    node_order = ["2_nodes", "4_nodes", "8_nodes", "16_nodes"]
    colors = {
        "2_nodes": "blue",
        "4_nodes": "red",
        "8_nodes": "green!70!black",
        "16_nodes": "orange",
    }
    
    metric_basename = os.path.basename(metric_dir)
    info = METRIC_INFO.get(metric_basename, {
        "ylabel": "Value", 
        "desc": "This graph visualizes the collected metrics for the specified test phase."
    })

    plots = []
    legend_entries = []

    for node_dir in node_order:
        node_path = os.path.join(metric_dir, node_dir)
        if not os.path.isdir(node_path):
            continue

        csv_file = os.path.join(node_path, f"{phase}.csv")
        if not os.path.exists(csv_file):
            continue

        node_block = [f"        % {node_dir.replace('_nodes', ' Peers')}"]

        for i in range(1, len(headers)):
            node_block.append(
                f"        \\addplot[thick, color={colors[node_dir]}, solid, mark=none] "
                f"table [x index=0, y index={i}, col sep=comma] "
                f"{{csv/{os.path.relpath(csv_file, BASE_DIR)}}};"
            )
            legend_entries.append(latex_escape(f"{node_dir}-col{i}"))

        plots.append("\n".join(node_block))

    use_legend = len(legend_entries) <= 10

    figure = []
    figure.append("\begin{figure}[H]")
    figure.append("    \centering")
    figure.append("    \begin{tikzpicture}")
    figure.append("        \begin{axis}[")
    figure.append("            unbounded coords=jump,")
    figure.append("            width=1\textwidth,")
    figure.append("            height=8cm,")
    figure.append("            xlabel={Time (seconds)},")
    figure.append(f"            ylabel={{{info['ylabel']}}},")
    if use_legend:
        figure.append("            legend pos=outer north east,")
        figure.append("            legend style={font=\tiny},")
    figure.append("            grid=both,")
    figure.append("            minor tick num=1,")
    figure.append("            xtick distance=5")
    figure.append("        ]")
    figure.append("")

    for block in plots:
        figure.append(block)
        figure.append("")

    if use_legend:
        clean_legends = [e.split('-')[-1] if '-' in e else e for e in legend_entries]
        figure.append("        \legend{" + ",".join(clean_legends) + "}")
        figure.append("")

    figure.append("        \end{axis}")
    figure.append("    \end{tikzpicture}")
    figure.append(
        f"    \caption{{{latex_escape(clean_title(os.path.basename(metric_dir)))} for {latex_escape(format_phase_title(phase))}.}}"
    )
    figure.append(f"    \label{{fig:{(os.path.basename(metric_dir))}_{(phase)}}}")
    figure.append("\end{figure}")
    figure.append("")
    
    if not use_legend:
        figure.append("\textbf{Legend Note:} Due to the high number of data series, the traditional legend is omitted. ")
        figure.append("Configurations are distinguished by color: \textcolor{blue}{\textbf{blue}} lines represent the 2-peer network, ")
        figure.append("\textcolor{red}{\textbf{red}} lines represent 4 peers, \textcolor{green!70!black}{\textbf{dark green}} lines represent 8 peers, ")
        figure.append("and \textcolor{orange}{\textbf{orange}} lines represent 16 peers. Multiple lines of the exact same color denote individual system components ")
        figure.append("(e.g., separate peer containers or the orderer) operating concurrently within that specific network configuration.\\\\")
        figure.append("")
        
    figure.append(f"{info['desc']}")
    figure.append("")
    figure.append("% TODO: PASTE YOUR ANALYSIS FOR THIS GRAPH HERE")
    figure.append("")
    figure.append("\clearpage")

    return "\n".join(figure)


def main():
    global BASE_DIR
    if len(sys.argv) > 1:
        BASE_DIR = sys.argv[1]
    else:
        # Find latest timestamp folder
        folders = [d for d in os.listdir(".") if os.path.isdir(d) and "-to-" in d]
        if not folders:
            print("Error: No data folders found.")
            sys.exit(1)
        BASE_DIR = sorted(folders)[-1]
        print(f"Auto-selected latest data folder: {BASE_DIR}")

    with open(OUTPUT_FILE, "w") as out:
        metric_dirs = sorted(os.listdir(BASE_DIR))

        for phase in PHASE_ORDER:
            out.write(f"\\subsection{{{format_phase_title(phase)}}}\n\n")

            for metric in metric_dirs:
                metric_path = os.path.join(BASE_DIR, metric)
                if not os.path.isdir(metric_path):
                    continue

                headers = None
                for node_dir in os.listdir(metric_path):
                    test_csv = os.path.join(metric_path, node_dir, f"{phase}.csv")
                    if os.path.exists(test_csv):
                        headers = get_headers(test_csv)
                        break

                if not headers:
                    continue

                out.write(f"\\subsubsection{{{clean_title(metric)}}}\n\n")
                out.write(
                    f"\\textbf{{Fields:}} {latex_escape(', '.join(headers[1:]))}\n\n"
                )

                axis_code = build_axis(headers, metric_path, phase)
                out.write(axis_code + "\n\n")


if __name__ == "__main__":
    main()
