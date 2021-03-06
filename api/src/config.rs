use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Global Server Configuration
pub struct ServerConfig {
    /// Name of the server
    pub name: Option<String>,
    /// List of volumes
    pub volumes: Vec<VolumeConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Volume Configuration
pub struct VolumeConfig {
    /// Name of the volume
    pub name: Option<String>,
    /// Description of the volume
    pub description: Option<String>,
    /// Root path of the volume
    pub root: String,
    /// If the volume should be discoverable by ```shfs list```
    pub discoverable: Option<bool>,
    /// If the volume should be accessable by everyone
    pub public: Option<bool>,
    /// Enable the Trash Feature
    pub trash_enabled: Option<bool>, // TODO : Implement Trash
    /// Read Only Volume
    pub readonly: Option<bool>,
}
