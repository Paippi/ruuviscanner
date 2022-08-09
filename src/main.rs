use dbus::arg;
use dbus::arg::RefArg;
use dbus::arg::Variant;
use dbus::blocking::stdintf::org_freedesktop_dbus::PropertiesPropertiesChanged;
use dbus::blocking::Connection;
use dbus::Message;
use ruuvi_sensor_protocol::{ParseError, SensorValues};
use std::convert::TryInto;
use std::time::Duration;

mod hci0;
// USE MANAGER's DefaultAdapter method to get the default bluetooth device (read dongle)
// https://www.landley.net/kdocs/ols/2006/ols2006v1-pages-421-426.pdf

const _BLUETOOTH_MAC: &str = "00:1A:7D:DA:71:11";
const _RUUVITAG_MAC: &str = "CC:6F:70:EE:4C:AD";

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

struct SensorDataV5 {
    temperature: Option<i16>,
    humidity: Option<u16>,
    pressure: Option<u16>,
    acceleration: Option<Acceleration>,
    power_info: Option<u16>,
    movement_counter: Option<u8>,
    measurement_number: Option<u16>,
    mac: [u8; 6],
}

/// Joins two u8 primitives together.
///
/// E.g. A1 + B2 = A1B2
fn join_u8(left: u8, right: u8) -> u16 {
    (left as u16) << 8 | right as u16
}

impl SensorDataV5 {
    pub fn new(
        temperature: Option<i16>,
        humidity: Option<u16>,
        pressure: Option<u16>,
        acceleration: Option<Acceleration>,
        power_info: Option<u16>,
        movement_counter: Option<u8>,
        measurement_number: Option<u16>,
        mac: [u8; 6],
    ) -> Self {
        Self {
            temperature,
            humidity,
            pressure,
            acceleration,
            power_info,
            movement_counter,
            measurement_number,
            mac,
        }
    }
    pub fn from_dbus_changed_properties(changed_properties: arg::PropMap) -> Self {
        let data: Vec<&dyn arg::RefArg> = changed_properties["ManufacturerData"]
            .0
            .as_iter()
            .unwrap()
            .collect();

        let _manufacturer_key = data[0];
        // data[1] is a `Variant` of one list so make it a iterable and take the first element.
        let manufacturer_data = data[1].as_iter().unwrap().next().unwrap();
        let mut temp: Vec<u8> = Vec::new();
        for item in manufacturer_data.as_iter().unwrap() {
            temp.push(item.as_i64().unwrap() as u8);
        }
        let _data_format = temp[0];
        let temperature = join_u8(temp[1], temp[2]) as i16;
        let humidity = join_u8(temp[3], temp[4]);
        let pressure = join_u8(temp[5], temp[6]);
        let acceleration = Acceleration {
            x: join_u8(temp[7], temp[8]) as i16,
            y: join_u8(temp[9], temp[10]) as i16,
            z: join_u8(temp[11], temp[12]) as i16,
        };
        let power_info = join_u8(temp[13], temp[14]);
        let movement_counter = temp[15];
        let measurement_number = join_u8(temp[16], temp[17]);
        let mac: [u8; 6] = [temp[18], temp[19], temp[20], temp[21], temp[22], temp[23]];
        SensorDataV5 {
            temperature: Some(temperature),
            humidity: Some(humidity),
            pressure: Some(pressure),
            acceleration: Some(acceleration),
            power_info: Some(power_info),
            movement_counter: Some(movement_counter),
            measurement_number: Some(measurement_number),
            mac,
        }
    }
}

struct Acceleration {
    x: i16,
    y: i16,
    z: i16,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::new_system()?;
    {
        let proxy = conn.with_proxy(
            "org.bluez",
            "/org/bluez/hci0/dev_C0_CB_4E_3D_3E_12",
            Duration::from_millis(5000),
        );

        let _id = proxy.match_signal(
            |h: PropertiesPropertiesChanged, _: &Connection, _: &Message| {
                // let data: Vec<&dyn arg::RefArg> = h.changed_properties["ManufacturerData"];
                //     .0
                //     .as_iter()
                //     .unwrap()
                //     .collect();
                // let data: Vec<u8> = h.changed_properties.get("ManufacturerData").unwrap();
                let tag_data = SensorDataV5::from_dbus_changed_properties(h.changed_properties);

                // let foo: &dyn arg::RefArg = manufacturer_data
                //     .as_iter()
                //     .unwrap()
                //     .collect::<Vec<&dyn arg::RefArg>>()[0];
                // let what = arg::cast::<Vec<u8>>(foo.clone());

                // _print_refarg(foo);
                // println!("foo: {foo:?}");

                // let manufacturer_key = data.iter().collect();

                // let data: Vec<&dyn arg::RefArg> = foo.as_iter().unwrap().collect();
                // let manufacturer_key = data[0].as_i64().unwrap();
                // let manufacturer_data: Vec<&dyn arg::RefArg> = data[1].as_iter().unwrap().collect();
                // println!("{}", manufacturer_data.len());
                // println!("{:?}", manufacturer_data[0]);
                // let what: Vec<&dyn arg::RefArg> = manufacturer_data[0].as_iter().unwrap().collect();
                // let result = SensorValues::from_manufacturer_specific_data(
                //     manufacturer_key.try_into().unwrap(),
                //     manufacturer_data.try_into().unwrap(),
                // );

                // let data: dbus::arg::Variant<Box<dyn RefArg>> =
                //     h.changed_properties["ManufacturerData"].into();
                //let data = &h.changed_properties["ManufacturerData"];
                //
                // let dbus::arg::Variant(data) = &h.changed_properties["ManufacturerData"];

                // let p: Option<&dbus::arg::Variant<Box<dyn dbus::arg::RefArg>>> =
                true
            },
        );
    }

    loop {
        conn.process(Duration::from_millis(1000))?;
    }
}
