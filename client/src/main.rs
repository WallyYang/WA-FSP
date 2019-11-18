use std::collections::HashMap;
use std::fs::{self, DirEntry, File};
use std::io::Read;
use std::net::{SocketAddr, UdpSocket};
use std::path::Path;
use std::{io, str};

use client::Server;

struct Session {
    server: Server,
    files: HashMap<String, fs::File>,
}

fn main() {
    // create session with server config and files
    let mut session = Session {
        server: Server::from_file("config.yaml"),
        files: HashMap::new(),
    };
    for entry in fs::read_dir(Path::new("./files/")).unwrap() {
        let entry = entry.unwrap();
        session.files.insert(
            entry.file_name().into_string().unwrap(),
            fs::File::open(entry.path()).unwrap(),
        );
    }

    // create UDP socket and connect to server
    let socket = UdpSocket::bind("127.0.0.1:8000")
        .expect("Could not bind client socket");
    socket
        .connect(SocketAddr::new(session.server.address, session.server.port))
        .expect("Could not connect to server");

    for mut file in session.files {
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
        socket
            .send(input.as_bytes())
            .expect("Failed to write to server");

        socket
            .recv_from(&mut buffer)
            .expect("Could not read into buffer");
        print!(
            "{}",
            str::from_utf8(&buffer).expect("Could not write buffer as string")
        );
    }
}
