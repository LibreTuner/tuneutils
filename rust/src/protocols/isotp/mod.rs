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
    Continue = 0,
    Wait = 1,
    Overflow = 2,
}

pub struct FlowControlFrame {
    flag: FCFlag,
    block_size: u8,
    separation_time: time::Duration,
}

pub struct SingleFrame {
    size: u8,
    data: [u8; 7],
}

pub struct FirstFrame {
    size: u16,
    data: [u8; 6],
    data_length: u8,
}

pub struct ConsecutiveFrame {
    index: u8,
    data: [u8; 7],
    data_length: u8,
}

pub struct Frame {
    data: [u8; 8],
    length: u8,
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
        Frame {data, length: 8}
    }

    fn from_single(data: &[u8]) -> Frame {
        assert!(data.len() <= 7);

        let d = [0; 8];
        d[..data.len()].copy_from_slice(&data);
        Frame {
            data: d,
            length: data.len() as u8
        }
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