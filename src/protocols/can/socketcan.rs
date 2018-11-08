use std::io;
use std::mem;
use std::time;
use std::ffi;

use super::{CanInterface, Message};
use crate::error::{Error, Result};


const AF_CAN: libc::c_int = 29;
const PF_CAN: libc::c_int = 29;
const CAN_RAW: libc::c_int = 1;
const SOL_CAN_BASE: libc::c_int = 100;
const SOL_CAN_RAW: libc::c_int = SOL_CAN_BASE + CAN_RAW;
const CAN_RAW_FILTER: libc::c_int = 1;
const CAN_RAW_ERR_FILTER: libc::c_int = 2;
const CAN_RAW_LOOPBACK: libc::c_int = 3;
const CAN_RAW_RECV_OWN_MSGS: libc::c_int = 4;

#[repr(C)]
struct CanAddr {
    can_family: libc::c_short,
    if_index: libc::c_int, // address familiy,
    rx_id: u32,
    tx_id: u32,
}

pub struct SocketCan {
    fd: i32,
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct CanFrame {
    /// 32 bit CAN_ID + EFF/RTR/ERR flags
    can_id: u32,

    /// data length. Bytes beyond are not valid
    can_dlc: u8,

    /// padding
    _pad: u8,

    /// reserved
    _res0: u8,

    /// reserved
    _res1: u8,

    /// buffer for data
    data: [u8; 8],
}

impl Default for CanFrame {
    fn default() -> CanFrame {
        CanFrame {
            can_id: 0,
            can_dlc: 0,
            _pad: 0,
            _res0: 0,
            _res1: 0,
            data: [0; 8],
        }
    }
}

impl SocketCan {
    /// Opens a new SocketCan device
    /// 
    /// # Arguments
    /// 
    /// `ifname` - The name of the interface
    /// 
    /// # Example
    /// ```
    /// let interface = SocketCan::open("slcan0");
    /// ```
    pub fn open(ifname: &str) -> io::Result<SocketCan> {
        let c_string = ffi::CString::new(ifname).unwrap();
        let if_index = unsafe { libc::if_nametoindex(c_string.as_ptr()) };
        if if_index == 0 {
            return Err(io::Error::last_os_error());
        }
        SocketCan::open_idx(if_index)
    }

    pub fn open_idx(if_index: libc::c_uint) -> io::Result<SocketCan> {
        let sock_fd;
        unsafe {
            sock_fd = libc::socket(PF_CAN, libc::SOCK_RAW, CAN_RAW);
        }

        if sock_fd == -1 {
            return Err(io::Error::last_os_error());
        }

        let addr = CanAddr {
            if_index: if_index as libc::c_int,
            can_family: AF_CAN as libc::c_short,
            rx_id: 0,
            tx_id: 0,
        };

        // Bind
        let bind_res;
        unsafe {
            let sockaddr_ptr = &addr as *const CanAddr;
            bind_res = libc::bind(sock_fd,
                sockaddr_ptr as *const libc::sockaddr,
                mem::size_of::<CanAddr>() as u32);
        }
        
        if bind_res == -1 {
            let err = io::Error::last_os_error();
            unsafe { libc::close(sock_fd); }
            return Err(err);
        }

        Ok(SocketCan {
            fd: sock_fd,
        })
    }
}

impl Drop for SocketCan {
    fn drop(&mut self) {
        unsafe { libc::close(self.fd) };
    }
}

impl CanInterface for SocketCan {
    fn send(&self, id: u32, message: &[u8]) -> Result<()> {
        if message.len() > 8 {
            return Err(Error::TooMuchData);
        }

        let mut frame = CanFrame::default();
        frame.can_dlc = message.len() as u8;
        frame.can_id = id;
        frame.data[..message.len()].clone_from_slice(message);

        let frame_ptr = &frame as *const CanFrame;

        let res = unsafe { libc::write(self.fd, frame_ptr as *const libc::c_void, mem::size_of::<CanFrame>()) };
        if res != mem::size_of::<CanFrame>() as isize {
            if res < 0 {
                return Err(Error::Io(io::Error::last_os_error()));
            }
            return Err(Error::IncompleteWrite);
        }
        Ok(())
    }

    // FIXME: implement timeout
    fn recv(&self, timeout: time::Duration) -> Result<Message> {
        let mut frame = CanFrame::default();
        let frame_ptr = &mut frame as *mut CanFrame;
        
        let res = unsafe { libc::recv(self.fd, frame_ptr as *mut libc::c_void, mem::size_of::<CanFrame>(), 0) };
        if res < 0 {
            return Err(Error::Io(io::Error::last_os_error()));
        }
        Ok(Message {
            id: frame.can_id,
            data: frame.data[..(frame.can_dlc as usize)].to_vec(),
        })
    }
}