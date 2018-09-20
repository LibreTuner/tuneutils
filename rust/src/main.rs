extern crate tuneutils;

use tuneutils::protocols::can;
use tuneutils::protocols::isotp;

use can::Interface as CanInterface;
use can::InterfaceIterator as CanInterfaceIterator;
use isotp::Interface as IsotpInterface;
#[cfg(feature = "socketcan")]
use can::SocketCan;

use std::str;

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
    
    let iface = isotp::can::CanInterface::new(&can, isotp::Options::default());

    let response = iface.request(&[9, 2]).unwrap();
    let _uds_res = response[0];
    let _pid = response[1];

    let vin = str::from_utf8(&response[3..]).unwrap();
    println!("Response: {}", vin);
}