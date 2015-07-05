extern crate regex;

use regex::Regex;
use std::net::TcpStream;
use std::io::Write;
use std::thread;
use std::io::{BufRead, BufReader,BufWriter};
use std::sync::Arc;
use std::str;

#[derive(Debug)]
pub struct Network {
    username: String,
    addr: &'static str,
    channels: Vec<&'static str>,
    reader: Option<BufReader<TcpStream>>,
    writer: Option<BufWriter<TcpStream>>
}

impl Network {
    pub fn new(u: String, addr: &'static str, channels: Vec<&'static str>) -> Network {
        Network{username: u, addr: addr, reader: None, writer: None, channels: channels}
    }

    pub fn connect(&mut self) {
        let stream = TcpStream::connect(&self.addr).unwrap();
        self.reader = Some(BufReader::new(stream.try_clone().unwrap()));
        self.writer = Some(BufWriter::new(stream.try_clone().unwrap()));

        let mut ping_reader = BufReader::new(stream.try_clone().unwrap());
        let mut ping_writer = BufWriter::new(stream.try_clone().unwrap());

        thread::spawn(move || {
            loop {
                let mut msg: Vec<u8> = vec![];
                match ping_reader.read_until(b'\n', &mut msg) {
                    Ok(m) => {
                        let tmp = String::from_utf8(msg).unwrap();
                        println!("got: {}", tmp);
                        match Message::new(&tmp) {
                            Ok(x) => {
                                if x.msg_type == "PING" {
                                    ping_writer.write_all(format!("PONG {}", x.msg).as_bytes());
                                }
                            },
                            Err(e) => {println!("{}", e);},
                        }
                    },
                    Err(e) => println!("error reading message: {}", e),
                };
            }
        });

        let name = self.username.clone();
        self.send("", format!("NICK {}", name));
        self.send("", format!("USER {0} {0} {0} :{0}", name));
    }

    pub fn join(&mut self, channel: &'static str) {
        match self.writer {
            Some(ref mut s) => {s.write_all(format!("JOIN {:?}", channel).as_bytes());},
            None => {println!("Not connected to network {:?}", self);}
        }
    }

    pub fn send(&mut self, target: &str, msg: String) {
        self.writer.as_mut().unwrap().write_all(msg.as_bytes());
    }
}

pub struct Message<'a> {
    sender: &'a str,
    msg_type: &'a str,
    target: &'a str,
    msg: &'a str,
    orig: &'a str,
}

impl<'a> Message<'a> {
    pub fn new(msg: &'a str) -> Result<Message<'a>, &'a str> {
        let msg_re = Regex::new(r"^(?:[:](\S+) )?(\S+)(?: (?!:)(.+?))?(?: [:](.+))?$").unwrap();
        match msg_re.captures(msg) {
            Some(x) => Ok(Message {
                sender: x.at(1).unwrap_or(""),
                msg_type: x.at(2).unwrap_or(""),
                target: x.at(3).unwrap_or(""),
                msg: x.at(4).unwrap_or(""),
                orig: x.at(0).unwrap_or("")
            }),
            None => Err("Not a valid message")
        }
    }
}

pub struct Client {
    networks: Vec<Network>,
}

impl Client {
	pub fn new(networks: Vec<Network>) -> Client {
		Client{networks: networks}
	}

    pub fn connect(&mut self) {
        println!("Connecting");
        for i in self.networks.iter_mut() {
            i.connect();
        }
    }

    pub fn disconnect() {
    }
}

#[test]
fn it_works() {
    let n1 = Network::new("derpderpderp".to_string(), "irc.hackint.org:6667", vec!["#derptest"]);
    let mut c = Client::new(vec![n1]);
    c.connect();
}
