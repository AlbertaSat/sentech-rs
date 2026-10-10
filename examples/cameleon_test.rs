
use std::time::Duration;
use sentech_rs::cameleon_api::CameleonApi;
use std::time::{SystemTime, UNIX_EPOCH};
use std::path::PathBuf;

fn main(){
    // start up cam, try to read GainAuto, try to write GainAuto, then try to take 2 pictures
    println!("Starting Cameleon API test...");
    let mut cam_api = CameleonApi::initialize().unwrap();
    println!("Camera initialized;");
    // let gain_auto = cam_api.read_int_feature("GainAuto").unwrap();
    // println!("GainAuto: {}", gain_auto);
    // // cam_api.write_int_feature("GainAuto", 1).unwrap();
    // let gain_auto = cam_api.read_int_feature("GainAuto").unwrap();
    // println!("GainAuto: {}", gain_auto);

    // let pixel_format = cam_api.read_string_feature("PixelFormat").unwrap();
    // println!("PixelFormat: {}", pixel_format);
    let width = cam_api.read_int_feature("Width").unwrap();
    let height = cam_api.read_int_feature("Height").unwrap();
    println!("Width: {}, Height: {}", width, height);
    let firmware_version = cam_api.read_string_feature("DeviceFirmwareVersion").unwrap();
    println!("DeviceFirmwareVersion: {}", firmware_version);
    let serial_number = cam_api.read_string_feature("DeviceSerialNumber").unwrap();
    println!("DeviceSerialNumber: {}", serial_number);


    cam_api.start_streaming(3).unwrap();
    
    let frame1 = cam_api.recieve_frame(Duration::from_secs(1)).unwrap();
    let frame2 = cam_api.recieve_frame(Duration::from_secs(1)).unwrap();
    println!("Frame 1: id: {}, timestamp: {:?}", frame1.id(), frame1.timestamp());
    println!("Frame 2: id: {}, timestamp: {:?}", frame2.id(), frame2.timestamp());
    cam_api.stop_streaming().unwrap();  
    cam_api.camera.close().unwrap(); // gotta close that camera!
    //save path as capture_{SystemTime.now()}.jpeg
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let path1 = PathBuf::from(format!("vimba_capture1_{}.jpeg", ts));
    let path2 = PathBuf::from(format!("vimba_capture2_{}.jpeg", ts));
    println!("Saving images to {} and {}", path1.display(), path2.display());
    cam_api.save_image(&frame1, &path1, "jpeg");    
    cam_api.save_image(&frame2, &path2, "jpeg");
    println!("Done!");
}