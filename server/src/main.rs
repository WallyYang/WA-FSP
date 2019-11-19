use std::collections::{HashMap, HashSet};
use std::net::{SocketAddr, UdpSocket};
use std::str;

use wa_fsp::*;

struct FspServer {
    socket: UdpSocket,
    files: HashMap<String, HashSet<SocketAddr>>,
}

impl FspServer {
    fn new() -> FspServer {
        let socket =
            UdpSocket::bind("0.0.0.0:8080").expect("Cannot bind socket");

        FspServer {
            socket,
            files: HashMap::new(),
        }
    }

    fn run(&mut self) {
        let mut buffer: Vec<u8> = Vec::new();
        buffer.resize(BUF_SIZE, 0);

        loop {
            match self.socket.recv_from(&mut buffer) {
                Ok((bytes_read, src)) => {
                    // println!("{}", str::from_utf8(&buffer).unwrap());
                    let msg: Message = serde_json::from_str(
                        str::from_utf8(&buffer[..bytes_read]).unwrap(),
                    )
                    .expect("Error parsing message");
                    match msg.msg_type {
                        MsgType::Register => self.register(src, &msg),
                        MsgType::List => self.list(src),
                        MsgType::File => {}
                    }

                    // thread::spawn(move || {
                    // println!("Handling connection from {}", src);
                    // });
                }
                Err(e) => {
                    eprintln!("Couldn't receive a datagram: {}", e);
                }
            }

            buffer.clear();
            buffer.resize(BUF_SIZE, 0);
        }
    }

    fn register(&mut self, socket_addr: SocketAddr, msg: &Message) {
        println!("Registering files from UDP");

        let filenames: Vec<String> = serde_json::from_str(&msg.content)
            .expect("Unable to parse file name list");

        for filename in filenames {
            // create new set
            if !self.files.contains_key(&filename) {
                self.files.insert(filename.clone(), HashSet::new());
            }

            self.files.entry(filename).and_modify(|v| {
                v.insert(socket_addr);
            });
        }
    }

    fn list(&self, socket_addr: SocketAddr) {
        println!("Sending file list to UDP {}", socket_addr);

        let filenames = self
            .files
            .keys()
            .map(|k| k.clone())
            .collect::<Vec<String>>();

        let msg = serde_json::to_string(&Message {
            msg_type: MsgType::List,
            content: serde_json::to_string(&filenames).unwrap(),
        })
        .unwrap();
        println!("{}", msg);

        let bytes_write = self
            .socket
            .send_to(&msg.as_bytes(), socket_addr)
            .expect("Cannot sent to client");
        println!("{} bytes written", bytes_write);
    }
}

fn main() {
    // create server and initialize connections
    let mut server = FspServer::new();

    server.run();
}
