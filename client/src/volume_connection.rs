use super::udp_connection::UDPConnection;
use shfs_api::calls::{Call, RequestInfo};
use shfs_api::filesystem_entry;
use shfs_api::responses::Response;
use std::str::from_utf8;

/// Connection to Volume
pub struct VolumeConnection {
    con: UDPConnection,
    info: RequestInfo,
    // Optional Volume Caching
    pub cache: Option<shfs_caching::Cache>,
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
            cache: Some(shfs_caching::Cache::new()),
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
        println!(
            "client write {} {} {}",
            ino,
            offset,
            from_utf8(&data).unwrap().to_string()
        );
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
