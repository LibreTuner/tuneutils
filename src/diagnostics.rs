use crate::{
	protocols::uds::UdsInterface,
	error::Result,
};

use std::rc::Rc;
use std::fmt;

pub struct UdsScanner {
	interface: Rc<UdsInterface>,
}

pub struct Code {
	raw: [u8; 2],
}



impl fmt::Display for Code {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let first_char = match (self.raw[0] & 0xC0) >> 6 {
			0x00 => 'P', // Powertrain
			0x01 => 'C', // Chassis
			0x02 => 'B', // Body
			0x03 => 'U', // Network
			_    => '?', // Will never reach this
		};

		let second_char = ((self.raw[0] & 0x30) >> 4);


        write!(f, "{}{:X}{:X}{:X}{:X}", first_char, second_char, self.raw[0] & 0x0F, (self.raw[1] & 0xF0) >> 4, self.raw[1] & 0x0F)
    }
}

impl UdsScanner {
	pub fn new(interface: Rc<UdsInterface>) -> UdsScanner {
		UdsScanner {
			interface,
		}
	}

	pub fn scan(&self) -> Result<Vec<Code>> {
		let response = self.interface.request(3, &[])?;
		let mut codes = Vec::with_capacity(response.len() / 2);
		// The first byte is the length
		for i in (2..response.len()).step_by(2) {
			codes.push(Code {
				raw: [response[i - 1], response[i]],
			});
		}
		Ok(codes)
	}
}