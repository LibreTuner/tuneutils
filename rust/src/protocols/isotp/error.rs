use std::io;
use std::result;

use protocols::can::Error as CanError;

pub enum Error {
    Io(io::Error),
    Can(CanError),
}

pub type Result<T> = result::Result<T, Error>;