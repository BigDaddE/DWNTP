"use strict";

const { WorkloadModuleBase } = require("@hyperledger/caliper-core");

const RTU_POOL = [
	"RTU-A1", "RTU-A2", "RTU-A3", "RTU-A4", "RTU-A5",
	"RTU-B1", "RTU-B2", "RTU-B3", "RTU-B4", "RTU-B5",
	"RTU-C1", "RTU-C2", "RTU-C3", "RTU-C4", "RTU-C5",
	"RTU-D1", "RTU-D2", "RTU-D3", "RTU-D4", "RTU-D5",
	"RTU-E1", "RTU-E2", "RTU-E3", "RTU-E4", "RTU-E5",
];

const EVENT_TYPES = [
	"SetVoltage",
	"OpenBreaker",
	"CloseBreaker",
	"EnableRelay",
	"DisableRelay",
	"ReadMeter",
	"Reset",
];

const IDENTITIES = ["User1@org1.dwntp.com", "User2@org1.dwntp.com"];

class RealLogEventWorkload extends WorkloadModuleBase {
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

		const rtuId = RTU_POOL[Math.floor(Math.random() * RTU_POOL.length)];
		const eventName = EVENT_TYPES[Math.floor(Math.random() * EVENT_TYPES.length)];
		const invokerIdentity = IDENTITIES[this.txIndex % IDENTITIES.length];
		const eventDescription = `realworld w${this.workerIndex} r${this.roundIndex} i${this.txIndex}`;

		const uniqueTimestamp =
			Date.now() * 100000 + this.workerIndex * 10000 + this.txIndex;

		const request = {
			contractId: this.sutContext?.contractId || "dwntp",
			contractFunction: "LogEvent",
			invokerIdentity,
			contractArguments: [
				rtuId,
				eventName,
				eventDescription,
				uniqueTimestamp.toString(),
			],
			readOnly: false,
		};

		await this.sutAdapter.sendRequests(request);
	}
}

function createWorkloadModule() {
	return new RealLogEventWorkload();
}

module.exports.createWorkloadModule = createWorkloadModule;
