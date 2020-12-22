use sp_std::prelude::*;

const SEP: u8 = b'1';
const ALPHABET: &'static [u8] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

pub trait Bech32 {
    fn encode(&self, hrp: Vec<u8>) -> Result<Vec<u8>, &'static str>;
}

impl Bech32 for [u8] {
    fn encode(&self, hrp: Vec<u8>) -> Result<Vec<u8>, &'static str> {
        if hrp.len() < 1 {
            return Err("invalidData")
        }

        let mut combined: Vec<u8> = self.clone().to_vec();
        combined.extend_from_slice(&create_checksum(&hrp, &self.to_vec()));
        let mut encoded = hrp;
        encoded.push(SEP);
        for p in combined {
            if p >= 32 {
                return Err("invalidData")
            }
            encoded.push(ALPHABET[p as usize]);
        }
        Ok(encoded)
    }
}

const GEN: [u32; 5] = [0x3b6a57b2, 0x26508e6d, 0x1ea119fa, 0x3d4233dd, 0x2a1462b3];

fn hrp_expand(hrp: &Vec<u8>) -> Vec<u8> {
    let mut v: Vec<u8> = Vec::new();
    for b in hrp {
        v.push(*b >> 5);
    }
    v.push(0);
    for b in hrp {
        v.push(*b & 0x1f);
    }
    v
}

fn create_checksum(hrp: &Vec<u8>, data: &Vec<u8>) -> Vec<u8> {
    let mut values: Vec<u8> = hrp_expand(hrp);
    values.extend_from_slice(data);
    // Pad with 6 zeros
    values.extend_from_slice(&[0u8; 6]);
    let plm: u32 = polymod(values) ^ 1;
    let mut checksum: Vec<u8> = Vec::new();
    for p in 0..6 {
        checksum.push(((plm >> 5 * (5 - p)) & 0x1f) as u8);
    }
    checksum
}

fn polymod(values: Vec<u8>) -> u32 {
    let mut chk: u32 = 1;
    let mut b: u8;
    for v in values {
        b = (chk >> 25) as u8;
        chk = (chk & 0x1ffffff) << 5 ^ (v as u32);
        for i in 0..5 {
            if (b >> i) & 1 == 1 {
                chk ^= GEN[i]
            }
        }
    }
    chk
}

#[cfg(test)]
mod tests {
	use super::Bech32;
	use std::str::from_utf8;

	#[test]
	fn test_to_base58_basic() {
		assert_eq!(from_utf8(&vec![0x00, 0x01, 0x02].encode(b"bech32".to_vec()).unwrap()).unwrap(), "bech321qpz4nc4pe");
    }
    
}
