#[cfg(feature = "socketcan")]
use protocols::can::socketcan::SocketCan;
#[cfg(feature = "j2534")]
use protocols::can::j2534can::J2534Can;
#[cfg(feature = "j2534")]
extern crate j2534;

use protocols::can::CanInterface;
use protocols::isotp::{self, IsotpInterface, IsotpCan};
use protocols::uds::{UdsIsotp, UdsInterface};

use download::{self, Downloader};
use flash::{self, Flasher};
use error::Result;

use definition::{self, DownloadMode, FlashMode, LogMode};
use datalog;

use std::rc::Rc;
use std::time;

pub trait DataLink {
	/// Returns a CAN interface if supported
	fn can(&self, baudrate: usize) -> Option<Rc<CanInterface>>;

	/// Returns an ISO-TP interface if supported
	fn isotp(&self, options: isotp::Options) -> Option<Rc<IsotpInterface>>;
}

#[cfg(feature = "socketcan")]
pub struct SocketCanDataLink {
	interface: Rc<SocketCan>,
}


#[cfg(feature = "j2534")]
pub struct J2534DataLink {
	device: Rc<j2534::Device>,
}

#[cfg(feature = "j2534")]
impl J2534DataLink {
	pub fn new(device: Rc<j2534::Device>) -> J2534DataLink {
		J2534DataLink {
			device,
		}
	}
}

#[cfg(feature = "j2534")]
impl DataLink for J2534DataLink {
	fn can(&self, baudrate: usize) -> Option<Rc<CanInterface>> {
		 // Create a new CAN channel
		 if let Ok(interface) = J2534Can::connect(self.device.clone(), baudrate as u32) {
		 	return Some(Rc::new(interface));
		 }
		 // TODO: Process error
		 None
	}

	fn isotp(&self, options: isotp::Options) -> Option<Rc<IsotpInterface>> {
		// The PassThru device may have an ISO-TP layer, but we will use CAN for now
		// TODO: Replace hardcoded baudrate
		if let Some(interface) = self.can(500000) {
			return Some(Rc::new(IsotpCan::new(interface, options)));
		}
		None
	}
}

pub trait DataLinkEntry {
	/// Creates the datalink
	fn create(&self) -> Result<Box<DataLink>>;

	/// Returns the type of datalink
	fn typename(&self) -> &'static str;

	/// Returns the description of this datalink (specific to type)
	fn description(&self) -> String;
}

#[cfg(feature = "socketcan")]
pub struct SocketCanDataLinkEntry {
	pub interface: String,
}

#[cfg(feature = "socketcan")]
impl DataLinkEntry for SocketCanDataLinkEntry {
	fn create(&self) -> Result<Box<DataLink>> {
		Ok(Box::new(SocketCanDataLink::new(Rc::new(SocketCan::open(&self.interface)?))))
	}

	fn typename(&self) -> &'static str {
		"SocketCAN"
	}

	fn description(&self) -> String {
		String::from("Interface: ") + &self.interface
	}
}

#[cfg(feature = "j2534")]
pub struct J2534DataLinkEntry {
	pub entry: j2534::Listing,
}

#[cfg(feature = "j2534")]
impl DataLinkEntry for J2534DataLinkEntry {
	fn create(&self) -> Result<Box<DataLink>> {
		// Load interface and open any device
		let interface = Rc::new(j2534::Interface::new(&self.entry.path)?);
		let device = Rc::new(j2534::Device::open_any(interface)?);

		Ok(Box::new(J2534DataLink::new(device)))
	}

	fn typename(&self) -> &'static str {
		"PassThru"
	}

	fn description(&self) -> String {
		self.entry.name.clone()
	}
}

/// Searches for any connected datalinks
pub fn discover_datalinks() -> Vec<Box<DataLinkEntry>> {
	let mut links = Vec::new();

	// Search for PassThru interfaces
	#[cfg(feature = "j2534")]
	{
		if let Ok(list) = j2534::list() {
			for listing in list {
				let entry: Box<DataLinkEntry> = Box::new(J2534DataLinkEntry {
					entry: listing,
				});
				links.push(entry);
			}
		} // Else, listing failed (TODO: log error?)
	}

	links
}

#[cfg(feature = "socketcan")]
impl SocketCanDataLink {
	pub fn new(interface: Rc<SocketCan>) -> SocketCanDataLink {
		SocketCanDataLink {
			interface,
		}
	}
}

#[cfg(feature = "socketcan")]
impl DataLink for SocketCanDataLink {
	/// Returns a CAN interface if supported
	fn can(&self, _baudrate: usize) -> Option<Rc<CanInterface>> {
		// The baudrate for the socketcan device is declared externally
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
	pub fn new(link: Box<DataLink>, platform: Rc<definition::Main>) -> PlatformLink {
		PlatformLink {
			link,
			platform,
		}
	}
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

	/// Returns the datalogging interface for the platform, if supported by the platform AND datalink
	pub fn datalogger(&self) -> Option<Box<datalog::Logger>> {
		match self.platform.log_mode {
			LogMode::Uds => {
				if let Some(uds_interface) = self.uds() {
					return Some(Box::new(datalog::UdsLogger::new(uds_interface)));
				}
				None
			},
			_ => None,
		}
	}
}