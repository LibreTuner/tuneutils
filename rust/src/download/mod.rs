// Utilities for downloading ROMs

pub mod mazda;

use protocols::uds;
use std::convert;
use std::result;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Uds(uds::Error),
    /// Received an empty packet
    EmptyPacket,
}

impl convert::From<uds::Error> for Error {
    fn from(error: uds::Error) -> Error {
        Error::Uds(error)
    }
}

pub struct DownloadResponse {
    pub data: Vec<u8>,
}

pub trait Downloader {
    fn download(&self) -> Result<DownloadResponse>;
}