extern crate tuneutils;

use tuneutils::protocols::can;
use tuneutils::protocols::isotp;
use tuneutils::protocols::uds::{UdsInterface, UdsIsotp};
use tuneutils::download;
use tuneutils::download::Downloader;
use tuneutils::definition;

use std::path::Path;

use can::{CanInterface, CanInterfaceIterator};
use isotp::IsotpInterface;
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
        let iface = isotp::IsotpCan::new(&can, isotp::Options::default());
        let uds = UdsIsotp::new(&iface);
        
        println!("{:?}", uds.request(0x01, &[0x05]));
    }
}

#[cfg(feature = "socketcan")]
fn main() {
    let mut defs = definition::Definitions::default();
    defs.load(&Path::new("/home/altenius/projects/LibreTuner/src/resources/definitions")).unwrap();
    println!("Definitions: {:?}", defs.definitions);
    /*
    let can = SocketCan::open("slcan0").expect("Failed to find slcan0");
    
    let iface = isotp::IsotpCan::new(&can, isotp::Options::default());
    let uds = UdsIsotp::new(&iface);

    let downloader = download::mazda::Mazda1Downloader::new(&uds, "MazdA", 1024*5);
    println!("Downloading");
    let data = downloader.download().unwrap();

    println!("Got data: {:?}", data.data);*/
}