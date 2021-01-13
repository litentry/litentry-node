use sp_std::{prelude::*};
use frame_support::debug;
use sp_runtime::offchain::http;

// Fetch json result from remote URL with get method
pub fn fetch_json_http_get<'a>(remote_url: &'a [u8]) -> Result<Vec<u8>, &'static str> {
	let remote_url_str = core::str::from_utf8(remote_url)
		.map_err(|_| "Error in converting remote_url to string")?;

	let pending = http::Request::get(remote_url_str).send()
		.map_err(|_| "Error in sending http GET request")?;

	let response = pending.wait()
		.map_err(|_| "Error in waiting http response back")?;

	if response.code != 200 {
		debug::warn!("Unexpected status code: {}", response.code);
		return Err("Non-200 status code returned from http request");
	}

	let json_result: Vec<u8> = response.body().collect::<Vec<u8>>();

	let balance =
		core::str::from_utf8(&json_result).map_err(|_| "JSON result cannot convert to string")?;

	Ok(balance.as_bytes().to_vec())
}

// Fetch json result from remote URL with post method
pub fn fetch_json_http_post<'a>(remote_url: &'a [u8], body: &'a [u8]) -> Result<Vec<u8>, &'static str> {
	let remote_url_str = core::str::from_utf8(remote_url)
		.map_err(|_| "Error in converting remote_url to string")?;

	debug::info!("Offchain Worker post request url is {}.", remote_url_str);

	let pending = http::Request::post(remote_url_str, vec![body]).send()
		.map_err(|_| "Error in sending http POST request")?;

	let response = pending.wait()
		.map_err(|_| "Error in waiting http response back")?;

	if response.code != 200 {
		debug::warn!("Unexpected status code: {}", response.code);
		return Err("Non-200 status code returned from http request");
	}

	let json_result: Vec<u8> = response.body().collect::<Vec<u8>>();

	let balance =
		core::str::from_utf8(&json_result).map_err(|_| "JSON result cannot convert to string")?;

	Ok(balance.as_bytes().to_vec())
}

// u128 number string to u128
pub fn chars_to_u128(vec: &Vec<char>) -> Result<u128, &'static str> {
	// Check if the number string is decimal or hexadecimal (whether starting with 0x or not)
	let base = if vec.len() >= 2 && vec[0] == '0' && vec[1] == 'x' {
		// This is a hexadecimal number
		16
	} else {
		// This is a decimal number
		10
	};

	let mut result: u128 = 0;
	for (i, item) in vec.iter().enumerate() {
		// Skip the 0 and x digit for hex.
		// Using skip here instead of a new vec build to avoid an unnecessary copy operation
		if base == 16 && i < 2 {
			continue;
		}

		let n = item.to_digit(base);
		match n {
			Some(i) => {
				let i_64 = i as u128;
				result = result * base as u128 + i_64;
				if result < i_64 {
					return Err("Wrong u128 balance data format");
				}
			},
			None => return Err("Wrong u128 balance data format"),
		}
	}
	return Ok(result)
}

// number byte to string byte
pub fn u8_to_str_byte(a: u8) -> u8{
	if a < 10 {
		return a + 48 as u8;
	}
	else {
		return a + 87 as u8;
	}
}

// address to string bytes
pub fn address_to_string(address: &[u8]) -> Vec<u8> {

	let mut vec_result: Vec<u8> = Vec::new();
	for item in address {
		let a: u8 = item & 0x0F;
		let b: u8 = item >> 4;
		vec_result.push(u8_to_str_byte(b));
		vec_result.push(u8_to_str_byte(a));
	}
	return vec_result;
}

// Convert Vec of u8 array to Vec of u8 Vec
pub fn convert_u8_array_vec_to_u8_vec_vec(u8_array_vec: Vec<[u8; 20]>) -> Vec<Vec<u8>> {
	let mut vec_result: Vec<Vec<u8>> = Vec::new();

	for each_array in u8_array_vec {
		vec_result.push(each_array.to_vec());
	}

	vec_result
}

#[cfg(test)]
mod tests {
	#[test]
	fn test_chars_to_u128() {
		let correct_balance = vec!['5', '0', '0', '0', '0', '0', '0', '0', '0', '0', '0', '0', '0', '0', '0', '0', '0', '0'];
		assert_eq!(Ok(500000000000000000_u128), <Module<TestRuntime>>::chars_to_u128(&correct_balance));

		let correct_balance = vec!['a', '2'];
		assert_eq!(Err("Wrong u128 balance data format"), <Module<TestRuntime>>::chars_to_u128(&correct_balance));

		let correct_balance = vec!['0', 'x', 'f', 'e'];
		assert_eq!(Ok(254_u128), <Module<TestRuntime>>::chars_to_u128(&correct_balance));

		// Corner case check
		let correct_balance = vec!['0', 'x'];
		assert_eq!(Ok(0_u128), <Module<TestRuntime>>::chars_to_u128(&correct_balance));
	}
}
