use tokio::net::UdpSocket;

pub struct Connection {}

impl Connection {
    pub fn new() -> Connection {
        return Connection {};
    }

    /// Removing unnecessary zeros at the end of [Vec]
    pub fn remove_last_zeros(d: Vec<u8>) -> Vec<u8> {
        let mut reversed = d.clone();
        reversed.reverse();
        let mut c = 0;
        for e in reversed.iter() {
            if *e as isize != 0 {
                break;
            }
            c += 1;
        }
        let content = d.len() - c;
        return d[0..content].to_vec();
    }

    pub async fn send(&self, resp: Vec<u8>, socket: &UdpSocket, addr: String) -> usize {
        let pack_size = 8024;

        if (resp.len() / pack_size) > 1 {
            // Split Package if too large
            let mut lensum = socket
                .send_to(
                    &format!("PACK{}", (resp.len() / pack_size)).into_bytes(),
                    &addr,
                )
                .await
                .unwrap();
            for i in 0..(resp.len() / pack_size) {
                let mut end = (i * pack_size) + pack_size;
                if i + 1 == (resp.len() / pack_size) {
                    end = resp.len();
                }
                let len = socket
                    .send_to(&resp[(i * pack_size)..end], &addr)
                    .await
                    .unwrap();
                lensum += len;
            }
            return lensum;
        } else {
            // Else send complete
            return socket.send_to(&resp[..resp.len()], addr).await.unwrap();
        }
    }

    pub async fn recv(&self, socket: &UdpSocket) -> Vec<u8> {
        let mut resp = vec![0; 524288];
        socket.recv(&mut resp).await;
        if String::from_utf8(resp.clone()).unwrap().starts_with("PACK") {
            let mut sum = vec![];
            let s = String::from_utf8(Connection::remove_last_zeros(resp.clone())).unwrap();
            let parts = s[4..s.len()].to_string().parse().unwrap();
            for _ in 0..parts {
                let mut resp = vec![0; 524288];
                socket.recv(&mut resp).await;
                sum.append(&mut Connection::remove_last_zeros(resp));
            }
            return sum;
        }
        return resp;
    }
}
