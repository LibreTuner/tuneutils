extern crate byteorder;

pub mod isotp;

pub use self::isotp::UdsIsotp;

use error::{Error, Result};

use self::byteorder::{BigEndian, WriteBytesExt};

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
pub const UDS_REQ_READDATABYID: u8 = 0x22;

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

    fn read_data_by_identifier(&self, id: u16) -> Result<Vec<u8>> {
        let request = &[(id >> 8) as u8, (id & 0xFF) as u8];
        let mut res = self.request(UDS_REQ_READDATABYID, request)?;
        if res.len() < 2 {
            return Err(Error::InvalidPacket);
        }
        if res[0] != request[0] || res[1] != request[1] {
            // Check dataIdentifier
            return Err(Error::InvalidPacket);
        }
        // Remove dataIdentifier
        res.remove(0);
        res.remove(0);
        // Return dataRecord
        Ok(res)
    }
}