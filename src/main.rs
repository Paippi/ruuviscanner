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
        let msg: SensorDataV5 = rx.recv().unwrap();
        msg.print_sensor_data();
    }
}
