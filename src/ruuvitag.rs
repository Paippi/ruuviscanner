use dbus::arg;
/// Implementation of ruuvi data format 5
/// https://github.com/ruuvi/ruuvi-sensor-protocols/blob/master/dataformat_05.md

const BATTERY_OFFSET: u16 = 1600;
const TX_POWER_OFFSET: i8 = -40;

/// Joins two u8 primitives together.
///
/// E.g. 0xA1 + 00xxB2 = 0xA1B2
fn join_u8(left: u8, right: u8) -> u16 {
    (left as u16) << 8 | right as u16
}

#[derive(Debug)]
pub struct SensorDataV5 {
    temperature: i16,
    humidity: u16,
    pressure: u16,
    acceleration: Acceleration,
    power_info: u16,
    movement_counter: u8,
    measurement_number: u16,
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
    fn temperature_in_millicelcius(&self) -> i16 {
        self.temperature * 5
    }
    fn temperature_in_celcius(&self) -> f64 {
        self.temperature_in_millicelcius() as f64 / 1000_f64
    }
    fn get_humidity(&self) -> f64 {
        self.humidity as f64 / 400_f64
    }
    fn get_pressure(&self) -> u32 {
        50000 + self.pressure as u32
    }
    fn get_acceleration_in_mg(&self) -> &Acceleration {
        &self.acceleration
    }
    fn get_battery_voltage(&self) -> u16 {
        let power_info = self.power_info;
        // battery voltage in millivolts
        let mut battery_mv = power_info >> 5;
        battery_mv += BATTERY_OFFSET;
        battery_mv
    }
    fn get_tx_power(&self) -> i8 {
        let power_info = self.power_info;
        // TX power in decibel millivolts
        let mut tx_power_dbm = (power_info & 0x1f) as i8;
        tx_power_dbm += TX_POWER_OFFSET;
        tx_power_dbm
    }
    fn mac_as_str(&self) -> String {
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
    _x: i16,
    _y: i16,
    _z: i16,
}
