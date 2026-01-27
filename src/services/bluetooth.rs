use std::fs;
use std::io;
use std::time::{Duration, Instant};
use zbus::names::InterfaceName;
use zbus::zvariant::{OwnedObjectPath, OwnedValue, Value};
use zbus::{Connection, Proxy, fdo::ObjectManagerProxy, fdo::PropertiesProxy};

pub async fn find_device_path(
    conn: &Connection,
    address: &str,
) -> zbus::Result<Option<OwnedObjectPath>> {
    let om = ObjectManagerProxy::builder(conn)
        .destination("org.bluez")?
        .path("/")?
        .build()
        .await?;

    let objects = om.get_managed_objects().await?;

    for (path, ifaces) in objects {
        let Some(dev) = ifaces.get("org.bluez.Device1") else {
            continue;
        };

        let addr = get_string(dev, "Address").unwrap_or_default();
        if addr.eq_ignore_ascii_case(address) {
            return Ok(Some(path));
        }
    }

    Ok(None)
}

pub async fn pair_device(conn: &Connection, address: &str) -> zbus::Result<bool> {
    let Some(path) = find_device_path(conn, address).await? else {
        return Ok(false);
    };

    let dev = Proxy::new(conn, "org.bluez", path, "org.bluez.Device1").await?;

    dev.call_method("Pair", &()).await?;
    Ok(true)
}

pub async fn trust_device(conn: &Connection, address: &str, trusted: bool) -> zbus::Result<bool> {
    let Some(path) = find_device_path(conn, address).await? else {
        return Ok(false);
    };

    let props = PropertiesProxy::builder(conn)
        .destination("org.bluez")?
        .path(path)?
        .build()
        .await?;

    let iface = InterfaceName::try_from("org.bluez.Device1")?;
    props.set(iface, "Trusted", Value::from(trusted)).await?;
    Ok(true)
}

pub async fn connect_device(conn: &Connection, address: &str) -> zbus::Result<bool> {
    let Some(path) = find_device_path(conn, address).await? else {
        return Ok(false);
    };

    let dev = Proxy::new(conn, "org.bluez", path, "org.bluez.Device1").await?;

    dev.call_method("Connect", &()).await?;
    Ok(true)
}

pub async fn disconnect_device(conn: &Connection, address: &str) -> zbus::Result<bool> {
    let Some(path) = find_device_path(conn, address).await? else {
        return Ok(false);
    };

    let dev = Proxy::new(conn, "org.bluez", path, "org.bluez.Device1").await?;

    dev.call_method("Disconnect", &()).await?;
    Ok(true)
}

pub async fn remove_device(conn: &Connection, address: &str) -> zbus::Result<bool> {
    let Some(dev_path) = find_device_path(conn, address).await? else {
        return Ok(false);
    };

    let props = PropertiesProxy::builder(conn)
        .destination("org.bluez")?
        .path(dev_path.clone())?
        .build()
        .await?;

    let iface = InterfaceName::try_from("org.bluez.Device1")?;
    let adapter: OwnedObjectPath = props
        .get(iface, "Adapter")
        .await?
        .try_into()
        .map_err(|_| zbus::Error::Failure("Invalid Adapter property".into()))?;

    let adapter_proxy = Proxy::new(conn, "org.bluez", adapter, "org.bluez.Adapter1").await?;

    adapter_proxy
        .call_method("RemoveDevice", &(dev_path))
        .await?;
    Ok(true)
}

pub async fn rename_paired_device(
    conn: &Connection,
    address: &str,
    new_alias: &str,
) -> zbus::Result<bool> {
    let om = ObjectManagerProxy::builder(conn)
        .destination("org.bluez")?
        .path("/")?
        .build()
        .await?;

    let objects = om.get_managed_objects().await?;

    for (path, ifaces) in objects {
        let Some(dev) = ifaces.get("org.bluez.Device1") else {
            continue;
        };

        let addr = get_string(dev, "Address").unwrap_or_default();
        let paired = get_bool(dev, "Paired").unwrap_or(false);

        if paired && addr.eq_ignore_ascii_case(address) {
            let props = PropertiesProxy::builder(conn)
                .destination("org.bluez")?
                .path(path)?
                .build()
                .await?;
            let iface = InterfaceName::try_from("org.bluez.Device1")?;

            props.set(iface, "Alias", Value::from(new_alias)).await?;

            return Ok(true);
        }
    }

    Ok(false)
}

