use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxPolicy {
    pub meta: Meta,
    pub filesystem: Filesystem,
    pub network: Network,
    pub hardware: Hardware,
    pub system: System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    pub policy_version: String,
    pub strict_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filesystem {
    pub home_read: Permission,
    pub home_write: Permission,
    pub temp_access: Permission,
    pub documents_read: Permission,
    pub documents_write: Permission,
    pub downloads_read: Permission,
    pub system_read: Permission,
    pub arbitrary_path: Permission,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    pub internet: Permission,
    pub localhost: Permission,
    pub local_network: Permission,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hardware {
    pub camera: Permission,
    pub microphone: Permission,
    pub gpu_access: Permission,
    pub usb_devices: Permission,
    pub bluetooth: Permission,
    pub location: Permission,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct System {
    pub notifications: Permission,
    pub clipboard_read: Permission,
    pub clipboard_write: Permission,
    pub autostart: Permission,
    pub background_run: Permission,
    pub ipc: Permission,
    pub exec_subprocess: Permission,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Permission {
    Allow,
    Deny,
    Ask,
}

impl SandboxPolicy {
    pub fn from_toml(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string(self)
    }
}
