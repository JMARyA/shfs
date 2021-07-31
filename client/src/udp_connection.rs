use rich::*;
use shfs_api::calls::Call;
use shfs_api::responses::Response;
use shfs_networking::Connection;
use tokio::net::UdpSocket;
use tokio::runtime::Runtime;

/// Wrapper of [UdpSocket]
pub struct UDPConnection {
    addr: String,
    socket: UdpSocket,
    rt: Runtime,
    con: Connection,
}

impl UDPConnection {
    pub fn new(addr: &String) -> UDPConnection {
        let rt = Runtime::new().unwrap();
        let socket = unwrap_or_err(rt.block_on(UdpSocket::bind("0.0.0.0:0")), "");
        let _ = rt.block_on(socket.connect(addr));
        return UDPConnection {
            addr: addr.to_string(),
            socket,
            rt,
            con: Connection::new(),
        };
    }

    fn send(&mut self, msg: &Vec<u8>) -> Vec<u8> {
        let _ = self.rt.block_on(self.socket.send(msg));
        let resp = self.rt.block_on(self.con.recv(&self.socket));
        return resp;
    }

    /// Sending a [Call] to the Server returning [Response]
    pub fn send_call(&mut self, req: Call) -> Response {
        // TODO : Maybe Client Side Compression?
        let req = unwrap_or_err(serde_json::to_vec(&req), "Error serializing call");
        let resp = self.send(&req);
        let resp = Connection::remove_last_zeros(resp);
        let mut obj: Response =
            unwrap_or_err(serde_json::from_slice(&resp), "Error parsing response");
        // Decompression if Compression is applied
        obj = match obj {
            Response::Compressed { data } => {
                let resp = unwrap_or_err(
                    zstd::stream::decode_all(&data[0..data.len()]),
                    "Error decompressing response",
                );
                let resp = Connection::remove_last_zeros(resp);
                unwrap_or_err(serde_json::from_slice(&resp), "Error parsing response")
            }
            _ => obj,
        };
        return obj;
    }
}
