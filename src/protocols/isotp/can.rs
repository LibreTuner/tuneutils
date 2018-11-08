use crate::{
    protocols::can::CanInterface,
    error::{Error, Result},
};

use super::{IsotpInterface, Frame, Options, FCFlag, FlowControlFrame, FirstFrame, SingleFrame};
use std::cmp;
use std::time;
use std::thread;
use std::rc::Rc;

pub struct IsotpCan {
    can: Rc<CanInterface>,
    options: Options,
}

fn st_to_duration(st: u8) -> time::Duration {
    if st <= 127 {
        return time::Duration::from_millis(st as u64);
    }
    time::Duration::from_micros(st as u64)
}

impl IsotpCan {
    pub fn new(can: Rc<CanInterface>, options: Options) -> IsotpCan {
        IsotpCan {can, options}
    }

    fn send_frame(&self, frame: &Frame) -> Result<()> {
        self.can.send(self.options.source_id, &frame.data)?;
        Ok(())
    }

    fn send_flow_control_frame(&self, flow: FlowControlFrame) -> Result<()> {
        self.send_frame(&Frame::from_flow(flow))
    }

    fn recv_frame(&self) -> Result<Frame> {
        let start_time = time::Instant::now();
        for msg in self.can.recv_iter(self.options.timeout) {
            let msg = msg?;
            if msg.id == self.options.dest_id {
                let mut data = [0; 8];
                data[..msg.data.len()].copy_from_slice(&msg.data);
                return Ok(Frame::new(data));
            }
            if start_time.elapsed() >= self.options.timeout {
                return Err(Error::Timeout);
            }
        }
        Err(Error::Timeout)
    }

    fn recv_flow_control_frame(&self) -> Result<FlowControlFrame> {
        let frame = self.recv_frame()?;
        if frame.data[0] & 0xF0 != 0x30 {
            return Err(Error::InvalidFrame);
        }

        let fc_flag_i = frame.data[0] & 0x0F;
        if fc_flag_i > 2 {
            return Err(Error::InvalidFrame);
        }
        let fc_flag = match fc_flag_i {
            0 => FCFlag::Continue,
            1 => FCFlag::Wait,
            2 => FCFlag::Overflow,
            _ => FCFlag::Continue, // This will never happen because of the above check but rust complains
        };

        Ok(FlowControlFrame {
            flag: fc_flag,
            block_size: frame.data[1],
            separation_time: st_to_duration(frame.data[2])
        })
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

    fn eof(&self) -> bool {
        self.buffer.is_empty()
    }
}

impl IsotpInterface for IsotpCan {
    fn recv(&self) -> Result<Vec<u8>> {
        // Receive first or single frame
        let frame = self.recv_frame()?;
        let frame_id = frame.data[0] & 0xF0;

        if frame_id == 0 {
            // Single frame
            let single_frame = SingleFrame::new(&frame.data)?;
            return Ok(single_frame.data[..single_frame.length as usize].to_vec());
        } else if frame_id == 0x10 {
            // First frame
            let first_frame = FirstFrame::new(&frame.data)?;

            let mut buffer = Vec::new();
            buffer.extend_from_slice(&first_frame.data);

            let mut remaining = first_frame.length as usize - buffer.len();
            // Send the flow control frame
            self.send_flow_control_frame(FlowControlFrame {
                flag: FCFlag::Continue,
                block_size: 0,
                separation_time: st_to_duration(0)
            })?;

            // Wait for all consecutive packets
            let mut index = 1;
            while remaining > 0 {
                let frame = self.recv_frame()?;
                let id = frame.data[0] & 0xF0;
                if id != 0x20 {
                    // Not a consec frame
                    return Err(Error::InvalidFrame);
                }
                if frame.data[0] & 0x0F != index {
                    // Invalid index
                    return Err(Error::InvalidFrame);
                }

                let len = cmp::min(remaining, 7);
                buffer.extend_from_slice(&frame.data[1..=len]);
                remaining -= len;

                index += 1;
                if index == 16 {
                    index = 0;
                }
            }
            return Ok(buffer);
        }
        Err(Error::InvalidFrame)
    }

    fn send(&self, data: &[u8]) -> Result<()> {
        if data.len() <= 7 {
            // Send a single frame
            self.send_frame(&Frame::from_single_data(&data))?;
        } else {
            let mut packet = SendPacket::new(&data);
            // Send a first frame
            self.send_frame(&packet.first_frame())?;
            // Get flow control and send consecutive frames
            let mut flow_control = self.recv_flow_control_frame()?;
            while !packet.eof() {
                // Loop until the buffer is empty
                if flow_control.separation_time != time::Duration::new(0, 0) {
                    thread::sleep(flow_control.separation_time);
                }

                self.send_frame(&packet.next_consec_frame())?;

                if !packet.eof() && flow_control.block_size > 0 {
                    flow_control.block_size -= 1;
                    if flow_control.block_size == 0 {
                        // Get the next flow control packet
                        flow_control = self.recv_flow_control_frame()?;
                    }
                }
            }
        }
        Ok(())
    }
}