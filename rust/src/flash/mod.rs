// Utilities for flashing devices

pub mod mazda;

use protocols::uds;
use std::convert;
use std::result;
use std::io;
use std::cell::RefCell;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Uds(uds::Error),
    Io(io::Error),
}

impl convert::From<uds::Error> for Error {
    fn from(error: uds::Error) -> Error {
        Error::Uds(error)
    }
}

impl convert::From<io::Error> for Error {
	fn from(error: io::Error) -> Error {
		Error::Io(error)
	}
}

pub struct FlashData<'a> {
	pub offset: usize,
	pub data: &'a [u8],

	pub callback: Option<Box<RefCell<FnMut(f32)>>>,
}

impl<'a> FlashData<'a> {
	pub fn new(offset: usize, data: &[u8]) -> FlashData {
		FlashData {
			offset,
			data,
			callback: None,
		}
	}

	pub fn with_callback<CB: 'static + FnMut(f32)>(mut self, cb: CB) -> Self {
		self.callback = Some(Box::new(RefCell::new(cb)));
		self
	}
}

pub trait Flasher {
    fn flash(&self, data: &FlashData) -> Result<()>;
}