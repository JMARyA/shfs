use path_absolutize::*;
use rich::unwrap_or_err;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};
#[cfg(target_os = "linux")]
use std::os::linux::fs::MetadataExt;
#[cfg(target_os = "macos")]
use std::os::macos::fs::MetadataExt;
use std::path::Path;

pub mod filesystem_entry;
pub mod calls;
pub mod config;
pub mod responses;
pub mod volume;

#[derive(Debug)]
pub struct ApiError {
    err: String,
}

impl ApiError {
    pub fn new(msg: &str) -> ApiError {
        return ApiError {
            err: msg.to_string(),
        };
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.err)
    }
}

impl Error for ApiError {}

/// Server Side API handling Volume Requests
pub struct FilesystemAPI {
    pub root: String,
    inode_cache: HashMap<u64, filesystem_entry::FilesystemEntry>,
}

impl FilesystemAPI {
    pub fn new(root: String) -> FilesystemAPI {
        let mut api = FilesystemAPI {
            root,
            inode_cache: HashMap::new(),
        };
        let mut ret = unwrap_or_err(api.get_entry("/"), "Can not get root dir on server");
        ret.ino = 1;
        api.inode_cache.insert(1, ret.clone());

        return api;
    }

    /// Combine path with the root path of the Volume
    fn join_root_path(&self, path: &str) -> Result<String, &str> {
        let rpath = Path::new(&self.root);
        let mut newpath = String::new();
        for e in rpath.components() {
            let comp_str = e.as_os_str().to_str().expect("");
            newpath.push_str(comp_str);
            if comp_str != "/" {
                newpath.push_str("/");
            }
        }
        for e in Path::new(path).components() {
            let comp_str = e.as_os_str().to_str().expect("");
            newpath.push_str(comp_str);
            if comp_str != "/" {
                newpath.push_str("/");
            }
        }
        newpath = newpath.replace("//", "/");
        if newpath.ends_with("/") {
            newpath.pop();
        }
        let p = Path::new(&newpath);
        newpath = String::from(p.absolutize().unwrap().to_str().unwrap());
        if !Path::new(&newpath).starts_with(Path::new(&self.root)) {
            return Err("Root Escalation");
        }
        return Ok(newpath);
    }

    pub fn readdir(&self, path: &str) -> Vec<String> {
        let mut ret = vec![];
        let rpath = self.join_root_path(path);
        if rpath.is_err() {
            return ret;
        }
        let rpath = rpath.unwrap();

        // TODO : Maybe error handling?
        let entries = std::fs::read_dir(rpath);
        if entries.is_err() {
            return ret;
        }
        let entries = entries.unwrap();
        for entry in entries {
            let entry = entry.expect("Error getting entry");
            let path = entry.path();
            let pathstr = path.to_str().expect("Error converting to str");
            ret.push(String::from(pathstr.replace(&self.root, "")));
        }
        return ret;
    }

    pub fn get_path_from_inode(&self, ino: u64) -> Result<String, std::io::Error> {
        let parent_ino = self.get_entry_from_inode(ino);
        if parent_ino.is_err() {
            return Err(parent_ino.unwrap_err());
        }
        return Ok(parent_ino.unwrap().path);
    }

