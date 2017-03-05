#[macro_use] extern crate nickel;
extern crate hyper;
extern crate rustc_serialize;
extern crate rand;
extern crate chrono;
// extern crate websocket;
extern crate ws;
extern crate env_logger;

use rustc_serialize::json::{Json, ToJson};
use rand::Rng;
use std::collections::BTreeMap;
use std::thread;
use std::str::from_utf8;
use std::fmt::{self, Debug};
use chrono::prelude::*;
// use hyper::header::{AccessControlAllowOrigin, AccessControlAllowHeaders};
use nickel::status::StatusCode;
use nickel::{Nickel, StaticFilesHandler, JsonBody, HttpRouter};
// use websocket::{Server, Message, Sender, Reciever};
// use websocket::message::Type;
// use websocket::header::WebSocketProtocol;
use ws::{connect, listen, CloseCode, Sender, Handler, Message, Result, Handshake};

struct TrafficData {
    data: i32,
    time: DateTime<UTC>,
}

impl ToJson for TrafficData {
    fn to_json(&self) -> Json {
        let mut map = BTreeMap::new();
        map.insert("data".to_string(), self.data.to_json());
        Json::Object(map)
    }
}

fn generate_traffic_data() -> Json {
    let mut rng = rand::thread_rng();
    let traffic_data = TrafficData {
        data: rng.gen_range::<i32>(0, 3000),
        time: UTC::now(),
    };
    traffic_data.to_json()
}


struct Server {
    out: Sender,
}

impl Handler for Server {

    fn on_open(&mut self, shake: ws::Handshake) -> ws::Result<()> {
        if let Some(ip_addr) = try!(shake.remote_addr()) {
            println!("Connection opened from {}.", ip_addr);
        } else {
            println!("Unable to obtain client's IP address.");
        }
        Ok(())
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        println!("Server got message {}", msg);
        self.out.send(msg)
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        println!("Websocket closing for ({:?}) {}", code, reason);
        self.out.shutdown().unwrap();
    }
}

fn main() {

    let mut nickel_server = Nickel::new();

    nickel_server.utilize(StaticFilesHandler::new("public"));

    nickel_server.listen("127.0.0.1:3333").unwrap();

    env_logger::init().unwrap();

    let server = thread::spawn(move || {
        listen("127.0.0.1:3012", |out| {

            Server { out: out }

        }).unwrap()
    });

    let client = thread::spawn(move || {

        connect("ws://127.0.0.1:3012", |out| {

            let data = generate_traffic_data().to_string();

            out.send(&*data).unwrap();

            move |msg| {
                println!("Client got message '{}'. ", msg);
                out.close(CloseCode::Normal)
            }

        }).unwrap()

    });

    let _ = server.join();
    let _ = client.join();

}
