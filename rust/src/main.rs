extern crate tuneutils;

use tuneutils::protocols::can;

use can::{Interface, InterfaceIterator};
#[cfg(feature = "socketcan")]
use can::SocketCan;

#[cfg(feature = "j2534")]
extern crate j2534;
#[cfg(feature = "j2534")]
fn main() {
    for listing in j2534::list().unwrap() {
        println!("Loading {}", listing.name);
        let interface = j2534::Interface::new(&listing.path).unwrap();
        let device = interface.open_any().unwrap();
        let can = can::J2534Can::connect(&device, 500000).unwrap();
        can.apply_blank_filter().unwrap();
        loop {
            let msg = can.recv(std::time::Duration::from_secs(1)).unwrap();
            println!("Message: {:?}", msg);
        }
    }
}

#[cfg(feature = "socketcan")]
fn main() {
    let can = SocketCan::open("slcan0").expect("Failed to find slcan0");
    for msg in can.recv_iter(std::time::Duration::from_secs(1)) {
        println!("Message: {}", msg.unwrap());
    }
}