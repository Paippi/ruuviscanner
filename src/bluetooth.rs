use dbus::arg;
use dbus::blocking::Connection;
use std::error::Error;
use std::time::Duration;

/// Connects to a dbus bluetooth service.
///
/// Powers on and returns a connection to a dbus bluetooth service (bluez) and connects to hci0 interface.
///
/// # Panics
///
/// If the given interface or service doesn't exist on the machine.
///
/// # Examples
///
/// ```
/// use ruuviscanner::bluetooth::connect_bluetooth;
/// use std::time::Duration;
///
/// let conn = connect_bluetooth()?;
///
/// let proxy = conn.with_proxy(
///     "org.bluez",
///     // Replace AA_BB_CC_DD_EE_FF with your mac address you are connecting to.
///     format!("/org/bluez/hci0/AA_BB_CC_DD_EE_FF"),
///     Duration::from_millis(20),
/// );
///
/// let _id = proxy.match_signal(
///     move |h: PropertiesPropertiesChanged, _: &Connection, _: &Message| {
///         let tag_data =
///             SensorDataV5::from_dbus_changed_properties(h.changed_properties).unwrap();
///         // Do something here with tag data.
///         true
///     },
/// );
///
/// conn.process(Duration::from_millis(100)).unwrap();
/// ```
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
