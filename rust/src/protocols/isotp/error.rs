use std::io;
use std::result;

use protocols::can::Error as CanError;

pub enum Error {
    Io(io::Error),
    Can(CanError),
    Timeout,
    InvalidFrame,
}

pub type Result<T> = result::Result<T, Error>;

impl From<CanError> for Error {
    fn from(error: CanError) -> Error {
        Error::Can(error)
    }
}