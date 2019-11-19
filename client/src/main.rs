use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::net::{SocketAddr, UdpSocket};
use std::path::Path;
use std::str::{self, FromStr};
use std::thread;

use client::*;
use wa_fsp::*;

struct FspClient {
    socket: UdpSocket,
    server: SocketAddr,
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
        let server = SocketAddr::new(server.address, server.port);
        let socket =
            UdpSocket::bind("0.0.0.0:0").expect("Could not bind client socket");

        FspClient {
            socket,
            server,
            files,
        }
    }

    fn run(&mut self) {
        // register files with server
        self.send_reg();

        // create a separate thread listening to incomming messages
        let mut buffer: Vec<u8> = Vec::new();
        buffer.resize(BUF_SIZE, 0);
        let c_socket = self.socket.try_clone().unwrap();
        thread::spawn(move || loop {
            if let Ok((bytes_read, src)) = c_socket.recv_from(&mut buffer) {
                let msg: Message = serde_json::from_str(
                    str::from_utf8(&buffer[..bytes_read]).unwrap(),
                )
                .expect("Error parsing message");

                match msg.msg_type {
                    MsgType::List => {
                        println!("Files registered with the server: ");
                        let filenames: Vec<String> =
                            serde_json::from_str(&msg.content)
                                .expect("Error parsing filenames");
                        for filename in filenames {
                            println!("{}", filename);
                        }
                    }
                    MsgType::FileResp => {
                        println!("\nReceived list from the server");
                        FspClient::handle_file_resp(&c_socket, &msg);
                    }
                    MsgType::FileReq => {
                        println!("\nProcessing request from peer");
                        FspClient::handle_file_req(
                            &c_socket,
                            &msg.content,
                            src,
                        );
                    }
                    MsgType::FileTrans => {
                        println!("\nProcessing file transmission");
                        FspClient::handle_file_trans(&msg);
                    }
                    _ => {}
                }
            }
        });

        loop {
            print!("> ");
            io::stdout().flush().unwrap();

            let mut msg = String::new();
            io::stdin().read_line(&mut msg).unwrap();
            let msg = msg.trim();

            if msg.len() > 0 {
                if msg == ":l" {
                    self.req_list();
                } else {
                    self.req_file(&String::from_str(msg).unwrap());

                    // update file entries and send to server
                    for entry in fs::read_dir(Path::new("./files/")).unwrap() {
                        let entry = entry.unwrap();
                        self.files.insert(
                            entry.file_name().into_string().unwrap(),
                            fs::File::open(entry.path()).unwrap(),
                        );
                    }
                    self.send_reg();
                }
            }
        }
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

        self.socket
            .send_to(msg.as_bytes(), self.server)
            .expect("Could not send to server");
    }

    fn req_list(&self) {
        let msg = serde_json::to_string(&Message {
            msg_type: MsgType::List,
            content: String::new(),
        })
        .unwrap();

        self.socket
            .send_to(msg.as_bytes(), self.server)
            .expect("Could not send to server");
    }

    fn req_file(&self, filename: &String) {
        if self.files.contains_key(filename) {
            println!("File already exists locally!");
            return;
        }

        println!("Requesting file");
        let msg = serde_json::to_string(&Message {
            msg_type: MsgType::FileReq,
            content: filename.clone(),
        })
        .unwrap();

        self.socket
            .send_to(msg.as_bytes(), self.server)
            .expect("Could not send to server");
    }

    fn handle_file_resp(socket: &UdpSocket, msg: &Message) {
        let (filename, clients): (String, HashSet<SocketAddr>) =
            serde_json::from_str(&msg.content)
                .expect("Cannot parse server response");

        if clients.is_empty() {
            println!("File not found!");
            return;
        }

        let msg = serde_json::to_string(&Message {
            msg_type: MsgType::FileReq,
            content: filename,
        })
        .unwrap();

        for client in clients {
            match socket.send_to(msg.as_bytes(), client) {
                Ok(_) => {
                    println!("Sending request to {}", client);
                    break;
                }
                Err(_) => {}
            }
        }
    }

    fn handle_file_req(socket: &UdpSocket, filename: &String, src: SocketAddr) {
        let path = format!("{}{}", "./files/", filename);
        let path = Path::new(&path);

        let mut file = File::open(path).expect("Unable to open file");

        let mut buffer = String::new();
        file.read_to_string(&mut buffer).unwrap();

        let msg = serde_json::to_string(&Message {
            msg_type: MsgType::FileTrans,
            content: serde_json::to_string(&(filename.clone(), buffer))
                .unwrap(),
        })
        .unwrap();

        socket
            .send_to(msg.as_bytes(), src)
            .expect("Unable to send to requesting client");
    }

    fn handle_file_trans(msg: &Message) {
        let (filename, content): (String, String) =
            serde_json::from_str(&msg.content)
                .expect("Unable to parse file transmission");
        let path = format!("{}{}", "./files/", filename);

        let mut file =
            File::create(Path::new(&path)).expect("Unable to create file");
        file.write_all(content.as_bytes())
            .expect("Unable to write to files");

        println!("Files successfuly transmitted");
    }
}

fn main() {
    // create client with server config and files
    let mut client = FspClient::new();

    client.run();
}
