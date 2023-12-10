//! Ruuviscanner
//!
//! Retreive ruuvitag data using Rust and D-Bus!
//!
//! ## Examples
//!
//! ```rust
//! use ruuviscanner::ruuvitag::{subscribe_ruuvitag, SensorDataV5};
//!
//! let mac = "<mac address of you ruuvitag>";
//! let rx = subscribe_ruuvitag(&mac).await?;
//! loop {
//!     let current_sensor_data: SensorDataV5 = rx.recv().unwrap();
//!     current_sensor_data.print_sensor_data();
//!     println!("{}", current_sensor_data.temperature_in_celcius());
//!     println!("{}", current_sensor_data.get_humidity());
//!     println!("{}", current_sensor_data.get_pressure());
//!     println!("{:?}", current_sensor_data.get_acceleration_in_mg());
//!     println!("{}", current_sensor_data.get_battery_voltage());
//!     println!("{}", current_sensor_data.get_tx_power());
//!     println!("{}", current_sensor_data.mac_as_str());
//! }
//! ```
pub mod bluetooth;
pub mod ruuvitag;
