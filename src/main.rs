use ruuviscanner::ruuvitag::{subscribe_ruuvitag, SensorDataV5};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<(dyn Error + 'static)>> {
    // All of the ruuvitags I have.
    // let _ruuvitags = vec![
    //     "C0:CB:4E:3D:3E:12".to_owned(),
    //     "E1:16:22:5D:F6:C9".to_owned(),
    //     "CC:6F:70:EE:4C:AD".to_owned(),
    // ];
    let mac = "CC:6F:70:EE:4C:AD";
    let rx = subscribe_ruuvitag(&mac).await?;
    loop {
        let current_sensor_data: SensorDataV5 = rx.recv().unwrap();
        current_sensor_data.print_sensor_data();
        println!("{}", current_sensor_data.temperature_in_celcius());
        println!("{}", current_sensor_data.get_humidity());
        println!("{}", current_sensor_data.get_pressure());
        println!("{:?}", current_sensor_data.get_acceleration_in_mg());
        println!("{}", current_sensor_data.get_battery_voltage());
        println!("{}", current_sensor_data.get_tx_power());
        println!("{}", current_sensor_data.mac_as_str());
    }
}
