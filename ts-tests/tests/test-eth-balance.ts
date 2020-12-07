import { ApiPromise, WsProvider, Keyring } from "@polkadot/api";
import { KeyringPair } from '@polkadot/keyring/types';
import { U8aFixed } from '@polkadot/types/codec';
import * as crypto from '@polkadot/util-crypto';
import { testValidator } from "@polkadot/util-crypto/base32/is";
import { expect } from "chai";

// Provider is set to localhost for development
const wsProvider = new WsProvider("ws://localhost:9944");

// Keyring needed to sign using Alice account
const keyring = new Keyring({ type: 'sr25519' });

// Configs of test ropsten account
const test_eth_address = "[0x167453188a05082e3b347c1c518f3dd55d37fbbf]";
const test_r = '0x4306864b7928f8393579ad849fd47685840cdf197c0085a5d241d28b9c6d14b3';
const test_s = '0x3f4d2f5912213b3c0983c01a2199759b794d1423fe66cd5076b6ca9f5f60e45f';
const test_v = 27;


// Setup the API and Alice Account
async function init() {
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

// Create Ethereum Link from ALICE
async function eth_link(api: ApiPromise, alice: KeyringPair) {

  console.log(`\nStep 1: Link Ethereum account`);

  const transaction = api.tx.accountLinkerModule.link(alice.address, 0, 0, 0, test_r, test_s, test_v);

  const link = new Promise<{ block: string }>(async (resolve, reject) => {
		const unsub = await transaction.signAndSend(alice, (result) => {
			console.log(`Link creation is ${result.status}`);
			if (result.status.isInBlock) {
				console.log(`Link included at blockHash ${result.status.asInBlock}`);
        console.log(`Waiting for finalization... (can take a minute)`);
      } else if (result.status.isFinalized) {
				console.log(`Transfer finalized at blockHash ${result.status.asFinalized}`);
				unsub();
				resolve({
					block: result.status.asFinalized.toString(),
				});
			}
		});
	});
	return link;

}

// Retrieve Alice & Link Storage
async function check_linking_state(api: ApiPromise, alice: KeyringPair) {

	console.log(`\nStep 2: Retrieving linking state of Alice `);

	// Retrieve Alice account with new nonce value
	const { nonce, data: balance } = await api.query.system.account(alice.address);
	console.log(`Alice Substrate Account (nonce: ${nonce}) balance, free: ${balance.free}`);

	const linkedEthAddress = (await api.query.accountLinkerModule.ethereumLink(alice.address));
  console.log(`Linked Ethereum addresses of Alice are: ${linkedEthAddress.toString()}`);
  
  expect(linkedEthAddress.toString()).to.equal(test_eth_address);

	return;
}


// Claim Assets for Alice
async function asset_claim(api: ApiPromise, alice: KeyringPair) {

	console.log(`\nStep 3: Claim assets for Alice`);

	const transaction = await api.tx.offchainWorkerModule.assetClaim();

	const data = new Promise<{ block: string }>(async (resolve, reject) => {
		const unsub = await transaction.signAndSend(alice, (result) => {
			console.log(`Transfer is ${result.status}`);
			if (result.status.isInBlock) {
				console.log(`Transfer included at blockHash ${result.status.asInBlock}`);
        console.log(`Waiting for finalization... (can take a minute)`);
      } else if (result.status.isFinalized) {
				console.log(`Transfer finalized at blockHash ${result.status.asFinalized}`);
				unsub();
				resolve({
					block: result.status.asFinalized.toString(),
				});
			}
		});
	});
	return data;
}

// Retrieve assets balances of Alice
async function get_assets(api: ApiPromise, alice: KeyringPair) {

	console.log(`\nStep 4: Retrieving assets of Alice`);

	// Retrieve Alice account with new nonce value
	const { nonce, data: balance } = await api.query.system.account(alice.address);
	console.log(`Alice Substrate Account (nonce: ${nonce}) balance, free: ${balance.free}`);

	const assetsBalances = (await api.query.offchainWorkerModule.accountBalance(alice.address));
  console.log(`Linked Ethereum balances of Alice are: ${assetsBalances.toString()}`);
  
  // TODO fetch real time balance and compare it here
  expect(assetsBalances.toString()).to.equal(`[0,"0x000000000000000006f05b59d3b20000"]`);

	return;

}

async function main() {
	const { api, alice } = await init();

	// step 1: Creating the contract from ALICE
	const link = await eth_link(api, alice)

	// step 2: Retrieving Alice's linked Ethereum accounts
	await check_linking_state(api, alice);

	// step 3: Claim assets for Alice
	await asset_claim(api, alice);

	// step 4: Retrieving assets information of Alice
	await get_assets(api, alice);
}

main().catch(console.error).then(() => process.exit(0));