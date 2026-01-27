use super::{AppModel, ConnectedDevice, DeviceData, Message};
use crate::app::APP_ID;
use crate::config::Config;
use crate::fl;
use crate::services::{
    connect_device, disconnect_device, discover_device_by_address, list_connected_devices,
    list_paired_devices, pair_device, read_ps_controller_battery, remove_device,
    rename_paired_device, trust_device,
};
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::futures::SinkExt;
use cosmic::iced::{Limits, Subscription, time, window::Id};
use cosmic::iced_winit::commands::popup::{destroy_popup, get_popup};
use cosmic::prelude::*;
use std::time::Duration;
use zbus::Connection;

pub fn init(core: cosmic::Core, _flags: ()) -> (AppModel, Task<cosmic::Action<Message>>) {
    let config = cosmic_config::Config::new(APP_ID, Config::VERSION)
        .map(|context| match Config::get_entry(&context) {
            Ok(config) => config,
            Err((_errors, config)) => config,
        })
        .unwrap_or_default();

    let app = AppModel {
        core,
        config,
        ..Default::default()
    };

    (
        app,
        cosmic::task::future(async { Message::DataLoaded(load_devices().await) }),
    )
}

pub fn subscription(app: &AppModel) -> Subscription<Message> {
    struct RefreshSubscription;

    Subscription::batch(vec![
        Subscription::run_with_id(
            std::any::TypeId::of::<RefreshSubscription>(),
            cosmic::iced::stream::channel(4, move |mut channel| async move {
                loop {
                    let _ = channel.send(Message::Refresh).await;
                    tokio::time::sleep(Duration::from_secs(10)).await;
                }
            }),
        ),
        time::every(Duration::from_secs(1)).map(|_| Message::Tick),
        app.core
            .watch_config::<Config>(APP_ID)
            .map(|update| Message::UpdateConfig(update.config)),
    ])
}

pub fn update(app: &mut AppModel, message: Message) -> Task<cosmic::Action<Message>> {
    match message {
        Message::Refresh => {
            if !app.reconnecting.is_empty() || app.renaming_addr.is_some() {
                return Task::none();
            }
            return cosmic::task::future(async { Message::DataLoaded(load_devices().await) });
        }
        Message::Tick => {
            if app.reconnecting.is_empty() {
                return Task::none();
            }

            app.reconnecting.retain(|_, remaining| {
                if *remaining > 0 {
                    *remaining -= 1;
                    true
                } else {
                    false
                }
            });
        }
        Message::DataLoaded(Ok(data)) => {
            app.connected = data.connected;
            let mut paired = data.paired;
            app.paired_names = paired
                .iter()
                .map(|(addr, name)| (addr.clone(), name.clone()))
                .collect();

            for (addr, _) in &app.reconnecting {
                if paired.iter().all(|(known, _)| known != addr) {
                    let name = app
                        .paired_names
                        .get(addr)
                        .cloned()
                        .unwrap_or_else(|| addr.clone());
                    paired.push((addr.clone(), name));
                }
            }

            app.paired = paired;
            app.last_error = None;
        }
        Message::DataLoaded(Err(error)) => {
            app.last_error = Some(error);
        }
        Message::DisconnectDevice(addr) => {
            return cosmic::task::future(async move {
                Message::DisconnectResult(disconnect_by_addr(addr).await)
            });
        }
        Message::DisconnectResult(result) => {
            if let Err(error) = result {
                app.last_error = Some(error);
            } else {
                return cosmic::task::future(async { Message::DataLoaded(load_devices().await) });
            }
        }
        Message::RenameStart(addr, current) => {
            app.renaming_addr = Some(addr);
            app.renaming_value = current;
        }
        Message::RenameInput(value) => {
            app.renaming_value = value;
        }
        Message::RenameCancel => {
            app.renaming_addr = None;
            app.renaming_value.clear();
        }
        Message::RenameSubmit(addr) => {
            let new_name = app.renaming_value.trim().to_string();
            if new_name.is_empty() {
                app.last_error = Some(fl!("rename-empty").to_string());
                return Task::none();
            }
            app.renaming_addr = None;
            app.renaming_value.clear();
            return cosmic::task::future(async move {
                Message::RenameResult(rename_by_addr(addr, new_name).await)
            });
        }
        Message::RenameResult(result) => {
            if let Err(error) = result {
                app.last_error = Some(error);
            } else {
                return cosmic::task::future(async { Message::DataLoaded(load_devices().await) });
            }
        }
        Message::RemoveDevice(addr) => {
            return cosmic::task::future(async move {
                Message::RemoveResult(remove_by_addr(addr).await)
            });
        }
        Message::RemoveResult(result) => {
            if let Err(error) = result {
                app.last_error = Some(error);
            } else {
                return cosmic::task::future(async { Message::DataLoaded(load_devices().await) });
            }
        }
        Message::ReconnectDevice(addr) => {
            app.reconnecting.insert(addr.clone(), 60);
            let name = app
                .paired_names
                .get(&addr)
                .cloned()
                .or_else(|| {
                    app.paired
                        .iter()
                        .find(|(known, _)| known == &addr)
                        .map(|(_, name)| name.clone())
                })
                .unwrap_or_else(|| addr.clone());
            return cosmic::task::future(async move {
                Message::ReconnectResult(reconnect_by_addr(addr, name).await)
            });
        }
        Message::ReconnectResult(result) => match result {
            Ok((addr, _found)) => {
                app.reconnecting.remove(&addr);
                return cosmic::task::future(async { Message::DataLoaded(load_devices().await) });
            }
            Err(error) => {
                app.last_error = Some(error);
            }
        },
        Message::UpdateConfig(config) => {
            app.config = config;
        }
        Message::TogglePopup => {
            return if let Some(p) = app.popup.take() {
                destroy_popup(p)
            } else {
                let new_id = Id::unique();
                app.popup.replace(new_id);
                let mut popup_settings = app.core.applet.get_popup_settings(
                    app.core.main_window_id().unwrap(),
                    new_id,
                    None,
                    None,
                    None,
                );
                popup_settings.positioner.size_limits = Limits::NONE
                    .max_width(520.0)
                    .min_width(320.0)
                    .min_height(200.0)
                    .max_height(1080.0);
                get_popup(popup_settings)
            };
        }
        Message::PopupClosed(id) => {
            if app.popup.as_ref() == Some(&id) {
                app.popup = None;
            }
        }
    }

    Task::none()
}

