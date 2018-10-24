extern crate eval;
extern crate serde_yaml;

use std::result;
use std::io;
use std::fmt;
use std::error;


#[derive(Debug)]
pub enum Error {
	Io(io::Error),
	Eval(eval::Error),
	InvalidConnection,
    InvalidResponse,
    Timeout,
    TooMuchData,
    IncompleteWrite,
    Read,
    // ISO-TP Frame
    InvalidFrame,

    NegativeResponse(u8),
    InvalidPacket,

    Yaml(serde_yaml::Error),
    InvalidPlatformId,
    InvalidModelId,
    InvalidRomId,
    NotLoaded,
    InvalidTableId,
    NoTableOffset,

    /// Received an empty packet
    EmptyPacket,
    
    #[cfg(feature = "j2534")]
    J2534(j2534::Error),
}

pub type Result<T> = result::Result<T, Error>;

impl Error {
    fn as_str(&self) -> &str {
        match *self {
            Error::Io(ref _io) => "io error",
            Error::InvalidConnection => "invalid connection",
            Error::InvalidResponse => "invalid response",
            Error::Timeout => "timed out",
            Error::TooMuchData => "too much data",
            Error::IncompleteWrite => "only part of the data could be written",
            Error::Read => "failed to read",
            #[cfg(feature = "j2534")]
            Error::J2534(ref _err) => "J2534 error",
            _ => unimplemented!(),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Error {
        Error::Yaml(err)
    }
}

impl From<eval::Error> for Error {
    fn from(err: eval::Error) -> Error {
        Error::Eval(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        self.as_str()
    }
}