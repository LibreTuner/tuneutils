use std::time;
use std::default::Default;

pub mod error;
pub mod can;

pub type IsotpCan<'a> = can::IsotpCan<'a>;

use self::error::{Error, Result};
use std::cmp;

pub struct Options {
    source_id: u32,
    dest_id: u32,
    timeout: time::Duration,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            source_id: 0x7e0,
            dest_id: 0x7e8,
            timeout: time::Duration::from_secs(1),
        }
    }
}

pub enum FrameType {
    Single,
    First,
    Consecutive,
    Flow,
}

#[derive(Debug)]
pub enum FCFlag {
    Continue = 0,
    Wait = 1,
    Overflow = 2,
}

#[derive(Debug)]
pub struct FlowControlFrame {
    flag: FCFlag,
    block_size: u8,
    separation_time: time::Duration,
}

#[derive(Debug)]
pub struct SingleFrame {
    length: u8,
    data: [u8; 7],
}

impl SingleFrame {
    fn new(frame: &[u8]) -> Result<SingleFrame> {
        if frame.is_empty() {
            return Err(Error::InvalidFrame);
        }
        if frame[0] & 0xF0 != 0 {
            // Not a single frame
            return Err(Error::InvalidFrame);
        }
        let len = cmp::min(7, frame[0] & 0x0F) as usize;
        let mut data = [0; 7];
        data[0..len].copy_from_slice(&frame[1..=len]);
        Ok(SingleFrame {
            data,
            length: len as u8,
        })
    }
}

#[derive(Debug)]
pub struct FirstFrame {
    length: u16,
    data: [u8; 6],
}

impl FirstFrame {
    fn new(frame: &[u8]) -> Result<FirstFrame> {
        assert!(frame.len() <= 8);
        if frame.len() < 2 {
            return Err(Error::InvalidFrame);
        }
        if frame[0] & 0xF0 != 0x10 {
            // Not a first frame
            return Err(Error::InvalidFrame);
        }
        let length = ((frame[0] as u16 & 0x0F) << 8) | frame[1] as u16;
        let data_length = cmp::min(frame.len() - 2, cmp::min(length as usize, 6));
        let mut data = [0; 6];
        data[..data_length].copy_from_slice(&frame[2..data_length+2]);
        Ok(FirstFrame {
            length,
            data,
        })
    }
}

pub struct ConsecutiveFrame {
    index: u8,
    data: [u8; 7],
    data_length: u8,
}

#[derive(Debug)]
pub struct Frame {
    data: [u8; 8],
}

pub trait IsotpInterface {
    /// Receives an ISO-TP packet
    fn recv(&self) -> Result<Vec<u8>>;

    /// Sends an ISO-TP packet
    fn send(&self, data: &[u8]) -> Result<()>;

    fn request(&self, request: &[u8]) -> Result<Vec<u8>> {
        self.send(&request)?;
        self.recv()
    }
    

}

fn duration_to_st(duration: time::Duration) -> u8 {
    if duration.subsec_micros() <= 900 && duration.subsec_micros() >= 100 {
        return (cmp::max(duration.subsec_micros() / 100, 1) + 0xF0) as u8;
    }
    duration.subsec_micros() as u8
}

impl Frame {
    fn new(data: [u8; 8]) -> Frame {
        Frame {data}
    }

    fn from_single_data(data: &[u8]) -> Frame {
        assert!(data.len() <= 7);

        let mut d = [0; 8];
        d[0] = data.len() as u8; // Single Frame id (0 << 4) | size
        d[1..=data.len()].copy_from_slice(&data);
        Frame {
            data: d,
        }
    }

    fn from_flow(flow: FlowControlFrame) -> Frame {
        let mut frame = Frame {data: [0; 8]};
        frame.data[0] = 0x30 | (flow.flag as u8);
        frame.data[1] = flow.block_size;
        frame.data[2] = duration_to_st(flow.separation_time);
        frame
    }

    fn from_first_data(data: &[u8], size: u16) -> Frame {
        assert!(data.len() <= 6);

        let mut d = [0; 8];
        d[0] = (0x10 | ((size & 0xF00) >> 8)) as u8;
        d[1] = (size & 0xFF) as u8;
        d[2..data.len() + 2].copy_from_slice(&data);
        Frame {
            data: d
        }
    }

    fn from_consec_data(data: &[u8], index: u8) -> Frame {
        assert!(data.len() <= 7);

        let mut d = [0; 8];
        d[0] = ((0x20) | (index & 0xF)) as u8;
        d[1..=data.len()].copy_from_slice(&data);
        Frame {
            data: d
        }
    }

    pub fn from_single(frame: &SingleFrame) -> Frame {
        Self::from_single_data(&frame.data[..frame.length as usize])
    }

    fn get_type(&self) -> Option<FrameType> {
        if self.data.is_empty() {
            return None;
        }

        let b_type = self.data[0] & 0xF0;

        match b_type {
            0 => Some(FrameType::Single),
            1 => Some(FrameType::First),
            2 => Some(FrameType::Consecutive),
            3 => Some(FrameType::Flow),
            _ => None,
        }
    }
}