use std::time;
use std::default::Default;

pub mod error;
pub mod can;

use self::error::{Error, Result};

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

pub enum FCFlag {
    Continue,
    Wait,
    Overflow,
}

pub struct FlowControlFrame {
    flag: FCFlag,
    block_size: u8,
    separation_time: time::Duration,
}

pub struct SingleFrame {
    length: u8,
    data: [u8; 7],
}

impl SingleFrame {
    fn new(frame: &[u8]) -> Result<SingleFrame> {
        if (frame.len() == 0) {
            return Err(Error::InvalidFrame);
        }
        if frame[0] & 0xF0 != 0 {
            // Not a single frame
            return Err(Error::InvalidFrame);
        }
        let len = ops::min(7, frame[0] & 0x0F);
        let mut data = [0; 7];
        data[1..=len].copy_from_slice(&frame[1..]);
        Ok(SingleFrame {
            data,
            length: len,
        })
    }
}

pub struct FirstFrame {
    length: u16,
    data: [u8; 6],
}

impl FirstFrame {
    fn new(frame: [&u8]) -> Result<FirstFrame> {
        if frame.len() < 2 {
            return Err(Error::InvalidFrame);
        }
        if frame[0] & 0xF0 != 1 {
            // Not a first frame
            return Err(Error::InvalidFrame);
        }
        let length = ((frame[0] & 0x0F) << 8) | frame[1];
        let data_length = ops::min(frame.len(), ops::min(length, 6));
        Ok()
    }
}

pub struct ConsecutiveFrame {
    index: u8,
    data: [u8; 7],
    data_length: u8,
}

pub struct Frame {
    data: [u8; 8],
}

pub trait Interface {
    /// Receives an ISO-TP packet
    fn recv(&self) -> Result<Vec<u8>>;

    /// Sends an ISO-TP packet
    fn send(&self, data: &[u8]) -> Result<()>;

    // fn request(&self, request: &[u8]) -> Result<Vec<u8>>;
    

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
        if self.data.len() == 0 {
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