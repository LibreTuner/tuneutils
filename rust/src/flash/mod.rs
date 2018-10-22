// Utilities for flashing devices

pub mod mazda;

use std::cell::RefCell;

use error::Result;

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