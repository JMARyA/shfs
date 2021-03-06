//use fuse::{FileAttr, FileType};
use serde::{Deserialize, Serialize};
use time::Timespec;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Filesystem Types
pub enum FsFiletype {
    NamedPipe,
    CharDevice,
    BlockDevice,
    Directory,
    RegularFile,
    Symlink,
    Socket,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Filesystem Timespec
pub struct FsTimespec {
    sec: i64,
    nsec: i32,
}

impl FsTimespec {
    pub fn new(sec: i64, nsec: i32) -> FsTimespec {
        return FsTimespec { sec, nsec };
    }

    pub fn ts(&self) -> Timespec {
        return Timespec::new(self.sec, self.nsec);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Filesystem Object such as a Directory or a File
pub struct FilesystemEntry {
    /// Path of the entry
    pub path: String,
    /// Inode of the entry
    pub ino: u64,
    /// Size of the entry
    pub size: u64,
    /// Blocks of the entry
    pub blocks: u64,
    /// Access Time of the entry
    pub atime: FsTimespec,
    /// Modification Time of the entry
    pub mtime: FsTimespec,
    /// Change Time of the entry
    pub ctime: FsTimespec,
    /// Creation Time of the entry (macOS)
    pub crtime: FsTimespec,
    /// Permissions of the entry
    pub perm: u16,
    /// UID of the entry
    pub uid: u32,
    /// GID of the entry
    pub gid: u32,
    /// Kind of the entry
    pub kind: FsFiletype,
}
impl FilesystemEntry {
    pub fn new_file(
        path: String,
        ino: u64,
        size: u64,
        blocks: u64,
        perm: u16,
        uid: u32,
        gid: u32,
    ) -> FilesystemEntry {
        return FilesystemEntry {
            path: path,
            ino: ino,
            size: size,
            blocks: blocks,
            atime: FsTimespec::new(0, 0),
            mtime: FsTimespec::new(0, 0),
            ctime: FsTimespec::new(0, 0),
            crtime: FsTimespec::new(0, 0),
            perm: perm,
            uid: uid,
            gid: gid,
            kind: FsFiletype::RegularFile,
        };
    }

    pub fn new_directory(
        path: String,
        ino: u64,
        size: u64,
        blocks: u64,
        perm: u16,
        uid: u32,
        gid: u32,
    ) -> FilesystemEntry {
        return FilesystemEntry {
            path: path,
            ino: ino,
            size: size,
            blocks: blocks,
            atime: FsTimespec::new(0, 0),
            mtime: FsTimespec::new(0, 0),
            ctime: FsTimespec::new(0, 0),
            crtime: FsTimespec::new(0, 0),
            perm: perm,
            uid: uid,
            gid: gid,
            kind: FsFiletype::Directory,
        };
    }
}
