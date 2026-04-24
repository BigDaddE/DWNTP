#!/usr/bin/env python
import csv
import os

BASE_DIR = "20260420_100458-to-20260420_110724"
OUTPUT_FILE = "generated_graphs.tex"

PHASE_ORDER = [
    "Fixed-Rate-Writes-50",
    "Fixed-Rate-Writes-250",
    "Fixed-Rate-Writes-500",
    "Stress-Test-Backlog",
    "Query-Performance",
]


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
    node_order = ["2_nodes", "4_nodes", "8_nodes", "16_nodes"]
    colors = {
        "2_nodes": "blue",
        "4_nodes": "red",
        "8_nodes": "green!70!black",
        "16_nodes": "orange",
    }

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

    use_legend = len(legend_entries) <= 4

    figure = []
    figure.append("\\begin{figure}[H]")
    figure.append("    \\centering")
    figure.append("    \\begin{tikzpicture}")
    figure.append("        \\begin{axis}[")
    figure.append("            width=1\\textwidth,")
    figure.append("            height=8cm,")
    figure.append("            xlabel={Time (seconds)},")
    figure.append("            ylabel={Value},")
    if use_legend:
        figure.append("            legend pos=north west,")
        figure.append("            legend style={font=\\tiny},")
    figure.append("            grid=both,")
    figure.append("            minor tick num=1,")
    figure.append("            xtick distance=5")
    figure.append("        ]")
    figure.append("")

    for block in plots:
        figure.append(block)
        figure.append("")

    if use_legend:
        figure.append("        \\legend{" + ",".join(legend_entries) + "}")
        figure.append("")

    figure.append("        \\end{axis}")
    figure.append("    \\end{tikzpicture}")
    figure.append(
        f"    \\caption{{{latex_escape(clean_title(os.path.basename(metric_dir)))} for {latex_escape(format_phase_title(phase))}.}}"
    )
    figure.append(
        f"    \\label{{fig:{latex_escape(os.path.basename(metric_dir))}_{latex_escape(phase)}}}"
    )
    figure.append("\\end{figure}")

    return "\n".join(figure)


def main():
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
