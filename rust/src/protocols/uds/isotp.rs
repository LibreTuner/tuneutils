use protocols::isotp::IsotpInterface;

use super::{UdsInterface, Response, Result, Error};

pub struct UdsIsotp<'a> {
    interface: &'a IsotpInterface,
}

impl<'a> UdsIsotp<'a> {
    pub fn new(interface: &IsotpInterface) -> UdsIsotp {
        UdsIsotp {interface}
    }
}

impl<'a> UdsInterface for UdsIsotp<'a> {
    fn request(&self, request_sid: u8, data: &[u8]) -> Result<Vec<u8>> {
        let mut v = Vec::new();
        v.push(request_sid);
        v.extend_from_slice(&data);

        let response = self.interface.request(&v)?;
        if response.is_empty() {
            return Err(Error::InvalidPacket);
        }

        if response[0] == 0x7F {
            // Negative code
            if response.len() > 1 {
                return Err(Error::NegativeResponse(response[1]));
            }
            return Err(Error::NegativeResponse(0));
        }

        if response[0] != request_sid + 0x40 {
            return Err(Error::InvalidPacket);
        }

        Ok(response[1..].to_vec())
    }
}