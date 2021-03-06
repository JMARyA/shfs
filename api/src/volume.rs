use crate::config::VolumeConfig;
use crate::FilesystemAPI;
use std::path::Path;

/// Volume represented on the server
pub struct Volume {
    /// The name of the [Volume]
    pub name: String,
    /// The root path of the [Volume] on the Server
    pub root: String,
    /// General Config of the [Volume]
    pub config: VolumeConfig,
    /// Underlying [FilesystemAPI]
    pub api: FilesystemAPI,
}

impl Volume {
    pub fn new(conf: &VolumeConfig) -> Volume {
        let root = conf.root.to_string();
        let name: String;
        if conf.name.is_none() {
            name = String::from(Path::new(&root).file_name().unwrap().to_str().unwrap());
        } else {
            name = conf.name.clone().unwrap();
        }
        return Volume {
            name,
            root: root.clone(),
            config: conf.clone(),
            api: FilesystemAPI::new(root.to_string()),
        };
    }
}
