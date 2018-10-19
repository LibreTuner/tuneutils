extern crate byteorder;

use std::result;
use std::convert;

pub mod isotp;

pub use self::isotp::UdsIsotp;

use protocols::isotp::error::Error as IsotpError;

use self::byteorder::{BigEndian, WriteBytesExt};

#[derive(Debug)]
pub enum Error {
    NegativeResponse(u8),
    Isotp(IsotpError),
    InvalidPacket,
}

impl convert::From<IsotpError> for Error {
    fn from(error: IsotpError) -> Error {
        Error::Isotp(error)
    }
}

pub type Result<T> = result::Result<T, Error>;

pub struct Response {
    pub data: Vec<u8>,
}

// Request SIDs
pub const UDS_REQ_SESSION: u8 = 0x10;
pub const UDS_REQ_SECURITY: u8 = 0x27;
pub const UDS_REQ_READMEM: u8 = 0x23;
pub const UDS_REQ_REQUESTDOWNLOAD: u8 = 0x34;
pub const UDS_REQ_REQUESTUPLOAD: u8 = 0x35;
pub const UDS_REQ_TRANSFERDATA: u8 = 0x36;

// Negative response codes
// requestCorrectlyReceivedResponsePending
pub const UDS_NRES_RCRRP: u8 = 0x78;

pub trait UdsInterface {
    fn request(&self, request_sid: u8, data: &[u8]) -> Result<Vec<u8>>;

    /// Sends a DiagnosticSessionControl request. Returns parameter record.
    fn request_session(&self, session_type: u8) -> Result<Vec<u8>> {
    	let mut response = self.request(UDS_REQ_SESSION, &[session_type])?;
    	if response.is_empty() {
    		return Err(Error::InvalidPacket);
    	}

    	if response[0] != session_type {
    		return Err(Error::InvalidPacket);
    	}

    	response.remove(0);
    	Ok(response)
    }

    fn request_security_seed(&self) -> Result<Vec<u8>> {
    	let mut response = self.request(UDS_REQ_SECURITY, &[1])?;

    	if response.is_empty() {
    		return Err(Error::InvalidPacket);
    	}

    	if response[0] != 1 {
    		return Err(Error::InvalidPacket);
    	}

    	response.remove(0);
    	Ok(response)
    }

    fn request_security_key(&self, key: &[u8]) -> Result<()> {
    	let mut request = Vec::with_capacity(key.len() + 1);
    	request.push(2);
    	request.extend_from_slice(&key);

    	let _response = self.request(UDS_REQ_SECURITY, &request)?;
    	Ok(())
    }

    fn request_read_memory_address(&self, address: u32, length: u16) -> Result<Vec<u8>> {
    	let mut request = [0; 6];
    	{
	    	let mut writer: &mut [u8] = &mut request;
	    	writer.write_u32::<BigEndian>(address).unwrap();
	    	writer.write_u16::<BigEndian>(length).unwrap();
		}
		self.request(UDS_REQ_READMEM, &request)
    }
}