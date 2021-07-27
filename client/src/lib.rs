use rich::*;
use shfs_api::calls::{RequestInfo, Call};
use shfs_api::responses::Response;
use shfs_api::{filesystem_entry};
use shfs_caching;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};
use tokio::runtime::Runtime;
use std::str::from_utf8;

/// Removing unnecessary zeros at the end of [Vec]
fn remove_last_zeros(d: Vec<u8>) -> Vec<u8> {
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

/// Wrapper of [UdpSocket]
pub struct UDPConnection {
    addr: String,
    socket: UdpSocket,
    rt: Runtime,
}

impl UDPConnection {
    pub fn new(addr: &String) -> UDPConnection {
        let rt = Runtime::new().unwrap();
        let socket = unwrap_or_err(rt.block_on(UdpSocket::bind("0.0.0.0:0")), "");
        rt.block_on(socket.connect(addr));
        return UDPConnection {
            addr: addr.to_string(),
            socket,
            rt,
        };
    }

    async fn send(socket: &mut TcpStream, msg: &Vec<u8>) -> Result<Vec<u8>, std::io::Error> {
        unwrap_or_err(socket.write_all(msg).await, "Error sending request");

        let mut final_buf: Vec<u8> = Vec::with_capacity(1024);

        loop {
            let mut buf = [0; 1024];
            let n = socket.read(&mut buf).await;

            if n.is_err() {
                return Err(n.unwrap_err());
            }
            let n = n.unwrap();
            if n == 0 {
                break;
            }
            final_buf.extend_from_slice(&buf[0..n]);
        }
        //let buf_str = from_utf8(&buf).unwrap();

        return Ok(final_buf);
    }

    fn send_with_reconnect(&mut self, msg: &Vec<u8>) -> Vec<u8> {
        self.rt.block_on(self.socket.send(msg));
        let mut resp = vec![0; 524288];
        self.rt.block_on(self.socket.recv(&mut resp));
        if (String::from_utf8(resp.clone()).unwrap().starts_with("PACK")) {
            let mut sum = vec![];
            let s = String::from_utf8(remove_last_zeros(resp.clone())).unwrap();
            let parts = s[4..s.len()].to_string().parse().unwrap();
            for i in 0..parts {
                let mut resp = vec![0; 524288];
                self.rt.block_on(self.socket.recv(&mut resp));
                sum.append(&mut remove_last_zeros(resp));
            }
            println!("received {} bytes", sum.len());
            return sum;
        }
        return resp;
    }

    /// Sending a [Call] to the Server returning [Response]
    pub fn send_call(&mut self, req: Call) -> Response {
        let req = unwrap_or_err(serde_json::to_vec(&req), "Error serializing call");
        let resp = self.send_with_reconnect(&req);
        let resp = remove_last_zeros(resp);
        let mut obj: Response =
            unwrap_or_err(serde_json::from_slice(&resp), "Error parsing response");
        // Decompression if Compression is applied
        obj = match obj {
            Response::Compressed { data } => {
                let resp = unwrap_or_err(zstd::stream::decode_all(&data[0..data.len()]), "Error decompressing response");
                let resp = remove_last_zeros(resp);
                unwrap_or_err(serde_json::from_slice(&resp), "Error parsing response")
            }
            _ => obj,
        };
        //println!("{:?}", obj); // TODO : Optional verbosity
        return obj;
    }
}

/// General Connection to Server
pub struct ServerConnection {
    con: UDPConnection,
}

impl ServerConnection {
    pub fn new(addr: &String) -> ServerConnection {
        return ServerConnection {
            con: UDPConnection::new(addr),
        };
    }

    /// Lookup the ID of Volume
    pub fn lookup_volume(&mut self, name: &str) -> Result<u64, shfs_api::ApiError> {
        let req = Call::VolumeLookup {
            name: name.to_string(),
        };

        let obj = self.con.send_call(req);

        let ret = match obj {
            Response::VolumeLookup { id } => Ok(id),
            Response::Error { error } => Err(shfs_api::ApiError::new(&error)),
            _ => Err(shfs_api::ApiError::new(&String::new())),
        };

        return ret;
    }

    /// Get Server Info
    pub fn server_info(&mut self) -> Result<Response, shfs_api::ApiError> {
        let req = Call::ServerInfo {};
        let obj = self.con.send_call(req);
        let ret = match obj {
            Response::ServerInfo { .. } => Ok(obj),
            Response::Error { error } => Err(shfs_api::ApiError::new(&error)),
            _ => Err(shfs_api::ApiError::new(&String::new())),
        };

        return ret;
    }

    /// Get the List of Volumes
    pub fn list_volumes(&mut self) -> Result<Vec<String>, shfs_api::ApiError> {
        let req = Call::ListVolumes {};
        let obj = self.con.send_call(req);
        let ret = match obj {
            Response::ListVolumes { data } => Ok(data),
            Response::Error { error } => Err(shfs_api::ApiError::new(&error)),
            _ => Err(shfs_api::ApiError::new("")),
        };

        return ret;
    }
}


/// Connection to Volume
pub struct VolumeConnection {
    con: UDPConnection,
    info: RequestInfo,
    // Optional Volume Caching
    pub cache: Option<shfs_caching::Cache>
}

impl VolumeConnection {
    /// Creates a new [VolumeConnection]
    /// # Arguments
    /// * `addr` - Address of the Server: IP:PORT
    /// * `vol_id` - ID of Volume
    pub fn new(addr: &String, vol_id: u64) -> VolumeConnection {
        return VolumeConnection {
            con: UDPConnection::new(addr),
            info: RequestInfo { volume_id: vol_id },
            cache: Some(shfs_caching::Cache::new())
        };
    }

    pub fn readdir(&mut self, path: &str) -> Vec<String> {
        let req = Call::ReadDir {
            info: self.info.clone(),
            path: path.to_string(),
        };
        let obj = self.con.send_call(req);
        let ret = match obj {
            Response::ReadDir { data } => data,
            _ => vec![],
        };
        return ret;
    }

    pub fn read(&mut self, ino: u64, offset: i64, size: u32) -> Result<Vec<u8>, std::io::Error> {
        let req = Call::Read {
            info: self.info.clone(),
            ino,
            offset,
            size,
        };
        let obj = self.con.send_call(req);
        let ret = match obj {
            Response::Read { data } => Ok(data),
            Response::IOError { error } => Err(std::io::Error::from_raw_os_error(error)),
            _ => Ok(vec![]),
        };
        return ret;
    }

    pub fn rename(
        &mut self,
        parent: u64,
        name: &str,
        nparent: u64,
        nname: &str,
    ) -> Result<(), std::io::Error> {
        let req = Call::Rename {
            info: self.info.clone(),
            parent,
            name: name.to_string(),
            nparent,
            nname: nname.to_string(),
        };
        let obj = self.con.send_call(req);
        let ret = match obj {
            Response::Ok {} => Ok(()),
            Response::IOError { error } => Err(std::io::Error::from_raw_os_error(error)),
            _ => Ok(()),
        };
        return ret;
    }

    pub fn mkdir(
        &mut self,
        parent: u64,
        name: &str,
    ) -> Result<filesystem_entry::FilesystemEntry, std::io::Error> {
        let req = Call::Mkdir {
            info: self.info.clone(),
            parent,
            name: name.to_string(),
        };
        let obj = self.con.send_call(req);
        let ret = match obj {
            Response::GetEntry { data } => Ok(data),
            Response::IOError { error } => Err(std::io::Error::from_raw_os_error(error)),
            _ => Ok(filesystem_entry::FilesystemEntry::new_directory(
                String::from(""),
                0,
                0,
                0,
                755,
                0,
                0,
            )),
        };
        return ret;
    }

    pub fn create(
        &mut self,
        parent: u64,
        name: &str,
    ) -> Result<filesystem_entry::FilesystemEntry, std::io::Error> {
        let req = Call::Create {
            info: self.info.clone(),
            parent,
            name: name.to_string(),
        };
        let obj = self.con.send_call(req);
        let ret = match obj {
            Response::GetEntry { data } => Ok(data),
            Response::IOError { error } => Err(std::io::Error::from_raw_os_error(error)),
            _ => Err(std::io::Error::from_raw_os_error(0)),
        };
        return ret;
    }

    pub fn write(&mut self, ino: u64, offset: i64, data: &[u8]) -> Result<u32, std::io::Error> {
        println!("client write {} {} {}", ino, offset, from_utf8(&data).unwrap().to_string());
        let req = Call::Write {
            info: self.info.clone(),
            ino,
            offset,
            data: data.to_vec(),
        };
        let obj = self.con.send_call(req);
        let ret = match obj {
            Response::Write { data } => Ok(data),
            Response::IOError { error } => Err(std::io::Error::from_raw_os_error(error)),
            _ => Err(std::io::Error::from_raw_os_error(0)),
        };
        return ret;
    }

    pub fn unlink(&mut self, parent: u64, name: &str) -> Result<(), std::io::Error> {
        let req = Call::Unlink {
            info: self.info.clone(),
            parent,
            name: name.to_string(),
        };
        let obj = self.con.send_call(req);
        let ret = match obj {
            Response::Ok {} => Ok(()),
            Response::IOError { error } => Err(std::io::Error::from_raw_os_error(error)),
            _ => Ok(()),
        };
        return ret;
    }

    pub fn rmdir(&mut self, parent: u64, name: &str) -> Result<(), std::io::Error> {
        let req = Call::Rmdir {
            info: self.info.clone(),
            parent,
            name: name.to_string(),
        };
        let obj = self.con.send_call(req);
        let ret = match obj {
            Response::Ok {} => Ok(()),
            Response::IOError { error } => Err(std::io::Error::from_raw_os_error(error)),
            _ => Ok(()),
        };
        return ret;
    }

    pub fn get_entry(
        &mut self,
        path: &str,
    ) -> Result<filesystem_entry::FilesystemEntry, std::io::Error> {
        // Using cached version if it exists and is enabled
        if self.cache.is_some() {
            let entry = self.cache.as_ref().unwrap().get_entry(path);
            if entry.is_some() {
                return Ok(entry.unwrap().clone());
            }
        }
        // Calling if nothing is found
        let req = Call::GetEntry {
            info: self.info.clone(),
            path: path.to_string(),
        };
        let obj = self.con.send_call(req);
        let ret = match obj {
            Response::GetEntry { data } => Ok(data),
            Response::IOError { error } => Err(std::io::Error::from_raw_os_error(error)),
            _ => Ok(filesystem_entry::FilesystemEntry::new_directory(
                String::from(""),
                0,
                0,
                0,
                755,
                0,
                0,
            )),
        };
        // Adding entry to cache
        if ret.is_ok() {
            let ret = ret.unwrap();
            if self.cache.is_some() {
                self.cache.as_mut().unwrap().add_entry(&ret.clone());
            }
            return Ok(ret);
        } else {
            return ret;
        }
    }

    pub fn get_entry_from_inode(
        &mut self,
        ino: u64,
    ) -> Result<filesystem_entry::FilesystemEntry, std::io::Error> {
        let req = Call::GetEntryFromInode {
            info: self.info.clone(),
            ino: ino,
        };
        let obj = self.con.send_call(req);
        let ret = match obj {
            Response::GetEntry { data } => Ok(data),
            Response::IOError { error } => Err(std::io::Error::from_raw_os_error(error)),
            _ => Ok(filesystem_entry::FilesystemEntry::new_directory(
                String::from(""),
                0,
                0,
                0,
                755,
                0,
                0,
            )),
        };
        return ret;
    }

    pub fn get_path_from_inode(&mut self, ino: u64) -> Result<String, std::io::Error> {
        let req = Call::GetPathFromInode {
            info: self.info.clone(),
            ino: ino,
        };
        let obj = self.con.send_call(req);
        let ret = match obj {
            Response::GetPath { data } => Ok(data),
            Response::IOError { error } => Err(std::io::Error::from_raw_os_error(error)),
            _ => Ok(String::new()),
        };
        return ret;
    }
}
