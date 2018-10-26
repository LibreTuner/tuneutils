extern crate eval;
extern crate serde_yaml;
#[cfg(feature = "j2534")]
extern crate j2534;

use std::result;
use std::io;
use std::fmt;


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
        match *self {
            Error::Io(ref err) => write!(f, "IO error: {}", err),
            Error::Eval(ref err) => write!(f, "Eval error: {}", err),
            Error::InvalidConnection => write!(f, "Invalid connection"),
            Error::InvalidResponse => write!(f, "Invalid response"),
            Error::Timeout => write!(f, "Timed out"),
            Error::TooMuchData => write!(f, "Too much data"),
            Error::IncompleteWrite => write!(f, "Failed to finish writing data"),
            Error::Read => write!(f, "Read failed"),
            Error::InvalidFrame => write!(f, "Invalid frame"),
            Error::NegativeResponse(code) => write!(f, "Negative ISO-TP response received ({})", code),
            Error::InvalidPacket => write!(f, "Invalid packet received"),
            Error::Yaml(ref err) => write!(f, "Yaml error: {}", err),
            Error::InvalidPlatformId => write!(f, "Invalid platform id"),
            Error::InvalidModelId => write!(f, "Invalid model id"),
            Error::InvalidRomId => write!(f, "Invalid rom id"),
            Error::NotLoaded => write!(f, "Not loaded"),
            Error::InvalidTableId => write!(f, "Invalid table id"),
            Error::NoTableOffset => write!(f, "No table offset"),
            Error::EmptyPacket => write!(f, "Received an empty packet"),
            #[cfg(feature = "j2534")]
            Error::J2534(ref err) => write!(f, "J2534 error: {}", err),
            _ => write!(f, "unimplemented: {:?}", *self),
        }
    }
}