use crate::config::Config;
use cosmic::iced::window::Id;

#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    Refresh,
    Tick,
    DataLoaded(Result<super::DeviceData, String>),
    DisconnectDevice(String),
    DisconnectResult(Result<(), String>),
    RenameStart(String, String),
    RenameInput(String),
    RenameCancel,
    RenameSubmit(String),
    RenameResult(Result<(), String>),
    RemoveDevice(String),
    RemoveResult(Result<(), String>),
    ReconnectDevice(String),
    ReconnectResult(Result<(String, bool), String>),
    UpdateConfig(Config),
}