    pub fn get_entry_from_inode(
        &self,
        ino: u64,
    ) -> Result<filesystem_entry::FilesystemEntry, std::io::Error> {
        let res = self.inode_cache.get(&ino);
        if res.is_some() {
            return Ok(res.unwrap().clone());
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Not Found",
            ));
        }
    }

    pub fn get_entry(
        &mut self,
        path: &str,
    ) -> Result<filesystem_entry::FilesystemEntry, std::io::Error> {
        println!("Getting entry {}", path);
        let rpath = self.join_root_path(path);
        if rpath.is_err() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "",
            ));
        }
        let rpath = rpath.unwrap();
        let m = fs::metadata(&rpath);
        if m.is_err() {
            return Err(m.unwrap_err());
        }
        let m = m.unwrap();
        let mut ret = filesystem_entry::FilesystemEntry::new_file(
            String::from(path),
            m.st_ino(),
            m.st_size(),
            m.st_blocks(),
            755,
            m.st_gid(),
            m.st_uid(),
        );
        if m.is_dir() {
            ret = filesystem_entry::FilesystemEntry::new_directory(
                String::from(path),
                m.st_ino(),
                m.st_size(),
                m.st_blocks(),
                755,
                m.st_gid(),
                m.st_uid(),
            );
        }
        self.inode_cache.insert(m.st_ino(), ret.clone());
        return Ok(ret);
    }

    pub fn read(&self, ino: u64, offset: i64, size: u32) -> Result<Vec<u8>, std::io::Error> {
        let mut chunk = vec![0; size as usize];
        let file = self.get_entry_from_inode(ino).unwrap();

        let path = self.join_root_path(&file.path);
        if path.is_err() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "",
            ));
        }
        let path = path.unwrap();
        let mut fh = std::fs::File::open(&path).expect("File Opening failed");

        let _ = fh.seek(SeekFrom::Start(offset as u64));
        let _ = fh.read_exact(&mut chunk);
        /*
        unwrap_or_err(fh.seek(SeekFrom::Start(offset as u64)), "Error while seeking file");
        fh.read_exact(&mut chunk).expect("File could not be read");
*/
        return Ok(chunk);
    }

    pub fn rename(
        &mut self,
        parent: u64,
        name: &str,
        nparent: u64,
        nname: &str,
    ) -> Result<(), std::io::Error> {
        let parent_path = self.get_path_from_inode(parent).unwrap();
        let file_path = Path::new(&parent_path).join(name);

        let nparent_path = self.get_path_from_inode(nparent).unwrap();
        let nfile_path = Path::new(&nparent_path).join(nname);

        let err = self.get_entry(nfile_path.to_str().unwrap());
        if err.is_err() {
            let err = err.unwrap_err();
            if err.kind() == std::io::ErrorKind::AlreadyExists {
                return Err(err);
            }
        }

        let rpath = self.join_root_path(file_path.to_str().unwrap());
        let nrpath = self.join_root_path(nfile_path.to_str().unwrap());

        if rpath.is_err() || nrpath.is_err() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "",
            ));
        }
        let rpath = rpath.unwrap();
        let nrpath = nrpath.unwrap();

        println!("rename {} -> {}", &rpath, &nrpath);
        let err = std::fs::rename(rpath, nrpath);
        if err.is_err() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "",
            ));
        } else {
            return Ok(());
        }
    }

    pub fn mkdir(
        &mut self,
        parent: u64,
        name: &str,
    ) -> Result<filesystem_entry::FilesystemEntry, std::io::Error> {
        let parent_path = self.get_path_from_inode(parent).unwrap();
        let file_path = Path::new(&parent_path).join(name);
        let rpath = self.join_root_path(&file_path.to_str().expect(""));
        if rpath.is_err() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "",
            ));
        }
        let rpath = rpath.unwrap();
        let err = std::fs::create_dir(rpath);
        if err.is_err() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "",
            ));
        } else {
            let ret = self.get_entry(file_path.to_str().expect("")).expect("");
            return Ok(ret);
        }
    }

    pub fn create(
        &mut self,
        parent: u64,
        name: &str,
    ) -> Result<filesystem_entry::FilesystemEntry, std::io::Error> {
        let parent_path = self.get_path_from_inode(parent).unwrap();
        let file_path = Path::new(&parent_path).join(name);
        let rpath = self.join_root_path(&file_path.to_str().expect(""));
        if rpath.is_err() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "",
            ));
        }
        let rpath = rpath.unwrap();
        let err = std::fs::File::create(rpath);
        if err.is_err() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "",
            ));
        } else {
            let ret = self.get_entry(file_path.to_str().expect("")).expect("");
            return Ok(ret);
        }
    }

    pub fn write(&self, ino: u64, offset: i64, data: &[u8]) -> Result<u32, std::io::Error> {
        let file_path = self.get_path_from_inode(ino).unwrap();
        let rpath = self.join_root_path(&file_path);
        if rpath.is_err() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "",
            ));
        }
        let rpath = rpath.unwrap();
        let mut file = std::fs::File::open(&rpath).unwrap();
        unwrap_or_err(file.seek(SeekFrom::Start(offset as u64)), "Error while seeking file");
        let written = std::fs::write(&rpath, data);
        // TODO : Fix Writes
        //let written = file.write(data);
        if written.is_err() {
            let err = written.unwrap_err();
            return Err(err);
        } else {
            return Ok(0);
        }
    }

    pub fn unlink(&self, parent: u64, name: &str) -> Result<(), std::io::Error> {
        let parent_path = self.get_path_from_inode(parent).unwrap();
        let file_path = Path::new(&parent_path).join(name);
        let rpath = self.join_root_path(&file_path.to_str().expect(""));
        if rpath.is_err() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "",
            ));
        }
        let rpath = rpath.unwrap();
        let err = std::fs::remove_file(rpath);
        if err.is_err() {
            return err;
        } else {
            return Ok(());
        }
    }

    pub fn rmdir(&self, parent: u64, name: &str) -> Result<(), std::io::Error> {
        let parent_path = self.get_path_from_inode(parent).unwrap();
        let file_path = Path::new(&parent_path).join(name);
        let rpath = self.join_root_path(&file_path.to_str().expect(""));
        if rpath.is_err() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "",
            ));
        }
        let rpath = rpath.unwrap();
        println!("trying to remove {}", rpath);
        return std::fs::remove_dir(rpath);
    }
}