fn get_bool(props: &std::collections::HashMap<String, OwnedValue>, key: &str) -> Option<bool> {
    props.get(key).cloned().and_then(|v| v.try_into().ok())
}

fn get_string(props: &std::collections::HashMap<String, OwnedValue>, key: &str) -> Option<String> {
    props.get(key).cloned().and_then(|v| v.try_into().ok())
}

pub fn read_ps_controller_battery(addr: &str) -> io::Result<Option<(String, u8)>> {
    let dir = fs::read_dir("/sys/class/power_supply")?;

    for entry in dir {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();

        if !name.contains(addr) {
            continue;
        }

        let capacity_path = entry.path().join("capacity");
        let capacity_str = fs::read_to_string(capacity_path)?;
        if let Ok(capacity) = capacity_str.trim().parse::<u8>() {
            return Ok(Some((name.into_owned(), capacity)));
        }
    }

    Ok(None)
}

async fn list_devices_by<F>(
    conn: &Connection,
    mut predicate: F,
) -> zbus::Result<Vec<(String, String)>>
where
    F: FnMut(bool, bool) -> bool,
{
    let om = ObjectManagerProxy::builder(conn)
        .destination("org.bluez")?
        .path("/")?
        .build()
        .await?;

    let objects = om.get_managed_objects().await?;
    let mut devices = Vec::new();

    for (_path, ifaces) in objects {
        let Some(dev) = ifaces.get("org.bluez.Device1") else {
            continue;
        };

        let connected = get_bool(dev, "Connected").unwrap_or(false);
        let paired = get_bool(dev, "Paired").unwrap_or(false);

        if !predicate(connected, paired) {
            continue;
        }

        let addr = get_string(dev, "Address").unwrap_or_else(|| "<unknown>".into());
        let name = get_string(dev, "Alias")
            .or_else(|| get_string(dev, "Name"))
            .unwrap_or_else(|| "<unnamed>".into());

        devices.push((addr, name));
    }

    Ok(devices)
}

pub async fn list_connected_devices(conn: &Connection) -> zbus::Result<Vec<(String, String)>> {
    list_devices_by(conn, |connected, _paired| connected).await
}

pub async fn list_paired_devices(conn: &Connection) -> zbus::Result<Vec<(String, String)>> {
    list_devices_by(conn, |_connected, paired| paired).await
}

async fn get_default_adapter_path(conn: &Connection) -> zbus::Result<Option<OwnedObjectPath>> {
    let om = ObjectManagerProxy::builder(conn)
        .destination("org.bluez")?
        .path("/")?
        .build()
        .await?;

    let objects = om.get_managed_objects().await?;

    for (path, ifaces) in objects {
        if ifaces.contains_key("org.bluez.Adapter1") {
            return Ok(Some(path));
        }
    }

    Ok(None)
}

pub async fn discover_device_by_address(
    conn: &Connection,
    address: &str,
    timeout: Duration,
) -> zbus::Result<bool> {
    if find_device_path(conn, address).await?.is_some() {
        return Ok(true);
    }

    let Some(adapter_path) = get_default_adapter_path(conn).await? else {
        return Ok(false);
    };

    let adapter_proxy = Proxy::new(conn, "org.bluez", adapter_path, "org.bluez.Adapter1").await?;
    adapter_proxy.call_method("StartDiscovery", &()).await?;

    let start = Instant::now();
    let mut last_printed = None;
    loop {
        if find_device_path(conn, address).await?.is_some() {
            adapter_proxy.call_method("StopDiscovery", &()).await?;
            return Ok(true);
        }

        let elapsed = start.elapsed();
        if elapsed >= timeout {
            adapter_proxy.call_method("StopDiscovery", &()).await?;
            return Ok(false);
        }

        let remaining_secs = timeout.saturating_sub(elapsed).as_secs();
        if last_printed != Some(remaining_secs) {
            println!("Discovery: {remaining_secs}s left...");
            last_printed = Some(remaining_secs);
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
