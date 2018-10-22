use protocols::isotp::IsotpInterface;

use std::rc::Rc;

use super::{UdsInterface, UDS_NRES_RCRRP};
use error::{Error, Result};

pub struct UdsIsotp {
    interface: Rc<IsotpInterface>,
}

impl UdsIsotp {
    pub fn new(interface: Rc<IsotpInterface>) -> UdsIsotp {
        UdsIsotp {interface}
    }
}

impl UdsInterface for UdsIsotp {
    fn request(&self, request_sid: u8, data: &[u8]) -> Result<Vec<u8>> {
        let mut v = Vec::new();
        v.push(request_sid);
        v.extend_from_slice(&data);

        self.interface.send(&v)?;
        // Receive packets until we get a non-response-pending packet
        loop {
            let response = self.interface.recv()?;
            if response.is_empty() {
                return Err(Error::InvalidPacket);
            }

            if response[0] == 0x7F {
                // Negative code
                if response.len() > 1 {
                    if response[1] == UDS_NRES_RCRRP {
                        // Request correctly received, response pending
                        continue;
                    }
                    return Err(Error::NegativeResponse(response[1]));
                }
                return Err(Error::NegativeResponse(0));
            }

            if response[0] != request_sid + 0x40 {
                return Err(Error::InvalidPacket);
            }

           return Ok(response[1..].to_vec())
        }
    }
}