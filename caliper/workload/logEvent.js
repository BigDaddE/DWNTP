"use strict";

const { WorkloadModuleBase } = require("@hyperledger/caliper-core");
const { logPhaseStart } = require("./utils/phaseLogger");

class LogEventWorkload extends WorkloadModuleBase {
	constructor() {
		super();
		this.txIndex = 0;
	}

	async initializeWorkloadModule(
		workerIndex,
		totalWorkers,
		roundIndex,
		roundArguments,
		sutAdapter,
		sutContext,
	) {
		await super.initializeWorkloadModule(
			workerIndex,
			totalWorkers,
			roundIndex,
			roundArguments,
			sutAdapter,
			sutContext,
		);
		logPhaseStart(roundArguments.label, workerIndex);
	}

	async submitTransaction() {
		this.txIndex++;

		// Construct the arguments as expected by the DWNTP Go chaincode:
		// LogEvent(ctx, rtuId string, eventName string, eventDescription string, eventTimestamp uint64)
		const rtuId = `RTU-${Math.floor(Math.random() * 10)}`;
		const eventName = "SetVoltage";
		const eventDescription = `Benchmark test event ${this.txIndex} from worker ${this.workerIndex}`;
		const uniqueTimestamp =
			Date.now() * 100000 + this.workerIndex * 10000 + this.txIndex;
		const eventTimestamp = uniqueTimestamp.toString(); // Go expects uint64, string format is standard for Fabric CLI/args

		const request = {
			contractId: "dwntp",
			contractFunction: "LogEvent",
			invokerIdentity: "User1@org1.dwntp.com",
			contractArguments: [rtuId, eventName, eventDescription, eventTimestamp],
			readOnly: false,
		};

		await this.sutAdapter.sendRequests(request);
	}
}

function createWorkloadModule() {
	return new LogEventWorkload();
}

module.exports.createWorkloadModule = createWorkloadModule;
