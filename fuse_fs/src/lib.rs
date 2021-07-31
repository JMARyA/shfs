use fuse;
#[cfg(target_os = "macos")]
use fuse::ReplyXTimes;
use fuse::{
    FileAttr, FileType, ReplyAttr, ReplyBmap, ReplyCreate, ReplyData, ReplyDirectory, ReplyEmpty,
    ReplyEntry, ReplyLock, ReplyOpen, ReplyStatfs, ReplyWrite, ReplyXattr, Request,
};
use shfs_client::VolumeConnection;
use std::ffi::OsStr;
use std::path::Path;
use time::Timespec;

use shfs_api::filesystem_entry::{FilesystemEntry, FsFiletype};

/// Helper Function to convert [FsFiletype] of the API to FUSE [FileType]
pub fn to_filetype(t: &FsFiletype) -> FileType {
    let kind = match t {
        FsFiletype::Directory => FileType::Directory,
        FsFiletype::RegularFile => FileType::RegularFile,
        _ => FileType::RegularFile,
    };
    return kind;
}

/// Helper Function to convert [FilesystemEntry] of the API to FUSE [FileAttr]
pub fn attr(t: &FilesystemEntry) -> FileAttr {
    return fuse::FileAttr {
        ino: t.ino,
        size: t.size,
        blocks: t.blocks,
        atime: t.atime.ts(),
        mtime: t.mtime.ts(),
        ctime: t.ctime.ts(),
        crtime: t.crtime.ts(),
        kind: to_filetype(&t.kind),
        perm: t.perm,
        nlink: 0,
        uid: t.uid,
        gid: t.gid,
        rdev: 0,
        flags: 0,
    };
}

/// FUSE Filesystem for ShFS
pub struct Filesystem {
    pub api: VolumeConnection,
}

impl fuse::Filesystem for Filesystem {
    fn init(&mut self, _req: &Request) -> Result<(), i32> {
        //self.log.printInfo("Filesystem Initialized");
        return Ok(());
    }

    fn destroy(&mut self, _req: &Request) {
        //self.log.printInfo("Filesystem destroyed");
    }

    #[cfg(target_os = "macos")]
    fn setvolname(&mut self, _req: &Request, _name: &OsStr, reply: ReplyEmpty) {
        //self.log.printInfo("Filesystem SetVolName");
        reply.ok();
    }
    fn access(&mut self, _req: &Request, _ino: u64, _mask: u32, reply: ReplyEmpty) {
        //self.log.printAttrInfo("Filesystem Access");
        reply.ok();
    }

    // Attrs

    fn getattr(&mut self, _req: &Request, _ino: u64, reply: ReplyAttr) {
        //self.log.printAttrInfo("Filesystem GetAttr");

        let entry = self.api.get_entry_from_inode(_ino);
        if entry.is_ok() {
            return reply.attr(&Timespec::new(0, 0), &attr(&entry.unwrap()));
        }
        return reply.error(2);
    }

    // TODO : Implement setattr
    fn setattr(
        &mut self,
        _req: &Request,
        _ino: u64,
        _mode: Option<u32>,
        _uid: Option<u32>,
        _gid: Option<u32>,
        _size: Option<u64>,
        _atime: Option<Timespec>,
        _mtime: Option<Timespec>,
        _fh: Option<u64>,
        _crtime: Option<Timespec>,
        _chgtime: Option<Timespec>,
        _bkuptime: Option<Timespec>,
        _flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        reply.attr(
            &Timespec::new(0, 0),
            &attr(&FilesystemEntry::new_directory(
                String::from(""),
                0,
                0,
                0,
                755,
                501,
                20,
            )),
        );
    }

    // TODO : Implement setxattr
    fn setxattr(
        &mut self,
        _req: &Request,
        _ino: u64,
        _name: &OsStr,
        _value: &[u8],
        _flags: u32,
        _position: u32,
        reply: ReplyEmpty,
    ) {
        //self.log.printAttrInfo("Filesystem SetXAttr");
        reply.ok();
    }

