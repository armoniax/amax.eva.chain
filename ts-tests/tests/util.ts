import Web3 from "web3";
import { ethers } from "ethers";
import { JsonRpcResponse } from "web3-core-helpers";
import { spawn, ChildProcess } from "child_process";

import { NODE_BINARY_NAME, CHAIN_ID } from "./config";

export const PORT = 19931;
export const RPC_PORT = 19932;
export const WS_PORT = 19933;

export const DISPLAY_LOG = process.env.FRONTIER_LOG || false;
export const FRONTIER_LOG = process.env.FRONTIER_LOG || "info";
export const FRONTIER_BUILD = process.env.FRONTIER_BUILD || "release";

export const BINARY_PATH = `../target/${FRONTIER_BUILD}/${NODE_BINARY_NAME}`;
export const SPAWNING_TIME = 60000;

export async function customRequest(web3: Web3, method: string, params: any[]) {
	return new Promise<JsonRpcResponse>((resolve, reject) => {
		(web3.currentProvider as any).send(
			{
				jsonrpc: "2.0",
				id: 1,
				method,
				params,
			},
			(error: Error | null, result?: JsonRpcResponse) => {
				if (error) {
					reject(
						`Failed to send custom request (${method} (${params.join(",")})): ${
							error.message || error.toString()
						}`
					);
				}
				resolve(result);
			}
		);
	});
}

// Create a block and finalize it.
// It will include all previously executed transactions since the last finalized block.
export async function createAndFinalizeBlock(web3: Web3) {
	const response = await customRequest(web3, "engine_createBlock", [true, true, null]);
	if (!response.result) {
		throw new Error(`Unexpected result: ${JSON.stringify(response)}`);
	}
	await new Promise((resolve) => setTimeout(() => resolve(), 500));
}

// Create a block and finalize it.
// It will include all previously executed transactions since the last finalized block.
export async function createAndFinalizeBlockNowait(web3: Web3) {
	const response = await customRequest(web3, "engine_createBlock", [true, true, null]);
	if (!response.result) {
		throw new Error(`Unexpected result: ${JSON.stringify(response)}`);
	}
}

export async function startFrontierNode(provider?: string): Promise<{
	web3: Web3;
	binary: ChildProcess;
	ethersjs: ethers.providers.JsonRpcProvider;
}> {
	var web3;
	if (!provider || provider == "http") {
		web3 = new Web3(`http://127.0.0.1:${RPC_PORT}`);
	}

	const cmd = BINARY_PATH;
	const args = [
		`--chain=dev`,
		`--validator`, // Required by manual sealing to author the blocks
		`--execution=Native`, // Faster execution using native
		`--no-telemetry`,
		`--no-prometheus`,
		`--sealing=Manual`,
        `--unsafe-ws-external`,
        `--unsafe-rpc-external`,
		`--no-grandpa`,
		`--force-authoring`,
		`-l${FRONTIER_LOG}`,
		`--port=${PORT}`,
		`--rpc-port=${RPC_PORT}`,
		`--ws-port=${WS_PORT}`,
		`--ethapi`, `trace`,
		`--ethapi`, `txpool`,
		`--ethapi`, `debug`,
		`--tmp`,
	];
	const binary = spawn(cmd, args);

	binary.on("error", (err) => {
		if ((err as any).errno == "ENOENT") {
			console.error(
				`\x1b[31mMissing Frontier binary (${BINARY_PATH}).\nPlease compile the Frontier project:\ncargo build\x1b[0m`
			);
		} else {
			console.error(err);
		}
		process.exit(1);
	});

	const binaryLogs = [];
	await new Promise((resolve) => {
		const timer = setTimeout(() => {
			console.error(`\x1b[31m Failed to start Frontier Template Node.\x1b[0m`);
			console.error(`Command: ${cmd} ${args.join(" ")}`);
			console.error(`Logs:`);
			console.error(binaryLogs.map((chunk) => chunk.toString()).join("\n"));
			process.exit(1);
		}, SPAWNING_TIME - 2000);

		const onData = async (chunk) => {
			if (DISPLAY_LOG) {
				console.log(chunk.toString());
			}
			binaryLogs.push(chunk);
			if (chunk.toString().match(/Manual Seal Ready/)) {
				if (!provider || provider == "http") {
					// This is needed as the EVM runtime needs to warmup with a first call
					await web3.eth.getChainId();
				}

				clearTimeout(timer);
				if (!DISPLAY_LOG) {
					binary.stderr.off("data", onData);
					binary.stdout.off("data", onData);
				}
				// console.log(`\x1b[31m Starting RPC\x1b[0m`);
				resolve();
			}
		};
		binary.stderr.on("data", onData);
		binary.stdout.on("data", onData);
	});

	if (provider == "ws") {
		web3 = new Web3(`ws://127.0.0.1:${WS_PORT}`);
	}

	let ethersjs = new ethers.providers.StaticJsonRpcProvider(`http://127.0.0.1:${RPC_PORT}`, {
		chainId: CHAIN_ID,
		name: "frontier-dev",
	});

	return { web3, binary, ethersjs };
}

