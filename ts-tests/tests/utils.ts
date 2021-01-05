import Web3 from "web3";
import { JsonRpcResponse } from "web3-core-helpers";
import { spawn, ChildProcess } from "child_process";
import 'mocha';
import { ApiPromise, Keyring, WsProvider } from "@polkadot/api";
import { KeyringPair } from '@polkadot/keyring/types';

export const BINARY_PATH = `../target/debug/litentry-node`;
export const SPAWNING_TIME = 30000;

// Provider is set to localhost for development
const wsProvider = new WsProvider("ws://localhost:9944");

// Keyring needed to sign using Alice account
const keyring = new Keyring({ type: 'sr25519' });

export async function launchLitentryNode(specFilename: string, provider?: string): Promise<{ binary: ChildProcess }> {

	const cmd = BINARY_PATH;
	const args = [
		`--dev`,
		`--tmp`,
	];
	const binary = spawn(cmd, args);

	binary.on("error", (err) => {
		if ((err as any).errno == "ENOENT") {
			console.error(
				`\x1b[31mMissing litentry-node binary (${BINARY_PATH}).\nPlease compile the litentry project:\ncargo build\x1b[0m`
			);
		} else {
			console.error(err);
		}
		process.exit(1);
	});

//	await new Promise((resolve) => {
//		const timer = setTimeout(() => {
//			console.error(`\x1b[31m Failed to start Litentry Node.\x1b[0m`);
//			console.error(`Command: ${cmd} ${args.join(" ")}`);
//			process.exit(1);
//		}, SPAWNING_TIME - 2000);
//
//		const onData = async (chunk) => {
//			if (chunk.toString().match("Listening for new connections on 127.0.0.1:9944.")) {
//
//				clearTimeout(timer);
//				console.log(`Litentry Node Starts`);
//				resolve();
//			}
//		};
//		binary.stderr.on("data", onData);
//		binary.stdout.on("data", onData);
//	});

	return { binary };
}

export async function initApiPromise(wsProvider: WsProvider) {
	console.log(`Initiating the API (ignore message "Unable to resolve type B..." and "Unknown types found...")`);

	// Initiate the polkadot API.
	const api = await ApiPromise.create({
		provider: wsProvider,
		types: {
			// mapping the actual specified address format
			Address: "AccountId",
			// mapping the lookup
			LookupSource: "AccountId",
			Account: {
				nonce: "U256",
				balance: "U256"
			},
			Transaction: {
				nonce: "U256",
				action: "String",
				gas_price: "u64",
				gas_limit: "u64",
				value: "U256",
				input: "Vec<u8>",
				signature: "Signature"
			},
			Signature: {
				v: "u64",
				r: "H256",
				s: "H256"
			}
		}
  });

	console.log(`Initialization done`);
	console.log(`Genesis at block: ${api.genesisHash.toHex()}`);

	const alice = keyring.addFromUri('//Alice', { name: 'Alice default' });

	const { nonce, data: balance } = await api.query.system.account(alice.address);
	console.log(`Alice Substrate Account: ${alice.address}`);
	console.log(`Alice Substrate Account (nonce: ${nonce}) balance, free: ${balance.free.toHex()}`);

	return { api, alice };
}

export function describeLitentry(title: string, specFilename: string, cb: (context: {api: ApiPromise, alice: KeyringPair}) => void, provider?: string) {
	describe(title, function() {
    // Set timeout to 90 seconds
    this.timeout(90000);

    let binary: ChildProcess;
    let context: {api: ApiPromise, alice: KeyringPair} = { api:  {} as ApiPromise, alice: {} as KeyringPair};
		// Making sure the Litentry node has started
		before("Starting Litentry Test Node", async function () {
			//this.timeout(SPAWNING_TIME);
			const initNode = await launchLitentryNode(specFilename, provider);
      binary = initNode.binary;
      const initApi = await initApiPromise(wsProvider);
      context.api = initApi.api;
      context.alice = initApi.alice;
		});

		after(async function () {
			//console.log(`\x1b[31m Killing RPC\x1b[0m`);
      binary.kill();
      //context = { api:  {} as ApiPromise, alice: {} as KeyringPair};
      context.api.disconnect();
    });
    
    cb(context);
	});
}