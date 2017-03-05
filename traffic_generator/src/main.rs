#[macro_use] extern crate nickel;
extern crate hyper;
extern crate rustc_serialize;
extern crate rand;
// extern crate websocket;
extern crate ws;
extern crate env_logger;

use rustc_serialize::json::{Json, ToJson, decode, encode};
use rand::Rng;
use std::collections::BTreeMap;
use std::fmt::Display;
// use hyper::header::{AccessControlAllowOrigin, AccessControlAllowHeaders};
use nickel::status::StatusCode;
use nickel::{Nickel, StaticFilesHandler, JsonBody, HttpRouter};
// use websocket::{Server, Message, Sender, Reciever};
// use websocket::message::Type;
// use websocket::header::WebSocketProtocol;
use ws::{connect, listen, Handler, Sender, Handshake, Result, Message, CloseCode};

#[derive(RustcDecodable, RustcEncodable)]
struct TrafficData {
    data: i32,
}

impl ToJson for TrafficData {
    fn to_json(&self) -> Json {
        let mut map = BTreeMap::new();
        map.insert("data".to_string(), self.data.to_json());
        Json::Object(map)
    }
}

fn generateTrafficData() -> Json {
    let mut rng = rand::thread_rng();
    let traffic_data = TrafficData {
        data: rng.gen_range::<i32>(0, 3000)
    };
    traffic_data.to_json()
}

fn main() {

    env_logger::init().unwrap();

    let mut server = Nickel::new();

    server.utilize(StaticFilesHandler::new("public"));

    server.get("/", middleware! { |request, response|
        let traffic_data = try_with!(response, {
            request.json_as::<TrafficData>().map_err(|e| (StatusCode::BadRequest, e))
        });
        format!("{}", traffic_data.data);
    });

    server.get("/traffic", middleware! { |_, mut res|
        generateTrafficData()
    });

    if let Err(error) = connect("127.0.0.1:6767", |out| {

        if let Err(_) = out.send("Hello.") {
            println!("Websocket failed @ initial message");
        } else {
            println!("Client sent message");
        }

        move |msg| {
            println!("Client recieved: {}", msg);
            out.close(CloseCode::Normal)
        }
    }) {
        println!("Failed to create Websocket due to: {:?}", error);
    }


    server.listen("127.0.0.1:6767").unwrap();
}
