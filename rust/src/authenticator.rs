use protocols::uds;
use self::uds::UdsInterface;
use error::Result;

pub struct MazdaAuthenticator {

}

impl MazdaAuthenticator {
	/// Authenticate using Mazda's protocol. `session_type` should usually be 0x87
	pub fn authenticate(&self, key: &str, interface: &UdsInterface, session_type: u8) -> Result<()> {
		// Request the session
		interface.request_session(session_type)?;
		// Request seed

		let seed = interface.request_security_seed()?;
		let key = Self::generate_key(key, 0xC541A9, &seed);
		// Authenticate with the key
		interface.request_security_key(&key)?;
		Ok(())
	}

	pub fn generate_key(key: &str, parameter: u32, seed: &[u8]) -> [u8; 3] {
		let mut parameter = parameter;
		// This is Mazda's key generation algorithm reverse engineered from a
		// Mazda 6 MPS ROM. Internally, the ECU uses a timer/counter for the seed
		// generation

		let nseed = {
			let mut nseed = seed.to_vec();
			nseed.extend_from_slice(key.as_bytes());
			nseed
		};

		for c in nseed.iter().cloned() {
			let mut c = c;
			for r in (1..=8).rev() {
				let s = (c & 1) ^ (parameter & 1) as u8;
				let mut m: u32 = 0;
				if s != 0 {
					parameter |= 0x0100_0000;
					m = 0x0010_9028;
				}

				c >>= 1;
				parameter >>= 1;
				let p3 = parameter & 0xFFEF_6FD7;
				parameter ^= m;
				parameter &= 0x0010_9028;

				parameter |= p3;
				parameter &= 0x00FF_FFFF;
			}
		}

		let mut res = [0; 3];
		res[0] = ((parameter >> 4) & 0xFF) as u8;
		res[1] = (((parameter >> 20) & 0xFF) + ((parameter >> 8) & 0xF0)) as u8;
		res[2] = (((parameter << 4) & 0xFF) + ((parameter >> 16) & 0x0F)) as u8;

		res
	}
}