export function describeWithFrontier(title: string, cb: (context: { web3: Web3 }) => void, provider?: string) {
	describe(title, () => {
		let context: {
			web3: Web3;
			ethersjs: ethers.providers.JsonRpcProvider;
		} = { web3: null, ethersjs: null };

		let binary: ChildProcess;
		// Making sure the Frontier node has started
		before("Starting Frontier Test Node", async function () {
			this.timeout(SPAWNING_TIME);
			let init = await startFrontierNode(provider);
			init.web3 = extendTrace(init.web3)
			context.web3 = init.web3;
			context.ethersjs = init.ethersjs;
			binary = init.binary;
		});

		after(async function () {
			//console.log(`\x1b[31m Killing RPC\x1b[0m`);
			binary.kill();
		});

		cb(context);
	});
}

// extends web3 for usage with parity's `trace` module
export function extendTrace(web3: Web3) {
	let web3_with_trace = web3.extend({
		property: 'trace',
		methods: [{
			name: 'call',
			call: 'trace_call',
			params: 2
		}, {
			name: 'callMany',
			call: 'trace_callMany',
			params: 2
		}, {
			name: 'rawTransaction',
			call: 'trace_rawTransaction',
			params: 2
		}, {
			name: 'replayBlockTransactions',
			call: 'trace_replayBlockTransactions',
			params: 2,
			// inputFormatter: [web3.extend.formatters.inputBlockNumberFormatter,null]
		}, {
			name: 'replayTransaction',
			call: 'trace_replayTransaction',
			params: 2
		}, {
			name: 'block',
			call: 'trace_block',
			params: 1,
			// inputFormatter: [web3.extend.formatters.inputBlockNumberFormatter]
		}, {
			name: 'filter',
			call: 'trace_filter',
			params: 1
		}, {
			name: 'get',
			call: 'trace_get',
			params: 2
		}, {
			name: 'transaction',
			call: 'trace_transaction',
			params: 1
		}]
	});

	let web3_with_debug = web3_with_trace.extend({
		property: 'debug',
		methods: [{
			name: 'traceBlockByNumber',
			call: 'debug_traceBlockByNumber',
			params: 2
		}, {
			name: 'traceBlockByHash',
			call: 'debug_traceBlockByHash',
			params: 2
		}, {
			name: 'traceTransaction',
			call: 'debug_traceTransaction',
			params: 2
		}]
	});
	
	let web3_with_txpool = web3_with_debug.extend({
		property: 'txpool',
		methods: [{
			name: 'content',
			call: 'txpool_content',
			params: 0
		}, {
			name: 'inspect',
			call: 'txpool_inspect',
			params: 0
		}, {
			name: 'status',
			call: 'txpool_status',
			params: 0
		}]
	});

	return web3_with_txpool;
  } // note: some methods must be manually hexified, due to the fact that it takes arrays with hexified values inside

export function describeWithFrontierWs(title: string, cb: (context: { web3: Web3 }) => void) {
	describeWithFrontier(title, cb, "ws");
}

