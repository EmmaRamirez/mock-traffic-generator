extern crate hyper;
extern crate rustc_serialize;
extern crate rand;
extern crate chrono;
extern crate ws;
extern crate env_logger;

use rustc_serialize::json::{Json, ToJson};
use rand::Rng;
use std::collections::BTreeMap;
use std::thread;
use std::str::from_utf8;
use std::fmt::{self, Debug};
use chrono::prelude::*;
use ws::{connect, listen, CloseCode, Sender, Handler, Message, Result, Handshake};

// We create a struct for TrafficData that accepts a rate and time
#[derive(Debug, RustcEncodable, RustcDecodable)]
struct TrafficData {
    rate: i32,
    time: DateTime<UTC>,
}

// We implement a .to_json() method that create a tree from the struct
// then we map the values and return Json from the Object
impl ToJson for TrafficData {
    fn to_json(&self) -> Json {
        let mut map = BTreeMap::new();
        map.insert("rate".to_string(), self.rate.to_json());
        map.insert("time".to_string(), self.time.to_json());
        Json::Object(map)
    }
}

// With our implementation in place, we can now use a function
// that returns traffiCdata json; we generate a random number
// between 0 and 3000, then call the current time (via chrono)
fn generate_traffic_data() -> Json {
    let mut rng = rand::thread_rng();
    let traffic_data = TrafficData {
        rate: rng.gen_range::<i32>(0, 3000),
        time: UTC::now(),
    };
    traffic_data.to_json()
}

// our Server struct's out is of type Sender, which is the output of
// the WebSocket connection.
// https://ws-rs.org/api_docs/ws/struct.Sender.html
struct Server {
    out: Sender,
}

// Here we use the Handler trait for our Server, which provides the main
// socket functionality.
impl Handler for Server {

    // on_open trys a hanshake and prints the result, but will always Result
    // in an OK(()),
    fn on_open(&mut self, shake: ws::Handshake) -> ws::Result<()> {
        if let Some(ip_addr) = try!(shake.remote_addr()) {
            println!("Connection opened from {}.", ip_addr);
        } else {
            println!("Unable to obtain client's IP address.");
        }
        Ok(())
    }

    // on_message prints when we recieve a message
    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        println!("Server got message {}", msg);
        self.out.send(msg)
    }

    // on_close shuts down the socket when we close it
    fn on_close(&mut self, code: CloseCode, reason: &str) {
        println!("Websocket closing for ({:?}) {}", code, reason);
        self.out.shutdown().unwrap();
    }

}

fn main() {

    // initialize the logger
    env_logger::init().unwrap();

    // this is our server. We spawn a thread that listens to the address
    // And returns a Server struct. Recall that out is of type Sender,
    // i.e. the Socket connection
    let server = thread::spawn(move || {
        listen("127.0.0.1:3012", |out| {
            Server { out: out }
        }).unwrap()
    });

    // Likewise, we do this with the client
    let client = thread::spawn(move || {

        connect("ws://127.0.0.1:3012", |out| {

            let data = generate_traffic_data().to_string();

            // &* is an explicit conversion to a String type
            // that we use alongside .to_string() to ensure
            // it works with send(), which requires a &str type
            // https://doc.rust-lang.org/book/strings.html
            out.send(&*data).unwrap();

            // We log the data then close the connection
            move |msg| {
                println!("Client got message '{}'. ", msg);
                out.close(CloseCode::Normal)
            }

        }).unwrap()

    });

    // Wait for the threads to finish what they're doing
    let _ = server.join();
    let _ = client.join();

}
