use std::fmt::{Debug, Formatter};
use std::io::{Error, ErrorKind, Read, Write};
use std::net::SocketAddr;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::control_messages::ControlResponse;
use crate::MessageEncoding;
use crate::rpc::ControlRpcMessage;

#[derive(Debug, Eq, PartialEq)]
pub enum ControlFeed {
    Response(ControlRpcMessage<ControlResponse>),
    NewClient(NewClient),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct NewClient {
    pub connect_addr: SocketAddr,
    pub peer_addr: SocketAddr,
    pub claim_instructions: ClaimInstructions,
    pub tunnel_server_id: u64,
    pub data_center_id: u32,
}

#[derive(Eq, PartialEq, Clone)]
pub struct ClaimInstructions {
    pub address: SocketAddr,
    pub token: Vec<u8>,
}

impl Debug for ClaimInstructions {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ClaimInstructions {{ address: {}, token: {} }}", self.address, hex::encode(&self.token))
    }
}

impl MessageEncoding for ControlFeed {
    fn write_to<T: Write>(&self, out: &mut T) -> std::io::Result<()> {
        match self {
            ControlFeed::Response(res) => {
                out.write_u32::<BigEndian>(1)?;
                res.write_to(out)
            }
            ControlFeed::NewClient(client) => {
                out.write_u32::<BigEndian>(2)?;
                client.write_to(out)
            }
        }
    }

    fn read_from<T: Read>(read: &mut T) -> std::io::Result<Self> {
        match read.read_u32::<BigEndian>()? {
            1 => Ok(ControlFeed::Response(ControlRpcMessage::read_from(read)?)),
            2 => Ok(ControlFeed::NewClient(NewClient::read_from(read)?)),
            _ => Err(Error::new(ErrorKind::Other, "invalid ControlFeed id")),
        }
    }
}

impl MessageEncoding for NewClient {
    fn write_to<T: Write>(&self, out: &mut T) -> std::io::Result<()> {
        self.connect_addr.write_to(out)?;
        self.peer_addr.write_to(out)?;
        self.claim_instructions.write_to(out)?;
        out.write_u64::<BigEndian>(self.tunnel_server_id)?;
        out.write_u32::<BigEndian>(self.data_center_id)
    }

    fn read_from<T: Read>(read: &mut T) -> std::io::Result<Self> {
        Ok(NewClient {
            connect_addr: SocketAddr::read_from(read)?,
            peer_addr: SocketAddr::read_from(read)?,
            claim_instructions: ClaimInstructions::read_from(read)?,
            tunnel_server_id: read.read_u64::<BigEndian>()?,
            data_center_id: read.read_u32::<BigEndian>()?,
        })
    }
}

impl MessageEncoding for ClaimInstructions {
    fn write_to<T: Write>(&self, out: &mut T) -> std::io::Result<()> {
        self.address.write_to(out)?;
        self.token.write_to(out)
    }

    fn read_from<T: Read>(read: &mut T) -> std::io::Result<Self> {
        Ok(ClaimInstructions {
            address: SocketAddr::read_from(read)?,
            token: Vec::read_from(read)?,
        })
    }
}