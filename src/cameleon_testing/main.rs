use cameleon::u3v;

// this is code copy and pasted from the cameleon example!
// ===================
// Example code

fn main() {
    // Enumerates all cameras connected to the host.
    let mut cameras = u3v::enumerate_cameras().unwrap();
    
    if cameras.is_empty() {
        println!("no camera found");
        return;
    }
    
    
    let mut camera = cameras.pop().unwrap();
    
    // Opens the camera.
    camera.open().unwrap();
    // Loads `GenApi` context. This is necessary for streaming.
    camera.load_context().unwrap();
    
    // Start streaming. Channel capacity is set to 3.
    let payload_rx = camera.start_streaming(3).unwrap();
    
    for _ in 0..10 {
        let payload = match payload_rx.recv_blocking() {
            Ok(payload) => payload,
            Err(e) => {
                println!("payload receive error: {e}");
                continue;
            }
        };
        println!(
            "payload received! block_id: {:?}, timestamp: {:?}",
            payload.id(),
            payload.timestamp()
        );
        if let Some(image_info) = payload.image_info() {
            println!("{:?}\n", image_info);
            let image = payload.image();
            // do something with the image.
            // ...
        }
    
        // Send back payload to streaming loop to reuse the buffer. This is optional.
        payload_rx.send_back(payload);
    }
    
    // Closes the camera.
    camera.close().unwrap();
}