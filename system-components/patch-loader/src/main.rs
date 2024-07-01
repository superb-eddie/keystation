use std::{env, fs};
use std::net::UdpSocket;
use std::path::Path;

use rosc::{encoder, OscMessage, OscPacket, OscType};

const PATCH_DIRECTORY: &'static str = "/usr/share/patches";

const CARDINAL_ADDRESS: &'static str = "localhost:2228";

fn send_message(socket: &UdpSocket, addr: impl Into<String>, args: Vec<OscType>) {
    let msg_buf = encoder::encode(&OscPacket::Message(OscMessage {
        addr: addr.into(),
        args,
    }))
    .expect("Can't encode message");

    socket
        .send_to(&msg_buf, CARDINAL_ADDRESS)
        .expect("Can't send message");
}

fn recv_message(socket: &UdpSocket, addr: impl Into<String>) -> Vec<OscType> {
    let mut buf = [0u8; rosc::decoder::MTU];
    match socket.recv_from(&mut buf) {
        Ok((size, _)) => {
            let (_, packet) = rosc::decoder::decode_udp(&buf[..size]).unwrap();
            match packet {
                OscPacket::Message(msg) => {
                    assert_eq!(
                        msg.addr,
                        addr.into(),
                        "Address of received message didn't match"
                    );

                    return msg.args;
                }
                OscPacket::Bundle(_) => {
                    panic!("Received a bundle, not a message!")
                }
            }
        }
        Err(e) => {
            panic!("Error receiving from socket: {}", e);
        }
    }
}

fn main() {
    let sock = UdpSocket::bind("localhost:0").expect("Can't bind udp socket");

    // Make sure cardinal is up
    send_message(&sock, "/hello", vec![]);
    println!("{:?}", recv_message(&sock, "/resp"));

    // Load patch from disk
    let args: Vec<String> = env::args().collect();
    assert_eq!(args.len(), 2, "expected one argument");

    let patch_dir = Path::new(PATCH_DIRECTORY);
    assert!(patch_dir.exists(), "Patch dir doesn't exist");

    let patch_path = patch_dir.join(format!("{}.vcv", args[1]));
    assert!(patch_path.exists(), "Patch doesn't exist");

    let patch_contents = fs::read(&patch_path).expect("Can't read patch");

    send_message(&sock, "/load", vec![OscType::Blob(patch_contents)]);
    println!("{:?}", recv_message(&sock, "/resp"));
}
