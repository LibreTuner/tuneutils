use protocols::can::Interface as Can;

use super::{Interface, Result, Error, Frame, FrameType, Options};

pub struct CanInterface<'a> {
    can: &'a Can,
    options: Options,
}

impl<'a> CanInterface<'a> {
    pub fn new(can: &Can, options: Options) -> CanInterface {
        CanInterface {can, options}
    }

    fn send_frame(&self, frame: &Frame) -> Result<()> {
        self.can.send(self.options.dest_id, &frame.data)?;
        Ok(())
    }
}

impl<'a> Interface for CanInterface<'a> {
    fn recv(&self) -> Result<Vec<u8>> {
        
    }

    fn send(&self, data: &[u8]) -> Result<()> {
        if data.len() <= 7 {
            // Send a single frame
            self.send_frame(Frame::from_single(&data));
        } else {
            // Send a first frame
            self.send_frame(Frame::from_first(&data[..6]));
            // Get flow control and send consecutive frames
        }
    }
}