    // TODO : Implement getxattr
    fn getxattr(
        &mut self,
        _req: &Request,
        _ino: u64,
        _name: &OsStr,
        _size: u32,
        reply: ReplyXattr,
    ) {
        //self.log.printAttrInfo("Filesystem GetXattr");
        reply.data(&[]);
    }

    // TODO : Implement listxattr
    fn listxattr(&mut self, _req: &Request, _ino: u64, _size: u32, reply: ReplyXattr) {
        reply.data(&[]);
    }

    // TODO : Implement removexattr
    fn removexattr(&mut self, _req: &Request, _ino: u64, _name: &OsStr, reply: ReplyEmpty) {
        //self.log.printAttrInfo("Filesystem RemoveXAttr");
        reply.ok();
    }

    // Actions

    fn lookup(&mut self, _req: &Request, _parent: u64, _name: &OsStr, reply: ReplyEntry) {
        /*self.log.printInfo(&format!(
            "Filesystem Lookup parent {} name {}",
            _parent,
            _name.to_str().unwrap()
        ));*/
        let parent_path = self.api.get_path_from_inode(_parent).expect("");
        let name = _name.to_str().unwrap();
        let file_path = Path::new(&parent_path).join(name);
        let file = self.api.get_entry(&file_path.to_str().unwrap());
        if file.is_err() {
            let err = file.unwrap_err();
            if err.kind() == std::io::ErrorKind::NotFound {
                reply.error(2);
            } else {
                reply.error(0);
            }
        } else {
            let file = file.unwrap();
            reply.entry(&Timespec::new(0, 0), &attr(&file), 0);
        }
    }

    fn forget(&mut self, _req: &Request, _ino: u64, _nlookup: u64) {
        //self.log.printInfo("Filesystem Forget");
    }

    // TODO : Implement readlink
    fn readlink(&mut self, _req: &Request, _ino: u64, reply: ReplyData) {
        reply.error(0);
    }

    // TODO : Implement mknod
    fn mknod(
        &mut self,
        _req: &Request,
        _parent: u64,
        _name: &OsStr,
        _mode: u32,
        _rdev: u32,
        reply: ReplyEntry,
    ) {
        reply.error(0);
    }

    fn unlink(&mut self, _req: &Request, _parent: u64, _name: &OsStr, reply: ReplyEmpty) {
        /*self.log
        .printAction(&format!("Filesystem Unlink {}", _name.to_str().unwrap()));*/
        let err = self.api.unlink(_parent, _name.to_str().unwrap());
        if err.is_err() {
            let err = err.unwrap_err();
            reply.error(err.raw_os_error().unwrap());
        } else {
            reply.ok();
        }
    }

    // TODO : Implement symlink
    fn symlink(
        &mut self,
        _req: &Request,
        _parent: u64,
        _name: &OsStr,
        _link: &Path,
        reply: ReplyEntry,
    ) {
        reply.error(0);
    }