pub async fn load_devices() -> Result<DeviceData, String> {
    let conn = Connection::system()
        .await
        .map_err(|err| format!("DBus error: {err}"))?;

    let connected = list_connected_devices(&conn)
        .await
        .map_err(|err| format!("Failed to list connected devices: {err}"))?
        .into_iter()
        .map(|(addr, name)| ConnectedDevice {
            battery: read_ps_controller_battery(&addr.to_lowercase())
                .ok()
                .flatten()
                .map(|(_dev, capacity)| capacity),
            addr,
            name,
        })
        .collect();

    let paired = list_paired_devices(&conn)
        .await
        .map_err(|err| format!("Failed to list paired devices: {err}"))?;

    Ok(DeviceData { connected, paired })
}

async fn disconnect_by_addr(addr: String) -> Result<(), String> {
    let conn = Connection::system()
        .await
        .map_err(|err| format!("DBus error: {err}"))?;

    disconnect_device(&conn, &addr)
        .await
        .map_err(|err| format!("Failed to disconnect device: {err}"))?
        .then_some(())
        .ok_or_else(|| "Device not found".to_string())
}

async fn remove_by_addr(addr: String) -> Result<(), String> {
    let conn = Connection::system()
        .await
        .map_err(|err| format!("DBus error: {err}"))?;

    remove_device(&conn, &addr)
        .await
        .map_err(|err| format!("Failed to remove device: {err}"))?
        .then_some(())
        .ok_or_else(|| "Device not found".to_string())
}

async fn rename_by_addr(addr: String, new_alias: String) -> Result<(), String> {
    let conn = Connection::system()
        .await
        .map_err(|err| format!("DBus error: {err}"))?;

    rename_paired_device(&conn, &addr, &new_alias)
        .await
        .map_err(|err| format!("Failed to rename device: {err}"))?
        .then_some(())
        .ok_or_else(|| "Device not found".to_string())
}

async fn reconnect_by_addr(addr: String, name: String) -> Result<(String, bool), String> {
    let conn = Connection::system()
        .await
        .map_err(|err| format!("DBus error: {err}"))?;

    remove_device(&conn, &addr)
        .await
        .map_err(|err| format!("Failed to remove device: {err}"))?;

    let found = discover_device_by_address(&conn, &addr, Duration::from_secs(60))
        .await
        .map_err(|err| format!("Failed to discover device: {err}"))?;

    if !found {
        return Err(fl!("reconnect-not-found").to_string());
    }

    if !pair_device(&conn, &addr)
        .await
        .map_err(|err| format!("Failed to pair device: {err}"))?
    {
        return Err(fl!("reconnect-pair-failed").to_string());
    }

    if !trust_device(&conn, &addr, true)
        .await
        .map_err(|err| format!("Failed to trust device: {err}"))?
    {
        return Err(fl!("reconnect-trust-failed").to_string());
    }

    if !connect_device(&conn, &addr)
        .await
        .map_err(|err| format!("Failed to connect device: {err}"))?
    {
        return Err(fl!("reconnect-connect-failed").to_string());
    }

    if !name.is_empty() {
        let _ = rename_paired_device(&conn, &addr, &name).await;
    }

    Ok((addr, true))
}
