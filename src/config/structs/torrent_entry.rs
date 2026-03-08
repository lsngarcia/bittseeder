use crate::config::enums::seed_protocol::SeedProtocol;
use serde::{
    Deserialize,
    Serialize
};

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TorrentEntry {
    pub name: Option<String>,
    #[serde(default)]
    pub file: Vec<String>,
    #[serde(default)]
    pub trackers: Vec<String>,
    pub webseed: Option<Vec<String>>,
    pub ice: Option<Vec<String>>,
    pub rtc_interval: Option<u64>,
    #[serde(default)]
    pub protocol: Option<SeedProtocol>,
    pub version: Option<String>,
    pub torrent_file: Option<String>,
    pub create_torrent: bool,
    pub magnet: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub upload_limit: Option<u64>,
    #[serde(default)]
    pub allowed_extensions: Option<Vec<String>>,
    #[serde(default)]
    pub private: bool,
}