use protocols::can::Interface as Can;

use super::{Interface, Result, Error, Frame, FrameType};

pub struct CanInterface {
    can: Box<Can>,
}

impl CanInterface {
    fn new(can: Box<Can>) -> CanInterface {
        CanInterface {can}
    }

    fn send_single_frame(&self, data: &[u8]) -> Result<()> {
        debug_assert!(data.len() <= 7);

    }

    fn send_first_frame(&self, data: &[u8]) -> Result<()> {

    }
}

impl Interface for CanInterface {
    fn recv(&self) -> Result<Vec<u8>> {
        
    }

    fn send(&self, data: &[u8]) -> Result<()> {
        if data.len() <= 7 {
            // Send a single frame
            self.send_single_frame(&data);
        } else {
            // Send a first frame
            self.send_first_frame(&data[..6]);
            // Get flow control and send consecutive frames
        }
    }
}