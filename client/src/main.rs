use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::net::{SocketAddr, UdpSocket};
use std::path::Path;
use std::str;
use std::thread;

use client::*;
use wa_fsp::*;

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

        self.send_reg();

        // create a separate thread listening to incomming messages
        let mut buffer: Vec<u8> = Vec::new();
        buffer.resize(BUF_SIZE, 0);
        let c_socket = self.socket.try_clone().unwrap();
        thread::spawn(move || loop {
            if let Ok(_) = c_socket.recv_from(&mut buffer) {}
        });

        loop {}
    }

    fn send_reg(&self) {
        let filenames = self
            .files
            .keys()
            .map(|k| k.clone())
            .collect::<Vec<String>>();

        let msg = serde_json::to_string(&Message {
            msg_type: MsgType::Register,
            content: serde_json::to_string(&filenames).unwrap(),
        })
        .unwrap();

        println!("{:?}", msg);

        self.socket
            .send(msg.as_bytes())
            .expect("Could not send to server");
    }
}

fn main() {
    // create client with server config and files
    let mut client = FspClient::new();

    // create UDP socket and connect to server
    client.run();
}
