use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::net::{SocketAddr, UdpSocket};
use std::path::Path;
use std::{io, str};

use client::Server;

struct FspClient {
    socket: UdpSocket,
    files: HashMap<String, fs::File>,
}

impl FspClient {
    fn new() -> FspClient {
        let mut files = HashMap::new();
        for entry in fs::read_dir(Path::new("./files/")).unwrap() {
            let entry = entry.unwrap();
            files.insert(
                entry.file_name().into_string().unwrap(),
                fs::File::open(entry.path()).unwrap(),
            );
        }

        let server = Server::from_file("config.yaml");
        let socket = UdpSocket::bind("127.0.0.1:8000")
            .expect("Could not bind client socket");
        socket
            .connect(SocketAddr::new(server.address, server.port))
            .expect("Could not connect to server");

        FspClient { socket, files }
    }

    fn run(&mut self) {
        for mut file in &self.files {
            println!("{}", file.0);
            let mut content = String::new();
            file.1.read_to_string(&mut content).unwrap();
            println!("{}", content);
        }

        loop {
            let mut input = String::new();
            let mut buffer = [0u8; 1500];
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read from stdin");
            self.socket
                .send(input.as_bytes())
                .expect("Failed to write to server");

            self.socket
                .recv_from(&mut buffer)
                .expect("Could not read into buffer");
            print!(
                "{}",
                str::from_utf8(&buffer)
                    .expect("Could not write buffer as string")
            );
        }
    }
}

fn main() {
    // create client with server config and files
    let mut client = FspClient::new();

    // create UDP socket and connect to server
    client.run();
}
