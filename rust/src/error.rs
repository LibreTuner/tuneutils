extern crate eval;
extern crate serde_yaml;
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

impl Error {
    fn as_str(&self) -> String {
        match *self {
            Error::Io(ref _io) => String::from("io error"),
            Error::InvalidConnection => String::from("invalid connection"),
            Error::InvalidResponse => String::from("invalid response"),
            Error::Timeout => String::from("timed out"),
            Error::TooMuchData => String::from("too much data"),
            Error::IncompleteWrite => String::from("only part of the data could be written"),
            Error::Read => String::from("failed to read"),
            #[cfg(feature = "j2534")]
            Error::J2534(ref _err) => String::from("J2534 error"),
            _ => format!("unimplemented: {:?}", *self)
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
/*
impl error::Error for Error {
    fn description(&self) -> &str {
        self.as_str()
    }
}*/