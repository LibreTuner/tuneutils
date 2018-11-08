use std::rc::Rc;
use std::time;
use byteorder::{WriteBytesExt, ReadBytesExt, BigEndian};
use std::io::{Write, Read};
use crate::{
    protocols::can::{CanInterface, Message},
    error::{Error, Result};
};


impl From<j2534::Error> for Error {
    fn from(err: j2534::Error) -> Self {
        Error::J2534(err)
    }
}


pub struct J2534Can {
    channel: j2534::Channel,
}

impl J2534Can {
    /// Creates a new device from a J2534 channel. The channel must be a CAN channel.
    pub fn new(channel: j2534::Channel) -> Result<J2534Can> {
        Ok(J2534Can { channel })
    }

    /// Creates a CAN channel from a device with the specified baudrate
    pub fn connect(device: Rc<j2534::Device>, baudrate: u32) -> Result<J2534Can> {
        J2534Can::new(j2534::Channel::connect(device, j2534::Protocol::CAN, j2534::ConnectFlags::CAN_ID_BOTH, baudrate)?)
    }

    /// Applies a filter that wil allow all messages through
    pub fn apply_blank_filter(&self) -> Result<()> {
        let msg_mask = j2534::PassThruMsg {
            protocol_id: j2534::Protocol::CAN as u32,
            data_size: 5,
            // data[0..5] is already set to 0
            ..Default::default()
        };

        let msg_pattern = msg_mask;
        let _id = self.channel.start_msg_filter(j2534::FilterType::Pass, Some(&msg_mask), Some(&msg_pattern), None)?;
        Ok(())
    }
}

impl CanInterface for J2534Can {
    /// Sends a CAN message through the PassThru channel.
    /// 
    /// # Arguments
    /// 
    /// * `id` - The CAN id of the message
    /// * `message` - The message data. Must not be larger than 8 bytes
    fn send(&self, id: u32, message: &[u8]) -> Result<()> {
        if message.len() > 8 {
            return Err(Error::TooMuchData);
        }
        let data = {
            let mut d: [u8; 4128] = [0; 4128];
            {
                let mut writer: &mut [u8] = &mut d;
                writer.write_u32::<BigEndian>(id)?;
                writer.write(message)?;
            }
            d
        };
        let mut msg = [j2534::PassThruMsg::new_raw(j2534::Protocol::CAN, 0, 0, 0, message.len() as u32 + 4, 0, data)];

        // Use a timeout of 100ms
        let num_msgs = self.channel.write_msgs(&mut msg, 100)?;
        if num_msgs != 1 {
            return Err(Error::IncompleteWrite);
        }
        Ok(())
    }

    /// Received a single message from the PassThru channel.
    /// If no messages are received before the timeout, returns `Error::Timeout`
    /// 
    /// # Arguments
    /// 
    /// * `timeout` - The time to wait for a message before returning
    fn recv(&self, timeout: time::Duration) -> Result<Message> {
        let mut remaining = timeout;
        loop {
            let millis = (remaining.as_secs() * 1000 + remaining.subsec_millis() as u64) as u32;
            let msg = self.channel.read_msg(millis)?;
            if msg.data_size < 4 {
                continue;
            }
            let mut reader: &[u8] = &msg.data;
            let id = reader.read_u32::<BigEndian>()?;
            let mut buffer = vec![0; (msg.data_size - 4) as usize];
            let _amount = reader.read(&mut buffer)?;

            break Ok(Message {
                id,
                data: buffer,
            });
        }
    }
}