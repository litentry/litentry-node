pub fn addr_from_sig(msg: [u8; 32], sig: [u8; 65]) -> Result<[u8; 20], sp_io::EcdsaVerifyError> {
    let pubkey = sp_io::crypto::secp256k1_ecdsa_recover(&sig, &msg)?;
    let hashed_pk = sp_io::hashing::keccak_256(&pubkey);

    let mut addr = [0u8; 20];
    addr[..20].copy_from_slice(&hashed_pk[12..32]);
    Ok(addr)
}

#[cfg(test)]
mod tests {
	use super::*;
	use hex::decode;

	/// Returns a eth_sign-compatible hash of data to sign.
	/// The data is prepended with special message to prevent
	/// malicious DApps from using the function to sign forged transactions.
	pub fn eth_data_hash(mut data: Vec<u8>) -> [u8; 32] {
		let mut message_data = format!("\x19Ethereum Signed Message:\n{}", data.len()).into_bytes();
		message_data.append(&mut data);
		sp_io::hashing::keccak_256(&message_data)
	}

	#[test]
	fn correct_recover() {

		let msg = decode("61626364656667").unwrap();
		let msg = eth_data_hash(msg);

		let sig_bytes = decode("5900a81f236e27be7ee2c796e0de9b383aadcd8b3c53fd881dd378f4c2bc1a54406be632a464c197131c668432f32a966a19354920686a8f8fdd9c9ab0a0dd011b").unwrap();
		let mut sig = [0u8; 65];
		sig[0..65].copy_from_slice(&sig_bytes[0..65]);

		let addr_expected_bytes = decode("Fe7cef4F3A7eF57Ac2401122fB51590bfDf9350a").unwrap();
		let mut addr_expected = [0u8; 20];
		addr_expected[0..20].copy_from_slice(&addr_expected_bytes[0..20]);

		let addr = addr_from_sig(msg, sig).ok().unwrap();
		assert_eq!(addr, addr_expected);
	}

	#[test]
	fn wrong_msg() {
		
		let msg = decode("626364656667").unwrap();
		let msg = eth_data_hash(msg);

		let sig_bytes = decode("5900a81f236e27be7ee2c796e0de9b383aadcd8b3c53fd881dd378f4c2bc1a54406be632a464c197131c668432f32a966a19354920686a8f8fdd9c9ab0a0dd011b").unwrap();
		let mut sig = [0u8; 65];
		sig[0..65].copy_from_slice(&sig_bytes[0..65]);

		let addr_expected_bytes = decode("Fe7cef4F3A7eF57Ac2401122fB51590bfDf9350a").unwrap();
		let mut addr_expected = [0u8; 20];
		addr_expected[0..20].copy_from_slice(&addr_expected_bytes[0..20]);

		let addr = addr_from_sig(msg, sig).ok().unwrap();
		assert_ne!(addr, addr_expected);
	}

	#[test]
	fn sig_from_another_addr() {
		
		let msg = decode("61626364656667").unwrap();
		let msg = eth_data_hash(msg);

		let sig_bytes = decode("a4543cd17d07a9b5207bbf4ccf3c9d47e0a292a6ce461427ebc50de24387887b14584651c3bc11376ba9fe662df325ced20f5c30dd782b6bee15cb474c206a341b").unwrap();
		let mut sig = [0u8; 65];
		sig[0..65].copy_from_slice(&sig_bytes[0..65]);

		let addr_expected_bytes = decode("Fe7cef4F3A7eF57Ac2401122fB51590bfDf9350a").unwrap();
		let mut addr_expected = [0u8; 20];
		addr_expected[0..20].copy_from_slice(&addr_expected_bytes[0..20]);

		let addr = addr_from_sig(msg, sig).ok().unwrap();
		assert_ne!(addr, addr_expected);
	}
}
