extern crate irc_test;
use irc_test::*;

fn main() {
    let n1 = Network::new("derpderpderp".to_string(), "chat.freenode.net:6666", vec!["#derptest"]);
    let mut c = Client::new(vec![n1]);
    c.connect();
}
