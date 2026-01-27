use crate::config::Config;
use cosmic::iced::window::Id;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ConnectedDevice {
    pub addr: String,
    pub name: String,
    pub battery: Option<u8>,
}

#[derive(Debug, Clone)]
pub struct DeviceData {
    pub connected: Vec<ConnectedDevice>,
    pub paired: Vec<(String, String)>,
}

#[derive(Default)]
pub struct AppModel {
    pub core: cosmic::Core,
    pub popup: Option<Id>,
    pub config: Config,
    pub connected: Vec<ConnectedDevice>,
    pub paired: Vec<(String, String)>,
    pub paired_names: HashMap<String, String>,
    pub reconnecting: HashMap<String, u64>,
    pub renaming_addr: Option<String>,
    pub renaming_value: String,
    pub last_error: Option<String>,
}
