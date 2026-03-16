use std::io::{Read, Write};

use bincode::{Decode, Encode};
use crate::{IpcConnection, get_socket_name};

pub struct IpcClient {
    conn: IpcConnection,
}

impl IpcClient {
    pub fn connect() -> std::io::Result<Self> {
        let name = get_socket_name()?;
        let mut stream = interprocess::local_socket::ConnectOptions::new()
            .name(name.borrow())
            .connect_sync()?;

        // Handshake
        let handshake = [b'L', b'C', b'C', crate::IPC_PROTOCOL_VERSION[0], crate::IPC_PROTOCOL_VERSION[1]];
        stream.write_all(&handshake)?;

        let mut resp = [0u8; 5];
        stream.read_exact(&mut resp)?;

        if &resp[0..3] == b"ERR" {
            return Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                format!("Daemon rejected connection: Version mismatch! Daemon is v{}.{}, Client is v{}.{}.{}. Please update.",
                    resp[3], resp[4],
                    crate::IPC_PROTOCOL_VERSION[0], crate::IPC_PROTOCOL_VERSION[1], crate::IPC_PROTOCOL_VERSION[2]
                )
            ));
        } else if &resp[0..3] != b"OKK" {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid IPC handshake"));
        }

        Ok(Self {
            conn: IpcConnection { stream },
        })
    }

    pub fn request<Req: Encode, Res: Decode<()>>(&mut self, req: &Req) -> std::io::Result<Res> {
        self.conn.send(req)?;
        self.conn.recv()
    }
}
