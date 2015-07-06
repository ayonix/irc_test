extern crate regex;

use regex::Regex;
use std::net::TcpStream;
use std::io::Write;
use std::thread;
use std::io::{BufRead, BufReader,BufWriter};
use std::sync::Arc;
use std::str;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;

#[derive(Debug)]
pub struct Network {
    username: String,
    addr: &'static str,
    channels: Vec<&'static str>,
    reader: Option<BufReader<TcpStream>>,
    writer: Option<BufWriter<TcpStream>>,
}

impl Network {
    pub fn new(u: String, addr: &'static str, channels: Vec<&'static str>) -> Network {
        Network{username: u, addr: addr, reader: None, writer: None, channels: channels}
    }

    pub fn connect(&mut self) {
        let stream = match TcpStream::connect(&self.addr) {
            Ok(x) => x,
            Err(e) => {panic!("{}", e);},
        };

        println!("Connected to {}", self.addr);
        self.reader = match stream.try_clone() {
            Ok(x) => Some(BufReader::new(x)),
            Err(e) => {panic!("{}", e);},
        };
        self.writer = match stream.try_clone() {
            Ok(x) => Some(BufWriter::new(x)),
            Err(e) => {panic!("{}", e);},
        };

        let mut ping_reader = match stream.try_clone() {
            Ok(x) => BufReader::new(x),
            Err(e) => {panic!("{}", e);},
        };
        let mut ping_writer = match stream.try_clone() {
            Ok(x) => BufWriter::new(x),
            Err(e) => {panic!("{}", e);},
        };

        let net_addr = self.addr.clone();

        thread::spawn(move || {
            loop {
                let mut msg = &mut String::new();
                match ping_reader.read_line(&mut msg) {
                    Ok(_) => {
                        println!("{}//{}", net_addr, msg);
                        match Message::new(msg, net_addr.to_string()) {
                            Ok(x) => {
                                if x.code == "PING" {
                                    match ping_writer.write_all(format!("PONG {}", x.param).as_bytes()) {
                                        Ok(_) => println!("{}//PONG {}\r\n", net_addr, x.param),
                                        Err(e) => println!("Couldn't pong: {}", e),
                                    }
                                    ping_writer.flush();
                                }
                            },
                            Err(e) => {println!("error decoding message: {}", e);}
                        }
                    },
                    Err(e) => println!("Some error: {}", e),
                }
            }
        });

        let name = self.username.clone();
        self.send("", format!("NICK {}", name));
        self.send("", format!("USER {0} 8 * :{0}", name));
    }

    pub fn join(&mut self, channel: &'static str) {
        match self.writer {
            Some(ref mut s) => {s.write_all(format!("JOIN {:?}", channel).as_bytes());},
            None => {println!("Not connected to network {:?}", self);}
        }
    }

    pub fn send(&mut self, target: &str, msg: String) {
        match self.writer {
            Some(ref mut s) => {
                match s.write_all(format!("{}\r\n", msg).as_bytes()) {
                    Ok(_) => println!("{}", msg),
                    Err(e) => println!("Error while writing: {}", e),
                };
                s.flush();
            },
            None => { println!("Not connected to network {:?}", self); }
        }
    }
}

#[derive(Debug)]
pub struct Message {
    prefix: String,
    code: String,
    param: String,
    orig: String,
    network: String,
}

impl Message {
    pub fn new(msg: &str, network: String) -> Result<Message, &str> {
        let msg_re = Regex::new(r"^(:\S+)?\s*(\S+)\s+(.*)\r?").unwrap();
        match msg_re.captures(msg) {
            Some(x) => Ok(Message {
                prefix: x.at(1).unwrap_or("").to_string(),
                code: x.at(2).unwrap_or("").to_string(),
                param: x.at(3).unwrap_or("").to_string(),
                orig: msg.to_string(),
                network: network
            }),
            None => Err("Couldn't parse message"),
        }
    }
}

pub struct Client {
    networks: Vec<Network>,
    receiver: mpsc::Receiver<Message>,
    sender: mpsc::Sender<Message>
}

impl Client {
	pub fn new(networks: Vec<Network>) -> Client {
        let (tx, rx) = mpsc::channel();
		Client{
            networks: networks,
            receiver: rx, // for every network
            sender: tx, // for user input
        }
	}

    pub fn connect(&mut self) {
        println!("Connecting");
        for i in self.networks.iter_mut() {
            i.connect();
        }

        for m in self.receiver.iter() {
            println!("{:?}", m);
        }
    }

    pub fn disconnect() {
    }
}