    fn rename(
        &mut self,
        _req: &Request,
        _parent: u64,
        _name: &OsStr,
        _newparent: u64,
        _newname: &OsStr,
        reply: ReplyEmpty,
    ) {
        /*self.log.printAction(&format!(
            "Filesystem Rename {} -> {}",
            _name.to_str().unwrap(),
            _newname.to_str().unwrap()
        ));*/
        let err = self.api.rename(
            _parent,
            _name.to_str().unwrap(),
            _newparent,
            _newname.to_str().unwrap(),
        );
        if err.is_err() {
            reply.error(err.unwrap_err().raw_os_error().unwrap());
        } else {
            reply.ok();
        }
    }
    // Directories
    fn mkdir(
        &mut self,
        _req: &Request,
        _parent: u64,
        _name: &OsStr,
        _mode: u32,
        reply: ReplyEntry,
    ) {
        /*self.log
        .printAction(&format!("Filesystem MkDir {}", _name.to_str().unwrap()));*/
        let dir = self.api.mkdir(_parent, _name.to_str().expect(""));
        if dir.is_ok() {
            reply.entry(&Timespec::new(0, 0), &attr(&dir.unwrap()), 0);
        } else {
            reply.error(dir.unwrap_err().raw_os_error().unwrap());
        }
    }
    fn rmdir(&mut self, _req: &Request, _parent: u64, _name: &OsStr, reply: ReplyEmpty) {
        /*self.log
        .printAction(&format!("Filesystem RmDir {}", _name.to_str().unwrap()));*/
        let err = self.api.rmdir(_parent, _name.to_str().unwrap());
        if err.is_err() {
            let err = err.unwrap_err();
            reply.error(err.raw_os_error().unwrap());
        } else {
            reply.ok();
        }
    }
    fn opendir(&mut self, _req: &Request, _ino: u64, _flags: u32, reply: ReplyOpen) {
        /*self.log
        .printInfo(&format!("Filesystem OpenDir INO {} FLAGS {}", _ino, _flags));*/
        reply.opened(_ino, 0);
    }

    fn readdir(
        &mut self,
        _req: &Request,
        _ino: u64,
        _fh: u64,
        _offset: i64,
        mut reply: ReplyDirectory,
    ) {
        /*self.log
        .printAction(&format!("Filesystem ReadDir INO {}", _ino));*/

        let mut entries = vec![
            //(1, FileType::Directory, "."),
            //(1, FileType::Directory, ".."),
           // (2, FileType::RegularFile, "hello.txt"),
        ];

        let readdir_entry = self.api.get_entry_from_inode(_ino);
        if readdir_entry.is_err() {
            //reply.error(0);
            return;
        }

        let dir_entries = self.api.readdir(&readdir_entry.unwrap().path);
        for dir_entr in dir_entries.iter() {
            let p = Path::new(dir_entr);
            let comp = p.components().last().expect("");
            let name = comp.as_os_str().to_str().expect("");
            let finfo = self.api.get_entry(dir_entr); // FIX
            if finfo.is_err() {
                continue;
            }
            let finfo = finfo.unwrap();
            let inode_num = attr(&finfo).ino;
            let kind = finfo.kind;
            let file_tup: (u64, FileType, &str) = (inode_num, to_filetype(&kind), &name);
            /*self.log.printInfo(&format!(
                "{} {} {}",
                file_tup.0,
                if to_filetype(&kind) == FileType::RegularFile {
                    "File"
                } else {
                    "Directory"
                },
                file_tup.2
            ));*/
            entries.push(file_tup);
        }

        for (i, entry) in entries.into_iter().enumerate().skip(_offset as usize) {
            // i + 1 means the index of the next entry
            reply.add(entry.0, (i + 1) as i64, entry.1, entry.2);
        }
        reply.ok();
    }

    fn releasedir(&mut self, _req: &Request, _ino: u64, _fh: u64, _flags: u32, reply: ReplyEmpty) {
        //self.log.printInfo("Filesystem ReleaseDir");
        reply.ok();
    }

    // TODO : Implement fsyncdir
    fn fsyncdir(
        &mut self,
        _req: &Request,
        _ino: u64,
        _fh: u64,
        _datasync: bool,
        reply: ReplyEmpty,
    ) {
        reply.ok();
    }

    // ETC

    // TODO : Implement link
    fn link(
        &mut self,
        _req: &Request,
        _ino: u64,
        _newparent: u64,
        _newname: &OsStr,
        reply: ReplyEntry,
    ) {
        reply.error(0);
    }

    fn open(&mut self, _req: &Request, _ino: u64, _flags: u32, reply: ReplyOpen) {
        /*self.log
        .printAction(&format!("Filesystem OPEN INO {}", _ino));*/
        reply.opened(0, 0);
    }

