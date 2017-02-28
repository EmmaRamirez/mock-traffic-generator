#[macro_use] extern crate nickel;
extern crate rustc_serialize;
extern crate rand;

use rustc_serialize::json::{Json, ToJson};
use rand::Rng;
use std::collections::BTreeMap;
use nickel::status::StatusCode;
use nickel::{Nickel, JsonBody, HttpRouter, MediaType};

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

    server.post("/", middleware! { |request, response|
        let traffic_data = try_with!(response, {
            request.json_as::<TrafficData>().map_err(|e| (StatusCode::BadRequest, e))
        });
        format!("{}", traffic_data.data)
    });

    server.get("/traffic", middleware! { |_, mut res|
        let mut rng = rand::thread_rng();
        let traffic_data = TrafficData {
            data: rng.gen::<i32>()
        };
        res.set(MediaType::Json);
        r#"{ "data": "traffic_data" }"#
    });

    server.listen("127.0.0.1:6767").unwrap();
}
