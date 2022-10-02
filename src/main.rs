use dbus::arg;
use dbus::blocking::Connection;
use dbus::nonblock::stdintf::org_freedesktop_dbus::PropertiesPropertiesChanged;
use dbus::Message;
use ruuviscanner::ruuvitag::SensorDataV5;
use std::error::Error;
use std::time::Duration;

mod hci0;
// USE MANAGER's DefaultAdapter method to get the default bluetooth device (read dongle)
// https://www.landley.net/kdocs/ols/2006/ols2006v1-pages-421-426.pdf

fn _print_refarg(value: &dyn arg::RefArg) {
    // We don't know what type the value is. We'll try a few and fall back to
    // debug printing if the value is more complex than that.
    if let Some(s) = value.as_str() {
        println!("str: {}", s);
    } else if let Some(i) = value.as_i64() {
        println!("int: {}", i);
    } else {
        println!("other: {:?}", value);
    }
}

fn start_bluetooth(conn: &Connection) -> Result<(), Box<(dyn Error + 'static)>> {
    let set_bluetooth_on_proxy =
        conn.with_proxy("org.bluez", "/org/bluez/hci0", Duration::from_millis(5000));

    set_bluetooth_on_proxy.method_call(
        "org.freedesktop.DBus.Properties",
        "Set",
        ("org.bluez.Adapter1", "Powered", arg::Variant(true)),
    )?;
    set_bluetooth_on_proxy.method_call("org.bluez.Adapter1", "StartDiscovery", ())?;
    println!("Bluetooth started");
    Ok(())
}

fn scan_ruuvitag(
    mut mac_addresses: Vec<String>,
    conn: &Connection,
) -> Result<(), Box<(dyn Error + 'static)>> {
    for mac in mac_addresses.iter_mut() {
        let mac_dbus_format = mac.replace(':', "_");
        *mac = format!("dev_{mac_dbus_format}");
    }

    for mac in mac_addresses {
        let proxy = conn.with_proxy(
            "org.bluez",
            format!("/org/bluez/hci0/{mac}"),
            Duration::from_millis(20),
        );

        let _id = proxy.match_signal(
            |h: PropertiesPropertiesChanged, _: &Connection, _: &Message| {
                let tag_data =
                    SensorDataV5::from_dbus_changed_properties(h.changed_properties).unwrap();
                tag_data.print_sensor_data();
                true
            },
        );
    }

    loop {
        conn.process(Duration::from_millis(20))?;
    }
}

fn main() -> Result<(), Box<(dyn Error + 'static)>> {
    let conn = Connection::new_system()?;
    let ruuvitags = vec![
        "C0:CB:4E:3D:3E:12".to_owned(),
        "E1:16:22:5D:F6:C9".to_owned(),
        "CC:6F:70:EE:4C:AD".to_owned(),
    ];
    start_bluetooth(&conn)?;
    scan_ruuvitag(ruuvitags, &conn)?;
    Ok(())
}
