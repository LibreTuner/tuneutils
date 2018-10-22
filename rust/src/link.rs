use protocols::can::socketcan::SocketCan;
use protocols::can::CanInterface;
use protocols::isotp::{self, IsotpInterface, IsotpCan};
use protocols::uds::{UdsIsotp, UdsInterface};

use download::{self, Downloader};
use flash::{self, Flasher};

use definition::{self, DownloadMode, FlashMode};

use std::rc::Rc;
use std::time;

pub trait DataLink {
	/// Returns a CAN interface if supported
	fn can(&self) -> Option<Rc<CanInterface>>;

	/// Returns an ISO-TP interface if supported
	fn isotp(&self, options: isotp::Options) -> Option<Rc<IsotpInterface>>;
}

pub struct SocketCanDataLink {
	interface: Rc<SocketCan>,
}


pub struct J2534DataLink {

}

impl SocketCanDataLink {
	pub fn new(interface: Rc<SocketCan>) -> SocketCanDataLink {
		SocketCanDataLink {
			interface,
		}
	}
}


impl DataLink for SocketCanDataLink {
	/// Returns a CAN interface if supported
	fn can(&self) -> Option<Rc<CanInterface>> {
		Some(self.interface.clone())
	}

	/// Returns an ISO-TP interface if supported
	fn isotp(&self, options: isotp::Options) -> Option<Rc<IsotpInterface>> {
		let iface = IsotpCan::new(self.interface.clone(), options);
		Some(Rc::new(iface))
	}
}


pub struct PlatformLink {
	link: Box<DataLink>,
	platform: Rc<definition::Main>,
}

impl PlatformLink {
	/// Returns the ISO-TP options for the platform
	pub fn isotp_options(&self) -> isotp::Options {
		let server_id = self.platform.transfer.server_id as u32;
		isotp::Options {
			source_id: server_id,
			dest_id: server_id + 0x08,
			// TODO: Pull this from a config
			timeout: time::Duration::from_secs(1),
		}
	}

	/// Returns the ISO-TP interface for the platform, if supported
	pub fn isotp(&self) -> Option<Rc<IsotpInterface>> {
		self.link.isotp(self.isotp_options())
	}

	/// Returns the UDS interface for the platform, if supported
	pub fn uds(&self) -> Option<Rc<UdsInterface>> {
		if let Some(isotp_interface) = self.isotp() {
			return Some(Rc::new(UdsIsotp::new(isotp_interface)));
		}
		None
	}

	/// Returns the downloader for the platform, if supported by the platform AND datalink
	pub fn downloader(&self) -> Option<Box<Downloader>> {
		match self.platform.transfer.download_mode {
			DownloadMode::Mazda1 => {
				if let Some(uds_interface) = self.uds() {
					return Some(Box::new(download::mazda::Mazda1Downloader::new(uds_interface, &self.platform.transfer.key, self.platform.rom_size)));
				}
				None
			},
			_ => None,
		}
	}

	/// Returns the flash interface for the platform, if supported by the platform AND datalink
	pub fn flasher(&self) -> Option<Box<Flasher>> {
		match self.platform.transfer.flash_mode {
			FlashMode::Mazda1 => {
				if let Some(uds_interface) = self.uds() {
					return Some(Box::new(flash::mazda::Mazda1Flasher::new(uds_interface, &self.platform.transfer.key)));
				}
				None
			},
			_ => None,
		}
	}
}