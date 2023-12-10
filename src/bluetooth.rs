use dbus::arg;
use dbus::blocking::Connection;
use std::error::Error;
use std::time::Duration;

pub fn connect_bluetooth() -> Result<Connection, Box<(dyn Error + 'static)>> {
    let conn = Connection::new_system().unwrap();
    let set_bluetooth_on_proxy =
        conn.with_proxy("org.bluez", "/org/bluez/hci0", Duration::from_millis(5000));

    set_bluetooth_on_proxy.method_call(
        "org.freedesktop.DBus.Properties",
        "Set",
        ("org.bluez.Adapter1", "Powered", arg::Variant(true)),
    )?;
    set_bluetooth_on_proxy.method_call("org.bluez.Adapter1", "StartDiscovery", ())?;
    Ok(conn)
}
