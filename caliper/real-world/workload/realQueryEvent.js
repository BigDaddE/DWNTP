"use strict";

const { WorkloadModuleBase } = require("@hyperledger/caliper-core");

const IDENTITIES = ["User1@org1.dwntp.com", "User2@org1.dwntp.com"];

class RealQueryEventWorkload extends WorkloadModuleBase {
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
	}

	async submitTransaction() {
		this.txIndex++;

		const invokerIdentity = IDENTITIES[this.txIndex % IDENTITIES.length];

		const request = {
			contractId: this.sutContext?.contractId || "dwntp",
			contractFunction: "GetAllEvents",
			invokerIdentity,
			contractArguments: [],
			readOnly: true,
		};

		await this.sutAdapter.sendRequests(request);
	}
}

function createWorkloadModule() {
	return new RealQueryEventWorkload();
}

module.exports.createWorkloadModule = createWorkloadModule;
