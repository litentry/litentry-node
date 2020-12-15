//! Based on https://github.com/debris/base58/blob/master/src/lib.rs
//! works only up to 128 bytes
use sp_std::prelude::*;

const ALPHABET: &'static [u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

/// A trait for converting a value to base58 encoded string.
pub trait ToBase58 {
	/// Converts a value of `self` to a base58 value, returning the owned string.
	fn to_base58(&self) -> Vec<u8>;
}

impl ToBase58 for [u8] {
	fn to_base58(&self) -> Vec<u8> {
		let zcount = self.iter().take_while(|x| **x == 0).count();
		let size = (self.len() - zcount) * 138 / 100 + 1;
		let mut buffer = vec![0u8; size];

		let mut i = zcount;
		let mut high = size - 1;

		while i < self.len() {
			let mut carry = self[i] as u32;
			let mut j = size - 1;

			while j > high || carry != 0 {
				carry += 256 * buffer[j] as u32;
				buffer[j] = (carry % 58) as u8;
				carry /= 58;

				// in original trezor implementation it was underflowing
				if j  > 0 {
					j -= 1;
				}
			}

			i += 1;
			high = j;
		}

		let mut j = buffer.iter().take_while(|x| **x == 0).count();

		let mut result = Vec::new();
		for _ in 0..zcount {
			result.push(b'1');
		}

		while j < size {
			result.push(ALPHABET[buffer[j] as usize]);
			j += 1;
		}

		result
	}
}

#[cfg(test)]
mod tests {
    use super::ToBase58;
    use std::str::from_utf8;

	#[test]
	fn test_to_base58_basic() {
		assert_eq!(from_utf8(&b"".to_base58()).unwrap(), "");
		assert_eq!(from_utf8(&[32].to_base58()).unwrap(), "Z");
		assert_eq!(from_utf8(&[45].to_base58()).unwrap(), "n");
		assert_eq!(from_utf8(&[48].to_base58()).unwrap(), "q");
		assert_eq!(from_utf8(&[49].to_base58()).unwrap(), "r");
		assert_eq!(from_utf8(&[57].to_base58()).unwrap(), "z");
		assert_eq!(from_utf8(&[45, 49].to_base58()).unwrap(), "4SU");
		assert_eq!(from_utf8(&[49, 49].to_base58()).unwrap(), "4k8");
		assert_eq!(from_utf8(&b"abc".to_base58()).unwrap(), "ZiCa");
		assert_eq!(from_utf8(&b"1234598760".to_base58()).unwrap(), "3mJr7AoUXx2Wqd");
		assert_eq!(from_utf8(&b"abcdefghijklmnopqrstuvwxyz".to_base58()).unwrap(), "3yxU3u1igY8WkgtjK92fbJQCd4BZiiT1v25f");
	}

	#[test]
	fn test_to_base58_initial_zeros() {
		assert_eq!(from_utf8(&b"\0abc".to_base58()).unwrap(), "1ZiCa");
		assert_eq!(from_utf8(&b"\0\0abc".to_base58()).unwrap(), "11ZiCa");
		assert_eq!(from_utf8(&b"\0\0\0abc".to_base58()).unwrap(), "111ZiCa");
		assert_eq!(from_utf8(&b"\0\0\0\0abc".to_base58()).unwrap(), "1111ZiCa");
	}
}