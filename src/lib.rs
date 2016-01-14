
extern crate hyper;

use std::env;
use std::io;
use std::io::Read;

use hyper::Client;
use hyper::header::Connection;

pub fn test() {
    println!("Hello, Stockfighter!");

    // Create a client.
    let mut client = Client::new();

    // Creating an outgoing request.
    let mut res = client.get("http://api.stockfighter.io/ob/api/heartbeat")
        // set a header
        .header(Connection::close())
        // let 'er go!
        .send().unwrap();

    // Read the Response.
    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();

    println!("Response: {}", body);    
}


