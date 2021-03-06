use crate::filesystem_entry::FilesystemEntry;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "kind")]
/// Possible Responses of the Server
pub enum Response {
    // Status Responses
    #[serde(rename = "invalid")]
    Invalid,
    #[serde(rename = "error")]
    /// General Error containg String representation
    Error { error: String },
    #[serde(rename = "io_error")]
    /// IO Error Response containing the raw os error
    IOError { error: i32 },
    #[serde(rename = "ok")]
    Ok {},
    // Filesystem Responses
    #[serde(rename = "read_dir")]
    ReadDir { data: Vec<String> },
    #[serde(rename = "get_entry")]
    GetEntry { data: FilesystemEntry },
    #[serde(rename = "get_path")]
    GetPath { data: String },
    #[serde(rename = "read")]
    Read { data: Vec<u8> },
    #[serde(rename = "write")]
    Write { data: u32 },
    // Server Responses
    #[serde(rename = "list_volumes")]
    ListVolumes { data: Vec<String> },
    #[serde(rename = "volume_lookup")]
    VolumeLookup { id: u64 },
    #[serde(rename = "server_info")]
    ServerInfo { name: String, version: String },
    #[serde(rename = "compressed")]
    Compressed { data: Vec<u8> },
}
