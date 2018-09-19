use protocols::can::Interface as Can;

use super::{Interface, Result, Error, Frame, FrameType, Options};
use std::cmp;
use std::time;

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

    fn recv_frame(&self) -> Result<Frame> {
        let start_time = time::Instance::now();
        for msg in self.can.recv_iter(self.options.timeout) {
            let msg = msg?;
            if msg.id == self.options.source_id {
                let mut data = [0; 8];
                data[..msg.data.len()].copy_from_slice(&msg.data);
                return Ok(Frame::new(data));
            }
            if start_time >= self.options.timeout {
                return Err(Error::Timeout);
            }
        }
        Err(Error::Timeout)
    }
}

struct SendPacket<'a> {
    buffer: &'a [u8],
    index: u8,
}

/// Used for sending mutli-frame packets.
/// It is NOT used for single-frame packets.
impl<'a> SendPacket<'a> {
    fn new(buffer: &[u8]) -> SendPacket {
        assert!(buffer.len() <= 4095);
        SendPacket {buffer, index: 0}
    }

    fn first_frame(&mut self) -> Frame {
        let len = cmp::min(self.buffer.len(), 6);
        let frame = Frame::from_first_data(&self.buffer[..len], self.buffer.len() as u16);
        self.buffer = &self.buffer[len..];
        self.index = 1;
        frame
    }

    fn next_consec_frame(&mut self) -> Frame {
        let len = cmp::min(self.buffer.len(), 7);
        let frame = Frame::from_consec_data(&self.buffer[..len], self.index);
        self.buffer = &self.buffer[len..];
        self.index += 1;
        if self.index == 16 {
            self.index = 0;
        }
        frame
    }
}

impl<'a> Interface for CanInterface<'a> {
    fn recv(&self) -> Result<Vec<u8>> {
        
    }

    fn send(&self, data: &[u8]) -> Result<()> {
        if data.len() <= 7 {
            // Send a single frame
            self.send_frame(&Frame::from_single_data(&data));
        } else {
            let mut packet = SendPacket::new(&data);
            // Send a first frame
            self.send_frame(&packet.first_frame());
            // Get flow control and send consecutive frames
        }
    }
}