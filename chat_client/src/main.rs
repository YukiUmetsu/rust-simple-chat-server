use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 32;

fn main() {
    let mut client = TcpStream::connect(LOCAL).expect("stream failed to connect");
    client.set_nonblocking(true).expect("fialed to initialize non-blocking");

    let (tx, rx) = mpsc::channel::<String>();
    thread::spawn(move || loop {
        // Read message:
        let mut buff = vec![0; MSG_SIZE];
        match client.read_exact(&mut buff) {
            Ok(_) => {
                let msg_byte_vec = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                let msg = String::from_utf8(msg_byte_vec).expect("invalid utf8 message");
                println!("message received {:?}", msg);
            },
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("connection with server was served");
                break;
            }
        }

        // Receive message from channel and Write message to the server
        match rx.try_recv() {
            Ok(msg) => {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).expect("writing to socket failed");
                println!("message sent {:?}", msg);
            },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break
        }

        thread::sleep(Duration::from_millis(100));
    });

    println!("what is your name?");
    let mut name_buff = String::new();
    io::stdin().read_line(&mut name_buff).expect("Reading from stdin failed");
    let name = name_buff.trim().to_string();

    loop {
        println!("write a message: ");
        let mut buff = String::new();
        io::stdin().read_line(&mut buff).expect("Reading from stdin failed");
        let msg = format!("{}{}{}", &name, &String::from(": "), &buff.trim().to_string());
        if msg == ":quit" || tx.send(msg).is_err() { break }
    }
    println!("bye");
}
