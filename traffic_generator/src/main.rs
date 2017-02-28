#[macro_use] extern crate nickel;
extern crate rustc_serialize;

use rustc_serialize::json;
use std::io::Read;
use std::sync::{Arc, Mutex};
use nickel::Nickel;

fn generator() -> &'static str {
    "It works."
}

fn main() {
    let mut server = Nickel::new();

    server.utilize(router! {
        get "/" => |_req, _res| {
            generator()
        }
    });

    server.listen("127.0.0.1:6767");
}
