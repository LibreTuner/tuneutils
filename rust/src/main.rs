extern crate tuneutils;

use tuneutils::can::Interface;

extern crate j2534;

fn main() {
    for listing in j2534::list().unwrap() {
        println!("Loading {}", listing.name);
        let interface = j2534::Interface::new(&listing.path).unwrap();
        let device = interface.open_any().unwrap();
        let can = tuneutils::j2534can::J2534Can::connect(&device, 500000).unwrap();
        can.apply_blank_filter().unwrap();
        loop {
            let msg = can.recv(std::time::Duration::from_secs(1)).unwrap();
            println!("Message: {:?}", msg);
        }
    }
}
