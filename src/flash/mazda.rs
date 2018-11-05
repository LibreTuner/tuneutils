extern crate byteorder;

use super::{Flasher, FlashData};

use error::Result;

use protocols::uds::{self, UdsInterface};
use authenticator::MazdaAuthenticator;

use self::byteorder::{BigEndian, WriteBytesExt};

use std::cmp;
use std::rc::Rc;


pub struct Mazda1Flasher {
	interface: Rc<UdsInterface>,
	key: String,
}

impl Mazda1Flasher {
	pub fn new(interface: Rc<UdsInterface>, key: &str) -> Mazda1Flasher {
		Mazda1Flasher {
			interface,
			key: key.to_string(),
		}
	}
}

impl Mazda1Flasher {
	/// Erases the ECU for writing. It MUST be erased before writing
	/// or the microcontroller controller could be damaged.
	fn erase(&self) -> Result<()> {
		self.interface.request(0xB1, &[0x00, 0xB2, 0x00])?;
		Ok(())
	}
}

impl Flasher for Mazda1Flasher {
	fn flash(&self, data: &FlashData) -> Result<()> {
		// Authenticate
		let auth = MazdaAuthenticator{};
		auth.authenticate(&self.key, &*self.interface, 0x85)?;

		// Erase
		self.erase()?;

		// Request download
		let mut msg = [0; 8];
		{
			let mut writer = &mut msg as &mut [u8];
			writer.write_u32::<BigEndian>(data.offset as u32)?;
			writer.write_u32::<BigEndian>(data.data.len() as u32)?;
		}
		// Mazda does not use the standard download request, so we make a packet from scratch
		self.interface.request(uds::UDS_REQ_REQUESTDOWNLOAD, &msg)?;

		// Upload
		let mut buffer = data.data;
		let mut sent = 0;
		while buffer.len() != 0 {
			let to_send = cmp::min(buffer.len(), 0xFFE);
			self.interface.request(uds::UDS_REQ_TRANSFERDATA, &buffer[0..=to_send])?;
			sent += to_send;
			buffer = &data.data[sent..];

			// Call the update callback
			if let Some(ref cb) = data.callback {
				let mut closure = cb.borrow_mut();
				(&mut *closure)(sent as f32 / data.data.len() as f32);
			}
		}

		Ok(())
	}
}