use std::error::Error;
use std::fs::*;
use std::io::Read;
use std::net::IpAddr;
use std::path::Path;

extern crate serde;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Server {
    pub name: String,
    pub address: IpAddr,
    pub port: u16,
}

impl Server {
    pub fn from_file(filename: &str) -> Server {
        let path = Path::new(filename);

        let mut file = match File::open(&path) {
            Ok(file) => file,
            Err(e) => panic!(
                "Could not open config file: {}, {}",
                path.display(),
                e.description()
            ),
        };

        let mut content = String::new();
        match file.read_to_string(&mut content) {
            Ok(_) => (),
            Err(e) => panic!(
                "Could not read config file: {}, {}",
                path.display(),
                e.description(),
            ),
        }

        let server = match serde_yaml::from_str(&content) {
            Ok(server) => server,
            Err(e) => panic!("Error parsing config file: {}", e.description(),),
        };
        server
    }
}
