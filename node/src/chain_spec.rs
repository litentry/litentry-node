use sp_core::{Pair, Public, sr25519, crypto::UncheckedInto,};
use hex_literal::hex;
use litentry_runtime::{
	AccountId, AuraConfig, BalancesConfig, GenesisConfig, GrandpaConfig,
	SudoConfig, SystemConfig, WASM_BINARY, Signature
};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{Verify, IdentifyAccount};
use sc_service::{ChainType, Properties};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
	(
		get_from_seed::<AuraId>(s),
		get_from_seed::<GrandpaId>(s),
	)
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || testnet_genesis(
			wasm_binary,
			// Initial PoA authorities
			vec![
				authority_keys_from_seed("Alice"),
			],
			// Sudo account
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			// Pre-funded accounts
			vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
				get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			],
			true,
		),
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		Some(litentry_properties()),
		// Extensions
		None,
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || testnet_genesis(
			wasm_binary,
			// Initial PoA authorities
			vec![
				authority_keys_from_seed("Alice"),
				authority_keys_from_seed("Bob"),
			],
			// Sudo account
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			// Pre-funded accounts
			vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Charlie"),
				get_account_id_from_seed::<sr25519::Public>("Dave"),
				get_account_id_from_seed::<sr25519::Public>("Eve"),
				get_account_id_from_seed::<sr25519::Public>("Ferdie"),
				get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
				get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
				get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
				get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
				get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
			],
			true,
		),
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		None,
		// Extensions
		None,
	))
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	_enable_println: bool,
) -> GenesisConfig {
	GenesisConfig {
		frame_system: Some(SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_balances: Some(BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts.iter().cloned().map(|k|(k, 1 << 60)).collect(),
		}),
		pallet_aura: Some(AuraConfig {
			authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
		}),
		pallet_grandpa: Some(GrandpaConfig {
			authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
		}),
		pallet_sudo: Some(SudoConfig {
			// Assign network admin rights.
			key: root_key,
		}),
	}
}

/// Properties for Litentry.
pub fn litentry_properties() -> Properties {
	let mut properties = Properties::new();

	properties.insert("ss58Format".into(), 31.into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("tokenSymbol".into(), "LIT".into());

	properties
}

pub fn litentry_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Litentry",
		// ID
		"Litentry",
		ChainType::Live,
		move || testnet_genesis(
			wasm_binary,
			// Initial PoA authorities
			vec![
				// 49xBdg1G5MBrtPwPDK7Wv9WLjbRHZ71iRtAs3pRUPMDLVocf
				(
					hex!["9ce8c6f2c22502322fb29a1af5de753ce2c62e9eeb0a93efe1bd0ad56438e93a"].unchecked_into(),
					hex!["9ce8c6f2c22502322fb29a1af5de753ce2c62e9eeb0a93efe1bd0ad56438e93a"].unchecked_into(),
				),
				// 4Ackc6jWc31xiMdjPaDr7PzZCynKgi7im1Cc4Y2pRJcNvjo8
				(
					hex!["ba53195d0b128e94778f49c20933773ef9068ec322260d6991552cb078012a15"].unchecked_into(),
					hex!["ba53195d0b128e94778f49c20933773ef9068ec322260d6991552cb078012a15"].unchecked_into(),
				),
				// 46ruwarJX4oNZNXQFUuDPvp61PwoH7t2bTVCHorsqE34fDS2
				(
					hex!["142f074ccc4a78a0cdc584713f689bf3ed8d4bb279f6161d0bc066c1b4aff418"].unchecked_into(),
					hex!["142f074ccc4a78a0cdc584713f689bf3ed8d4bb279f6161d0bc066c1b4aff418"].unchecked_into(),
				),
				// 4BnqKkY6pE4c96p9rpyr4UFgEtqbnAHmaYbMvCwGNPBKAigR
				(
					hex!["ee3fab5285345a4551bd90df2c063a48cf9adb5a5f1310776b5e2a5747bbd12f"].unchecked_into(),
					hex!["ee3fab5285345a4551bd90df2c063a48cf9adb5a5f1310776b5e2a5747bbd12f"].unchecked_into(),
				),
			],
			// Sudo account
			hex!["9ce8c6f2c22502322fb29a1af5de753ce2c62e9eeb0a93efe1bd0ad56438e93a"].into(),
			// Pre-funded accounts
			vec![
				hex!["9ce8c6f2c22502322fb29a1af5de753ce2c62e9eeb0a93efe1bd0ad56438e93a"].into(),
				hex!["ba53195d0b128e94778f49c20933773ef9068ec322260d6991552cb078012a15"].into(),
				hex!["142f074ccc4a78a0cdc584713f689bf3ed8d4bb279f6161d0bc066c1b4aff418"].into(),
				hex!["ee3fab5285345a4551bd90df2c063a48cf9adb5a5f1310776b5e2a5747bbd12f"].into(),
			],
			true,
		),
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		Some("Litentry"),
		// Properties
		Some(litentry_properties()),
		// Extensions
		None,
	))
}
