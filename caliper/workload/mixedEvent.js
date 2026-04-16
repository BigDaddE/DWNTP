"use strict";

const { WorkloadModuleBase } = require("@hyperledger/caliper-core");

class MixedEventWorkload extends WorkloadModuleBase {
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
		// Default to 50% writes, 50% reads if not specified in the benchmark config
		this.writeRatio = this.roundArguments.writeRatio || 0.5;
	}

	async submitTransaction() {
		this.txIndex++;

		const isWrite = Math.random() < this.writeRatio;

		let request;
		if (isWrite) {
			// Construct arguments for the DWNTP Go chaincode Write
			const rtuId = `RTU-${Math.floor(Math.random() * 10)}`;
			const eventName = "SetVoltage";
			const eventDescription = `Mixed workload write ${this.txIndex} from worker ${this.workerIndex}`;
			const uniqueTimestamp = Date.now() * 100000 + this.workerIndex * 10000 + this.txIndex;

			request = {
				contractId: "dwntp",
				contractFunction: "LogEvent",
				invokerIdentity: "User1@org1.dwntp.com",
				contractArguments: [rtuId, eventName, eventDescription, uniqueTimestamp.toString()],
				readOnly: false,
			};
		} else {
			// Construct arguments for the DWNTP Go chaincode Read
			request = {
				contractId: "dwntp",
				contractFunction: "GetAllEvents",
				invokerIdentity: "User1@org1.dwntp.com",
				contractArguments: [],
				readOnly: true,
			};
		}

		await this.sutAdapter.sendRequests(request);
	}
}

function createWorkloadModule() {
	return new MixedEventWorkload();
}

module.exports.createWorkloadModule = createWorkloadModule;
