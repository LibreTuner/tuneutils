use super::{Downloader, DownloadResponse, DownloadCallback, Result, Error};

use protocols::uds::UdsInterface;
use authenticator::MazdaAuthenticator;

use std::cmp;


pub struct Mazda1Downloader<'a> {
	interface: &'a UdsInterface,
	key: String,
	download_size: usize,
}

impl<'a> Mazda1Downloader<'a> {
	pub fn new<'b>(interface: &'b UdsInterface, key: &str, download_size: usize) -> Mazda1Downloader<'b> {
		Mazda1Downloader {
			interface,
			key: key.to_string(),
			download_size
		}
	}
}

impl<'a> Downloader for Mazda1Downloader<'a> {
	fn download(&self, callback: &DownloadCallback) -> Result<DownloadResponse> {
		let auth = MazdaAuthenticator{};
		auth.authenticate(&self.key, self.interface, 0x87)?;

		// Start downloading through ReadMemoryByAddress
		let mut data = Vec::with_capacity(self.download_size);
		let mut offset = 0 as u32;
		let mut remaining = self.download_size as u32;

		while remaining > 0 {
			let section = self.interface.request_read_memory_address(offset, cmp::min(remaining, 0xFFE) as u16)?;

			if section.is_empty() {
				return Err(Error::EmptyPacket);
			}

			data.extend_from_slice(&section);
			offset += section.len() as u32;
			remaining -= section.len() as u32;

			// Call the update callback
			if let Some(ref cb) = callback.callback {
				let mut closure = cb.borrow_mut();
				(&mut *closure)((self.download_size as u32 - remaining) as f32 / self.download_size as f32);
			}
		}

		Ok(DownloadResponse {data})
	}
}