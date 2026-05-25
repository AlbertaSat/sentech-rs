use sentech_rs::api::{SentechApi, SystemHandle};


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

    println!(
        "Successfully initialized API, created system, and updated interfaces! Re-evaluated: {}",
        reval
    );

}