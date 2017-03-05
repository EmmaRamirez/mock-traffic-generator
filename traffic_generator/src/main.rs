#[macro_use] extern crate nickel;
extern crate hyper;
extern crate rustc_serialize;
extern crate rand;
extern crate websocket;

use rustc_serialize::json::{Json, ToJson};
use rand::Rng;
use std::collections::BTreeMap;
// use hyper::header::{AccessControlAllowOrigin, AccessControlAllowHeaders};
use nickel::status::StatusCode;
use nickel::{Nickel, StaticFilesHandler, JsonBody, HttpRouter};
use websocket::{Server, Message, Sender, Reciever};
use websocket::message::Type;
use websocket::header::WebSocketProtocol;

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

fn main() {
    let mut server = Nickel::new();

    server.utilize(StaticFilesHandler::new("public"));

    server.get("/", middleware! { |request, response|
        let traffic_data = try_with!(response, {
            request.json_as::<TrafficData>().map_err(|e| (StatusCode::BadRequest, e))
        });
        format!("{}", traffic_data.data);
    });

    server.get("/traffic", middleware! { |_, mut res|
        let mut rng = rand::thread_rng();
        let traffic_data = TrafficData {
            data: rng.gen_range::<i32>(0, 3000)
        };
        traffic_data.to_json()
    });

    for connection in server {
        thread::spawn(move || {
            let request = connection.unwrap().read_request().unwrap();
            let headers = request.headers.clone();

            request.validate().unwrap();

            let mut response = request.accept();

            if let Some(&WebSocketProtocol(ref protocols)) = headers.get() {
                if protocols.contains(&("traffic".to_string())) {
                    response.headers.set(WebSocketProtocol(vec!["traffic".to_string()]));
                }
            }

            let mut client = response.send().unwrap();

            let ip = client.get_mut_sender()
                        .get_mut()
                        .peer_addr()
                        .unwrap();

            println!("Connection from {}", ip);

            let message: Message = Message::text("Hello".to_string());
            client.send_message(&message).unwrap();

            let (mut sender, mut reciever) = client.split();

            for message in reciever.incoming_messages() {
                let message: Message = message.unwrap();

                match message.opcode {
                    Type::Close => {
                        let message = Message::close();
                        sender.send_message(&message).unwrap();
                        println!("Client {} disconnected", ip);
                        return;
                    },
                    Type::Ping => {
                        let message = Message::pong(message.payload);
                        sender.send_message(&message).unwrap();
                    }
                    _ => sender.send_message(&message).unwrap(),
                }
            }
        })
    }

    server.listen("127.0.0.1:6767").unwrap();
}
