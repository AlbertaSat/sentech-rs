use sentech_rs::api::{DeviceAccess, InterfaceHandle, SentechApi, SystemHandle};


// ============================================================================
// Start Sentech Camera
// ============================================================================

fn main() {
    let api: SentechApi = match SentechApi::initialize() {
        Ok(api) => api,
        Err(err) => {
            eprintln!("Failed to initialize API: {:?}", err);
            return;
        }
    };

    let system: SystemHandle = match api.create_system() {
        Ok(system) => system,
        Err(err) => {
            eprintln!("Failed to create system: {:?}", err);
            return;
        }
    };

    let reval = match system.update_interface_list() {
        Ok(reval) => reval,
        Err(e) => {
            eprintln!("Failed to update interface list: {:?}", e);
            return;
        }
    };

    let interface_count = match system.get_interface_count() {
        Ok(count) => count,
        Err(e) => {
            eprintln!("Failed to get interface count: {:?}", e);
            return;
        }
    };

    if interface_count == 0 {
        eprintln!("No interfaces found.");
        return;
    }

    let interface: InterfaceHandle = match system.get_interface(0) { // Zero means get the first interface
        Ok(interface) => interface,
        Err(e) => {
            eprintln!("Failed to get interface: {:?}", e);
            return;
        }
    };

    let access = DeviceAccess::Control;

    let available = match interface.device_available(0, access) { // check if the first device is available
        Ok(available) => available,
        Err(e) => {
            eprintln!("Failed to check device availability: {:?}", e);
            return;
        }
    };

    let dev = match system.create_first_ist_device(access) {
        Ok(dev) => dev,
        Err(e) => {
            eprintln!("Failed to create first device: {:?}", e);
            return;
        }
    };

    println!(
        "Successfully initialized API, created system, and updated interfaces! Re-evaluated: {}",
        reval
    );
    println!("Device available: {}", available);
    drop(dev);

}