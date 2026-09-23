fn main() {
    use cameleon::u3v;
    
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
    
    // read and write feature tests
    let mut params_ctxt = camera.params_ctxt()?;

    let gain_node = params_ctxt.node("Gain")?.as_float(&params_ctxt)?;
    if gain_node.is_readable(&mut params_ctxt)? {
        let value = gain_node.valid(&mut params_ctxt)?;
        println!("{}", value);
    }

    if gain_node.is_writable(&mut params_ctxt)? {
        gain_node.set_value(&mut params_ctxt, 0.1)?;
    }


    // Start streaming. Channel capacity is set to 3.
    let payload_rx = camera.start_streaming(3).unwrap();
    
    let mut payload_count = 0;
    while payload_count < 10 {
        match payload_rx.try_recv() {
            Ok(payload) => {
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
                payload_count += 1;
    
                // Send back payload to streaming loop to reuse the buffer. This is optional.
                payload_rx.send_back(payload);
            }
            Err(_err) => {
                continue;
            }
        }
    }
    
    // Closes the camera.
    camera.close().unwrap();
}