// Utilities for downloading ROMs

pub mod mazda;

use std::cell::RefCell;
use error::Result;

pub struct DownloadCallback {
	pub callback: Option<Box<RefCell<FnMut(f32)>>>,
}

impl DownloadCallback {
	pub fn null() -> Self {
		DownloadCallback {
			callback: None,
		}
	}

	pub fn with<CB: 'static + FnMut(f32)>(cb: CB) -> Self {
		DownloadCallback {
			callback: Some(Box::new(RefCell::new(cb))),
		}
	}
}

pub struct DownloadResponse {
    pub data: Vec<u8>,
}

pub trait Downloader {
    fn download(&self, callback: &DownloadCallback) -> Result<DownloadResponse>;
}
/*
/// Returns a downloader suitable for a platform
pub fn downloader_for(definition: &Main) -> Option<Box<Downloader>> {
	match definition.transfer.download_mode {
		DownloadMode::Mazda1 => Some(Box::new(mazda::Mazda1Downloader::new()))
	}
}*/