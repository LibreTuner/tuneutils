extern crate libc;
extern crate j2534;

use std::io;
use std::mem;
use std::time;

use can::{Interface, Result, Error, Message};


impl From<j2534::Error> for Error {
    fn from(err: j2534::Error) -> Self {
        Error::J2534(err)
    }
}


pub struct J2534Can {
    channel: j2534::Channel,
}

impl J2534Can {
    /// Creates a new device from a J2534 channel. The channel must be a CAN channel.
    pub fn new(channel: j2534::Channel) -> Result<J2534Can> {
        J2534Can { channel }
    }

    /// Creates a CAN channel from a device with the specified baudrate
    pub fn connect(device: &j2534::Device, baudrate: u32) -> Result<J2534Can> {
        new(device.connect(j2534::Protocol::CAN, j2534::ConnectFlags::CAN_ID_BOTH, baudrate)?)
    }

    
}

impl Interface for J2534Can {
    fn send(&self, id: u32, message: &[u8]) -> Result<()> {

    }

    fn recv(&self, timeout: time::Duration) -> Result<Message> {

    }
}