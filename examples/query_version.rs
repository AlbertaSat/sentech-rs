use vimba_rs::api::vmb_version_query;

fn main() {
    match vmb_version_query() {
        Ok(ver) => println!("{ver:?}"),
        Err(e) => eprintln!("Failed to query vmb version: {e}"),
    }
}
