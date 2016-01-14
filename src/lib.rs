extern crate hyper;
extern crate serde_json;

use std::io::Read;

use hyper::Client;
use hyper::header::Connection;

pub fn test() {
    println!("Hello, Stockfighter!");

    // Create a client.
    let client = Client::new();

    // Creating an outgoing request.
    let mut res = client.get("http://api.stockfighter.io/ob/api/heartbeat")
        // set a header
        .header(Connection::close())
        // let 'er go!
        .send().unwrap();

    // Read the Response.
    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();

    let value: serde_json::Value = serde_json::from_str(&body).unwrap();
    println!("Response: {}", value.as_object().unwrap().get("ok").unwrap().as_boolean().unwrap());
}


