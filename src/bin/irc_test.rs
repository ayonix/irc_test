extern crate irc_test;
use irc_test::*;

fn main() {
    let n1 = Network::new("derpderpderp".to_string(), "irc.hackint.org:6667", vec!["#derptest"]);
    let mut c = Client::new(vec![n1]);
    c.connect();
}
