use dbus::arg;
use dbus::arg::RefArg;
use dbus::blocking::stdintf::org_freedesktop_dbus::PropertiesPropertiesChanged;
use dbus::blocking::Connection;
use dbus::Message;
use std::time::Duration;

mod hci0;
// USE MANAGER's DefaultAdapter method to get the default bluetooth device (read dongle)
// https://www.landley.net/kdocs/ols/2006/ols2006v1-pages-421-426.pdf

const _BLUETOOTH_MAC: &str = "00:1A:7D:DA:71:11";
const _RUUVITAG_MAC: &str = "CC:6F:70:EE:4C:AD";
const BATTERY_OFFSET: u16 = 1600;
const TX_POWER_OFFSET: i8 = -40;

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

#[derive(Debug)]
struct SensorDataV5 {
    temperature: Option<i16>,
    humidity: Option<u16>,
    pressure: Option<u16>,
    acceleration: Option<Acceleration>,
    power_info: Option<u16>,
    movement_counter: Option<u8>,
    measurement_number: Option<u16>,
    mac: Option<[u8; 6]>,
}

/// Joins two u8 primitives together.
///
/// E.g. A1 + B2 = A1B2
fn join_u8(left: u8, right: u8) -> u16 {
    (left as u16) << 8 | right as u16
}

impl SensorDataV5 {
    pub fn _new(
        temperature: Option<i16>,
        humidity: Option<u16>,
        pressure: Option<u16>,
        acceleration: Option<Acceleration>,
        power_info: Option<u16>,
        movement_counter: Option<u8>,
        measurement_number: Option<u16>,
        mac: Option<[u8; 6]>,
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
    pub fn from_dbus_changed_properties(changed_properties: arg::PropMap) -> Result<Self, String> {
        let data: Vec<&dyn arg::RefArg> = match changed_properties["ManufacturerData"].0.as_iter() {
            Some(x) => x.collect(),
            None => return Err("ManufacturerData couldn't be collected".to_string()),
        };
        if data.len() != 2 {
            return Err(format!("Missing data in changed_properties {data:?}"));
        }
        let _manufacturer_key = data[0];
        // data[1] is a `Variant` of one list so make it a iterable and take the first element.
        let manufacturer_data = data[1].as_iter().unwrap().next().unwrap();

        let mut temp: Vec<u8> = Vec::new();
        for item in manufacturer_data.as_iter().unwrap() {
            temp.push(item.as_i64().unwrap() as u8);
        }
        if temp.len() != 24 {
            return Err(format!("Missing manufacturer data {temp:?}"));
        }
        let _data_format = temp[0];
        let temperature = join_u8(temp[1], temp[2]) as i16;
        let humidity = join_u8(temp[3], temp[4]);
        let pressure = join_u8(temp[5], temp[6]);
        let acceleration = Acceleration {
            _x: join_u8(temp[7], temp[8]) as i16,
            _y: join_u8(temp[9], temp[10]) as i16,
            _z: join_u8(temp[11], temp[12]) as i16,
        };
        let power_info = join_u8(temp[13], temp[14]);
        let movement_counter = temp[15];
        let measurement_number = join_u8(temp[16], temp[17]);
        let mac: [u8; 6] = [temp[18], temp[19], temp[20], temp[21], temp[22], temp[23]];
        Ok(SensorDataV5 {
            temperature: Some(temperature),
            humidity: Some(humidity),
            pressure: Some(pressure),
            acceleration: Some(acceleration),
            power_info: Some(power_info),
            movement_counter: Some(movement_counter),
            measurement_number: Some(measurement_number),
            mac: Some(mac),
        })
    }
    fn temperature_in_millicelcius(&self) -> Result<i16, String> {
        match self.temperature {
            Some(x) => Ok(x * 5),
            None => Err("Temperature is None".to_string()),
        }
    }
    fn temperature_in_celcius(&self) -> Result<f64, String> {
        Ok(self.temperature_in_millicelcius()? as f64 / 1000 as f64)
    }
    fn get_humidity(&self) -> Result<f64, String> {
        match self.humidity {
            Some(x) => Ok(x as f64 / 400 as f64),
            None => Err("Humidity is None".to_string()),
        }
    }
    fn get_pressure(&self) -> Result<u32, String> {
        match self.pressure {
            Some(x) => Ok(50000 + x as u32),
            None => Err("pressure is None".to_string()),
        }
    }
    fn get_acceleration_in_mg(&self) -> Result<&Acceleration, String> {
        match self.acceleration {
            Some(ref x) => Ok(x),
            None => Err("Acceleration is None".to_string()),
        }
    }
    fn get_battery_voltage(&self) -> Result<u16, String> {
        let power_info = self.power_info.ok_or("power_info is None")?;
        // battery voltage in millivolts
        let mut battery_mv = power_info >> 5;
        battery_mv += BATTERY_OFFSET;
        Ok(battery_mv)
    }
    fn get_tx_power(&self) -> Result<i8, String> {
        let power_info = self.power_info.ok_or("power_info is None")?;
        // TX power in decibel millivolts
        let mut tx_power_dbm = (power_info & 0x1f) as i8;
        tx_power_dbm += TX_POWER_OFFSET;
        Ok(tx_power_dbm)
    }
    fn mac_as_str(&self) -> Option<String> {
        match self.mac {
            Some(x) => Some(
                x.iter()
                    .map(|x| format!("{:02X}", x))
                    .collect::<Vec<String>>()
                    .join(":"),
            ),
            None => None,
        }
    }
}

#[derive(Debug)]
struct Acceleration {
    _x: i16,
    _y: i16,
    _z: i16,
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
                let tag_data =
                    SensorDataV5::from_dbus_changed_properties(h.changed_properties).unwrap();
                println!("MAC address: {:?}", tag_data.mac_as_str().unwrap());
                println!(
                    "temperature in millicelcius (°mC): {:?}",
                    tag_data.temperature_in_millicelcius().unwrap()
                );
                println!(
                    "temperature in celcius (°C): {:?}",
                    tag_data.temperature_in_celcius().unwrap()
                );
                println!("humidity (%): {:?}", tag_data.get_humidity().unwrap());
                println!(
                    "Atmoshperic pressure (Pa): {:?}",
                    tag_data.get_pressure().unwrap()
                );
                println!(
                    "Acceleration (mG): {:?}",
                    tag_data.get_acceleration_in_mg().unwrap()
                );
                println!(
                    "Battery voltage (mV): {:?}",
                    tag_data.get_battery_voltage().unwrap()
                );
                println!("Tx Power (dBm): {:?}", tag_data.get_tx_power().unwrap());
                println!("Movement counter: {:?}", tag_data.movement_counter.unwrap());
                println!(
                    "Measurement sequence number: {:?}",
                    tag_data.measurement_number.unwrap()
                );
                println!("");
                true
            },
        );
    }

    loop {
        conn.process(Duration::from_millis(1000))?;
    }
}
