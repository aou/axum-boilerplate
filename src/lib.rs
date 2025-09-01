use dotenvy::dotenv;

pub mod webapp;

pub fn get_config() {
    match dotenv() {
        Ok(path) => println!("Path: {:#?}", path),
        Err(_) => println!("No .env found, continuing..."),
    }
}
