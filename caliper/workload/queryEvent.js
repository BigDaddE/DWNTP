"use strict";

const { WorkloadModuleBase } = require("@hyperledger/caliper-core");

class QueryEventWorkload extends WorkloadModuleBase {
	constructor() {
		super();
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
	}

	async submitTransaction() {
		const request = {
			contractId: "dwntp",
			contractFunction: "GetAllEvents",
			invokerIdentity: "User1@org1.dwntp.com",
			contractArguments: [],
			readOnly: true,
		};

		await this.sutAdapter.sendRequests(request);
	}
}

function createWorkloadModule() {
	return new QueryEventWorkload();
}

module.exports.createWorkloadModule = createWorkloadModule;
