use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Additional Information about a request
pub struct RequestInfo {
    pub volume_id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "kind")]
/// Possible Calls from Client
pub enum Call {
    // Filesystem Requests
    #[serde(rename = "read_dir")]
    ReadDir { info: RequestInfo, path: String },
    #[serde(rename = "get_entry")]
    GetEntry { info: RequestInfo, path: String },
    #[serde(rename = "get_entry_from_inode")]
    GetEntryFromInode { info: RequestInfo, ino: u64 },
    #[serde(rename = "get_path_from_inode")]
    GetPathFromInode { info: RequestInfo, ino: u64 },
    #[serde(rename = "read")]
    Read {
        info: RequestInfo,
        ino: u64,
        offset: i64,
        size: u32,
    },
    #[serde(rename = "rename")]
    Rename {
        info: RequestInfo,
        parent: u64,
        name: String,
        nparent: u64,
        nname: String,
    },
    #[serde(rename = "mkdir")]
    Mkdir {
        info: RequestInfo,
        parent: u64,
        name: String,
    },
    #[serde(rename = "rmdir")]
    Rmdir {
        info: RequestInfo,
        parent: u64,
        name: String,
    },
    #[serde(rename = "create")]
    Create {
        info: RequestInfo,
        parent: u64,
        name: String,
    },
    #[serde(rename = "unlink")]
    Unlink {
        info: RequestInfo,
        parent: u64,
        name: String,
    },
    #[serde(rename = "write")]
    Write {
        info: RequestInfo,
        ino: u64,
        offset: i64,
        data: Vec<u8>,
    },

    // Server Requests
    #[serde(rename = "list_volumes")]
    ListVolumes {},
    #[serde(rename = "volume_lookup")]
    VolumeLookup { name: String },
    #[serde(rename = "server_info")]
    ServerInfo {},
}
