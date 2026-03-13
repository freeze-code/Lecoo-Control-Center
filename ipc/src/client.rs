use bincode::{Decode, Encode};
use crate::{IpcConnection, get_socket_name};

pub struct IpcClient {
    conn: IpcConnection,
}

impl IpcClient {
    pub fn connect() -> std::io::Result<Self> {
        let name = get_socket_name()?;
        let stream = interprocess::local_socket::ConnectOptions::new()
            .name(name.borrow())
            .connect_sync()?;

        Ok(Self {
            conn: IpcConnection { stream },
        })
    }

    pub fn request<Req: Encode, Res: Decode<()>>(&mut self, req: &Req) -> std::io::Result<Res> {
        self.conn.send(req)?;
        self.conn.recv()
    }
}
