pub fn addr_from_sig(msg: [u8; 32], sig: [u8; 65]) -> Result<[u8; 20], sp_io::EcdsaVerifyError> {
	let pubkey = sp_io::crypto::secp256k1_ecdsa_recover(&sig, &msg)?;
	let address = sp_io::hashing::keccak_256(&pubkey);

	let mut addr = [0u8; 20];
	addr[0..20].copy_from_slice(&address[12..32]);	
	Ok(addr)
}
