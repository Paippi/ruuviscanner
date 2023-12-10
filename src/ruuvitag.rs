use crate::bluetooth::connect_bluetooth;
use dbus::arg;
use dbus::blocking::Connection;
use dbus::nonblock::stdintf::org_freedesktop_dbus::PropertiesPropertiesChanged;
use dbus::Message;
use std::convert::TryFrom;
use std::error::Error;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

/// Implementation of ruuvi data format 5
/// https://github.com/ruuvi/ruuvi-sensor-protocols/blob/master/dataformat_05.md

const BATTERY_OFFSET: u16 = 1600;
const TX_POWER_OFFSET: i8 = -40;

/// Joins two u8 primitives together.
///
/// E.g. 0xA1 + 0xB2 = 0xA1B2
fn join_u8(left: u8, right: u8) -> u16 {
    (left as u16) << 8 | right as u16
}

pub async fn subscribe_ruuvitag(
    mac_address: &str,
) -> Result<Receiver<SensorDataV5>, Box<(dyn Error + 'static)>> {
    let (tx, rx) = channel();
    let conn = connect_bluetooth()?;
    let mac_dbus_format = mac_address.replace(':', "_");
    let mac_address = format!("dev_{mac_dbus_format}");
    let proxy = conn.with_proxy(
        "org.bluez",
        format!("/org/bluez/hci0/{mac_address}"),
        Duration::from_millis(20),
    );
    let _id = proxy.match_signal(
        move |h: PropertiesPropertiesChanged, _: &Connection, _: &Message| {
            let tag_data =
                SensorDataV5::from_dbus_changed_properties(h.changed_properties).unwrap();
            // Cannot currently gracefully shutdown if receiver gets dropped before sender does.
            // Probably because dbus system bus is implemented as sync.
            // This will lead to panics, if the receiver gets dropped.
            // TBD: reimplement in dbus-tokio.
            // https://docs.rs/dbus-tokio/latest/dbus_tokio/connection/index.html
            tx.send(tag_data).unwrap();

            true
        },
    );
    tokio::spawn(async move {
        loop {
            conn.process(Duration::from_millis(20)).unwrap();
        }
    });
    Ok(rx)
}

// TODO: max numbers such as i32::MAX should be considered as invalid/data not available
#[derive(Debug)]
pub struct SensorDataV5 {
    temperature: i16,
    humidity: u16,
    pressure: u16,
    pub acceleration: Acceleration,
    power_info: u16,
    pub movement_counter: u8,
    pub measurement_number: u16,
    mac: [u8; 6],
}

impl SensorDataV5 {
    pub fn new(
        temperature: i16,
        humidity: u16,
        pressure: u16,
        acceleration: Acceleration,
        power_info: u16,
        movement_counter: u8,
        measurement_number: u16,
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
        // TODO: Assert the data format that it is V5.
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

        Ok(SensorDataV5::new(
            temperature,
            humidity,
            pressure,
            acceleration,
            power_info,
            movement_counter,
            measurement_number,
            mac,
        ))
    }
    pub fn temperature_in_millicelcius(&self) -> i32 {
        // TODO: optimization wise it might be better to set self.temperature as i32 so we don't
        // need to cast it everytime. though memory wise it would be better to use i16 but I think
        // compiler might do this for us.
        i32::try_from(self.temperature).unwrap() * 5
    }
    pub fn temperature_in_celcius(&self) -> f64 {
        self.temperature_in_millicelcius() as f64 / 1000_f64
    }
    pub fn get_humidity(&self) -> f64 {
        self.humidity as f64 / 400_f64
    }
    pub fn get_pressure(&self) -> u32 {
        50000 + self.pressure as u32
    }
    pub fn get_acceleration_in_mg(&self) -> &Acceleration {
        &self.acceleration
    }
    pub fn get_battery_voltage(&self) -> u16 {
        let power_info = self.power_info;
        // battery voltage in millivolts
        let mut battery_mv = power_info >> 5;
        battery_mv += BATTERY_OFFSET;
        battery_mv
    }
    pub fn get_tx_power(&self) -> i8 {
        let power_info = self.power_info;
        // TX power in decibel millivolts
        let mut tx_power_dbm = (power_info & 0x1f) as i8 * 2;
        tx_power_dbm += TX_POWER_OFFSET;
        tx_power_dbm
    }
    pub fn mac_as_str(&self) -> String {
        self.mac
            .iter()
            .map(|x| format!("{:02X}", x))
            .collect::<Vec<String>>()
            .join(":")
    }
    pub fn print_sensor_data(&self) {
        println!("MAC address: {:?}", self.mac_as_str());
        println!(
            "temperature in millicelcius (°mC): {:?}",
            self.temperature_in_millicelcius()
        );
        println!(
            "temperature in celcius (°C): {:?}",
            self.temperature_in_celcius()
        );
        println!("humidity (%): {:?}", self.get_humidity());
        println!("Atmoshperic pressure (Pa): {:?}", self.get_pressure());
        println!("Acceleration (mG): {:?}", self.get_acceleration_in_mg());
        println!("Battery voltage (mV): {:?}", self.get_battery_voltage());
        println!("Tx Power (dBm): {:?}", self.get_tx_power());
        println!("Movement counter: {:?}", self.movement_counter);
        println!("Measurement sequence number: {:?}", self.measurement_number);
        println!();
    }
}

#[derive(Debug)]
pub struct Acceleration {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

impl Acceleration {
    pub fn new(x: i16, y: i16, z: i16) -> Acceleration {
        Acceleration { x, y, z }
    }
}

#[cfg(test)]
mod tests {

    use crate::ruuvitag::{Acceleration, SensorDataV5};

    #[test]
    fn test_ruuvitag_sensor_data_v5_min() {
        let sensor_data = SensorDataV5::new(
            i16::MIN,
            u16::MIN,
            u16::MIN,
            Acceleration::new(i16::MIN, i16::MIN, i16::MIN),
            u16::MIN,
            u8::MIN,
            u16::MIN,
            [u8::MIN, u8::MIN, u8::MIN, u8::MIN, u8::MIN, u8::MIN],
        );
        sensor_data.temperature_in_millicelcius();
    }
}
