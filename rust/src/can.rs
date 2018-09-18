#[cfg(windows)]
extern crate j2534;

use std::io;
use std;
use std::time;
use std::fmt;

use std::error::Error as StdError;

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
    
    #[cfg(windows)]
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
            #[cfg(windows)]
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

impl Message {
    pub fn new() -> Message {
        Message {
            id: 0,
            data: Vec::new(),
        }
    }
}

pub trait Interface {
    fn send(&self, id: u32, message: &[u8]) -> Result<()>;

    fn recv(&self, timeout: time::Duration) -> Result<Message>;
}