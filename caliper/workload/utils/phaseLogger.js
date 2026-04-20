"use strict";

const fs = require("fs");
const path = require("path");

const TIMESTAMPS_FILE = path.resolve(
	__dirname,
	"../../../docs/grafana/run_timestamps.csv",
);

/**
 * Appends a phase start timestamp to run_timestamps.csv.
 * Only worker 0 writes to avoid duplicate rows from concurrent workers.
 *
 * @param {string} label - The round label from roundArguments.label
 * @param {number} workerIndex - The index of the calling worker
 */
function logPhaseStart(label, workerIndex) {
	if (workerIndex !== 0) return;

	const nodes = process.env.CALIPER_NODES || "unknown";
	const timestamp = new Date().toISOString().replace("T", " ").slice(0, 19);
	const line = `${nodes},${label},${timestamp}\n`;

	try {
		fs.appendFileSync(TIMESTAMPS_FILE, line);
	} catch (err) {
		console.warn(
			`[phaseLogger] Warning: could not write to ${TIMESTAMPS_FILE}: ${err.message}`,
		);
	}
}

module.exports = { logPhaseStart };
