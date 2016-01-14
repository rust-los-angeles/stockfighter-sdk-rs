#[macro_use]
extern crate hyper;
extern crate serde_json;

use std::io::Read;

use hyper::Client;
use hyper::header::Connection;

header! { (XStarfighterAuthorization, "X-Starfighter-Authorization") => [String] }

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
    let value = get("https://api.stockfighter.io/ob/api/heartbeat");
    println!("Response: {}", value.as_object().unwrap().get("ok").unwrap().as_boolean().unwrap());
}

pub fn venue(venue: &str) -> bool {
    let url = format!("https://api.stockfighter.io/ob/api/venues/{}/heartbeat", venue);
    let value = get(&url);
    value.as_object().unwrap().get("ok").unwrap().as_boolean().unwrap()
}

pub fn quote(venue: &str, stock: &str) -> bool {

    let url = format!("https://api.stockfighter.io/ob/api/venues/{}/stocks/{}/quote", venue, stock);

    let client = Client::new();

    let mut res = client
        .get(&url)
        .header(XStarfighterAuthorization("b6eb6d0a2b606c02c8b027fca35383fb2dc741d3".to_owned()))
        .send()
        .unwrap();

    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();

    let value: serde_json::Value = serde_json::from_str(&body).unwrap();
    value.as_object().unwrap().get("ok").unwrap().as_boolean().unwrap()
}

#[cfg(test)]
mod test {
    use super::*;


    #[test]
    fn test_venue() {
        assert_eq!(true, venue("TESTEX"));
        assert_eq!(false, venue("INVALID"));
    }

    #[test]
    fn test_quote() {
        assert_eq!(true, quote("TESTEX", "FOOBAR"));
    }
}
