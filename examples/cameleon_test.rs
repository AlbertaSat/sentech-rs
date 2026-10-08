
use std::time::Duration;
use sentech_rs::cameleon_api::CameleonApi;

fn main(){
    // start up cam, try to read GainAuto, try to write GainAuto, then try to take 2 pictures
    println!("Starting Cameleon API test...");
    let mut cam_api = CameleonApi::initialize().unwrap();
    let gain_auto = cam_api.read_int_feature("GainAuto").unwrap();
    println!("GainAuto: {}", gain_auto);
    cam_api.write_int_feature("GainAuto", 1).unwrap();
    let gain_auto = cam_api.read_int_feature("GainAuto").unwrap();
    println!("GainAuto: {}", gain_auto);

    cam_api.start_streaming(3).unwrap();
    
    let frame1 = cam_api.recieve_frame(Duration::from_secs(1)).unwrap();
    let frame2 = cam_api.recieve_frame(Duration::from_secs(1)).unwrap();
    println!("Frame 1: id: {}, timestamp: {:?}", frame1.id(), frame1.timestamp());
    println!("Frame 2: id: {}, timestamp: {:?}", frame2.id(), frame2.timestamp());
    cam_api.stop_streaming().unwrap();  
    cam_api.camera.close().unwrap(); // gotta close that camera!
}