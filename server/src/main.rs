use std::net::UdpSocket;
use std::str;
use std::thread;

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:8080").expect("Cannot bind socket");

    loop {
        let mut buf = [0u8; 1500];
        let sock = socket.try_clone().expect("Failed to clone socket");
        match socket.recv_from(&mut buf) {
            Ok((_, src)) => {
                thread::spawn(move || {
                    println!("Handling connection from {}", src);
                    println!("{:?}", str::from_utf8(&buf));
                    sock.send_to(&buf, src).expect("Failed to send a response");
                });
            }
            Err(e) => {
                eprintln!("Couldn't receive a datagram: {}", e);
            }
        }
    }
}
