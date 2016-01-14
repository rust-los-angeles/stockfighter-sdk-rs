extern crate hyper;
extern crate serde_json;

use std::io::Read;

use hyper::Client;
use hyper::header::Connection;

fn get<U: hyper::client::IntoUrl>(url: U) -> serde_json::Value {
    // Create a client.
    let client = Client::new();

    // Creating an outgoing request.
    let mut res = client.get(url)
        // set a header
        .header(Connection::close())
        // let 'er go!
        .send().unwrap();

    // Read the Response.
    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();

    let value: serde_json::Value = serde_json::from_str(&body).unwrap();
    value
}

pub fn test_api() {
    let value = get("http://api.stockfighter.io/ob/api/heartbeat");
    println!("Response: {}", value.as_object().unwrap().get("ok").unwrap().as_boolean().unwrap());
}

pub fn test_venue(venue: &str) {
    let url = format!("https://api.stockfighter.io/ob/api/venues/{}/heartbeat", venue);
    let value = get(&url);
    println!("Response: {}", value.as_object().unwrap().get("ok").unwrap().as_boolean().unwrap());
}
