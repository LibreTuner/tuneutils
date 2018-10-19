#[cfg(feature = "j2534")]
extern crate j2534;
extern crate itertools;

use std::io;
use std;
use std::time;
use std::fmt;
use std::iter;
use self::itertools::Itertools;


#[cfg(feature = "j2534")]
pub mod j2534can;
#[cfg(feature = "j2534")]
pub use j2534can::J2534Can;

#[cfg(feature = "socketcan")]
pub mod socketcan;
#[cfg(feature = "socketcan")]
pub use self::socketcan::SocketCan;


pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    InvalidConnection,
    InvalidResponse,
    Timeout,
    TooMuchData,
    IncompleteWrite,
    ReadError,
    
    #[cfg(feature = "j2534")]
    J2534(j2534::Error),
}

impl Error {
    fn as_str(&self) -> &str {
        match *self {
            Error::Io(ref _io) => "io error",
            Error::InvalidConnection => "invalid connection",
            Error::InvalidResponse => "invalid response",
            Error::Timeout => "timed out",
            Error::TooMuchData => "too much data",
            Error::IncompleteWrite => "only part of the data could be written",
            Error::ReadError => "failed to read",
            #[cfg(feature = "j2534")]
            Error::J2534(ref _err) => "J2534 error",
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        self.as_str()
    }
}

#[derive(Debug)]
pub struct Message {
    pub id: u32,
    pub data: Vec<u8>,
}

impl Default for Message {
    fn default() -> Message {
        Message {
            id: 0,
            data: Vec::new(),
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{:X}] {}", self.id, self.data.iter()
                                               .map(|x| format!("{:X}", x))
                                               .join(" "))?;
        Ok(())
    }
}

/// Used to implement recv_iter
pub trait CanInterfaceIterator {
    fn recv_iter(&self, timeout: time::Duration) -> RecvIterator;
}

pub trait CanInterface: CanInterfaceIterator {
    /// Sends a CAN message through the interface.
    /// 
    /// # Arguments
    /// 
    /// * `id` - The CAN id of the message
    /// * `message` - The message data. Must not be larger than 8 bytes
    fn send(&self, id: u32, message: &[u8]) -> Result<()>;

    fn send_msg(&self, message: &Message) -> Result<()> {
        self.send(message.id, &message.data)
    }

    /// Received a single message from the interface.
    /// If no messages are received before the timeout, returns `Error::Timeout`
    /// 
    /// # Arguments
    /// 
    /// * `timeout` - The time to wait for a message before returning
    fn recv(&self, timeout: time::Duration) -> Result<Message>;
}

impl<S: Sized + CanInterface> CanInterfaceIterator for S {
    fn recv_iter(&self, timeout: time::Duration) -> RecvIterator {
        RecvIterator {
            interface: self,
            timeout
        }
    }
}

pub struct RecvIterator<'a> {
    interface: &'a (CanInterface + 'a),
    timeout: time::Duration,
}

impl<'a> iter::Iterator for RecvIterator<'a> {
    type Item = Result<Message>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.interface.recv(self.timeout))
    }
}