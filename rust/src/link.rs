use protocols::can::socketcan::SocketCan;
use protocols::can::CanInterface;
use protocols::isotp::{self, IsotpInterface, IsotpCan};
use protocols::uds::UdsInterface;
use definition;

use std::rc::Rc;

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
		let source_id = self.platform.transfer.
		isotp::Options {
			source_id: 
		}
	}

	/// Returns the UDS interface for the platform and datalink, if supported
	pub fn uds(&self) -> Some<Rc<UdsInterface>> {
		if let Some(isotp_interface) = self.link.isotp()
	}
}