    fn read(
        &mut self,
        _req: &Request,
        _ino: u64,
        _fh: u64,
        _offset: i64,
        _size: u32,
        reply: ReplyData,
    ) {
        /*self.log.printAction(&format!(
            "Filesystem Read INO {} OFFSET {} SIZE {}",
            _ino, _offset, _size
        ));*/
        let v = self.api.read(_ino, _offset, _size);
        if v.is_err() {
            let err = v.unwrap_err();
            reply.error(err.raw_os_error().unwrap());
        } else {
            let v = v.unwrap();
            reply.data(v.as_slice());
        }
    }

    fn write(
        &mut self,
        _req: &Request,
        _ino: u64,
        _fh: u64,
        _offset: i64,
        _data: &[u8],
        _flags: u32,
        reply: ReplyWrite,
    ) {
        println!("Fuse write");
        /*self.log
        .printAction(&format!("Filesystem Write INO {} OFFSET {}", _ino, _offset));*/
        let v = self.api.write(_ino, _offset, _data);
        if v.is_err() {
            let err = v.unwrap_err();
            reply.error(err.raw_os_error().unwrap());
        } else {
            reply.written(v.unwrap());
        }
    }

    fn flush(&mut self, _req: &Request, _ino: u64, _fh: u64, _lock_owner: u64, reply: ReplyEmpty) {
        //self.log.printInfo("Filesystem Flush");
        reply.ok();
    }

    fn release(
        &mut self,
        _req: &Request,
        _ino: u64,
        _fh: u64,
        _flags: u32,
        _lock_owner: u64,
        _flush: bool,
        reply: ReplyEmpty,
    ) {
        //self.log.printInfo("Filesystem Release");
        reply.ok();
    }

    // TODO : Implement fsync
    fn fsync(&mut self, _req: &Request, _ino: u64, _fh: u64, _datasync: bool, reply: ReplyEmpty) {
        reply.ok();
    }

    // TODO : Implement statfs
    fn statfs(&mut self, _req: &Request, _ino: u64, reply: ReplyStatfs) {
        //self.log.printInfo("Filesystem Statfs");
        let size = 1000;
        let used = 20;
        reply.statfs(size, size - used, size - used, 1, 0, 0, 0, 0);
    }

    fn create(
        &mut self,
        _req: &Request,
        _parent: u64,
        _name: &OsStr,
        _mode: u32,
        _flags: u32,
        reply: ReplyCreate,
    ) {
        /*self.log
        .printAction(&format!("Filesystem Create {}", _name.to_str().unwrap()));*/
        let dir = self.api.create(_parent, _name.to_str().expect(""));
        if dir.is_ok() {
            reply.created(&Timespec::new(0, 0), &attr(&dir.unwrap()), 0, 0, 0);
        } else {
            reply.error(dir.unwrap_err().raw_os_error().unwrap());
        }
    }

    // TODO : Implement getlk
    fn getlk(
        &mut self,
        _req: &Request,
        _ino: u64,
        _fh: u64,
        _lock_owner: u64,
        _start: u64,
        _end: u64,
        _typ: u32,
        _pid: u32,
        reply: ReplyLock,
    ) {
        reply.error(0);
    }

    // TODO : Implement setlk
    fn setlk(
        &mut self,
        _req: &Request,
        _ino: u64,
        _fh: u64,
        _lock_owner: u64,
        _start: u64,
        _end: u64,
        _typ: u32,
        _pid: u32,
        _sleep: bool,
        reply: ReplyEmpty,
    ) {
        reply.ok();
    }

    fn bmap(&mut self, _req: &Request, _ino: u64, _blocksize: u32, _idx: u64, reply: ReplyBmap) {
        reply.error(0);
    }

    #[cfg(target_os = "macos")]
    fn exchange(
        &mut self,
        _req: &Request,
        _parent: u64,
        _name: &OsStr,
        _newparent: u64,
        _newname: &OsStr,
        _options: u64,
        reply: ReplyEmpty,
    ) {
        reply.ok();
    }

    #[cfg(target_os = "macos")]
    fn getxtimes(&mut self, _req: &Request, _ino: u64, reply: ReplyXTimes) {
        reply.error(0);
    }
}
