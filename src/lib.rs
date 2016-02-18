#[macro_use]
extern crate hyper;
extern crate rustc_serialize;

use std::error;
use std::fmt;
use std::io::{self, Read};
use std::error::Error;
use std::result;

use hyper::Client;
use hyper::status::StatusCode;

use rustc_serialize::json;

header! { (XStarfighterAuthorization, "X-Starfighter-Authorization") => [String] }

#[derive(RustcDecodable, RustcEncodable)]
struct Heartbeat {
    ok: bool,
    error: String,
}

#[derive(RustcDecodable, RustcEncodable)]
struct VenueHeartbeat {
    ok: bool,
    // venue is not present on error
    venue: Option<String>,
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct Quote {
    ok: bool,
    symbol: String,
    venue: String,
    bid: Option<usize>,
    ask: Option<usize>,
    bid_size: Option<usize>,
    ask_size: Option<usize>,
    bid_depth: Option<usize>,
    ask_depth: Option<usize>,
    last: usize,
    last_size: Option<usize>,
    last_trade: Option<String>,
    quote_time: Option<String>,
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub enum OrderDirection {
    Buy,
    Sell,
}

// https://starfighter.readme.io/docs/place-new-order#order-types
#[derive(RustcDecodable, RustcEncodable, Debug)]
pub enum OrderType {
    Limit,
    Market,
    FillOrKill,
    ImmediateOrCancel,
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct StockTicker {
    name: String,
    symbol: String,
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct StockList {
    ok: bool,
    symbols: Vec< StockTicker>,
}

#[derive(Debug)]
pub enum StockfighterError {
    ApiDown,
    VenueDown(String), // Also means unknown venue
    ApiError,
    Hyper(hyper::error::Error),
    JsonDecoder(rustc_serialize::json::DecoderError),
    Io(io::Error),
}

impl From<hyper::error::Error> for StockfighterError {
    fn from(err: hyper::error::Error) -> Self {
        StockfighterError::Hyper(err)
    }
}

impl From<rustc_serialize::json::DecoderError> for StockfighterError {
    fn from(err: rustc_serialize::json::DecoderError) -> Self {
        StockfighterError::JsonDecoder(err)
    }
}

impl From<io::Error> for StockfighterError {
    fn from(err: io::Error) -> Self {
        StockfighterError::Io(err)
    }
}

impl fmt::Display for StockfighterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            StockfighterError::ApiDown => write!(f, "API down"),
            StockfighterError::VenueDown(ref venue) => write!(f, "Venue down: {}", venue),
            StockfighterError::ApiError => write!(f, "API error"),
            StockfighterError::Hyper(ref err) => write!(f, "{}", err),
            StockfighterError::JsonDecoder(ref err) => write!(f, "{}", err),
            StockfighterError::Io(ref err) => write!(f, "{}", err),
        }
    }
}

impl Error for StockfighterError {
    fn description(&self) -> &str {
        match *self {
            StockfighterError::ApiDown => "API down",
            StockfighterError::VenueDown(_) => "Venue down",
            StockfighterError::ApiError => "API error",
            StockfighterError::Hyper(ref err) => err.description(),
            StockfighterError::JsonDecoder(ref err) => err.description(),
            StockfighterError::Io(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            StockfighterError::Hyper(ref err) => Some(err as &Error),
            StockfighterError::JsonDecoder(ref err) => Some(err as &Error),
            StockfighterError::Io(ref err) => Some(err as &Error),
            _ => None
        }
    }
}

pub type Result<T> = result::Result<T, StockfighterError>;

pub struct Stockfighter {
    api_key: String,
}

impl Stockfighter {

    pub fn new<S>(api_key: S) -> Stockfighter where S: Into<String> {
        Stockfighter { api_key: api_key.into() }
    }

    ///
    /// Check that the Stockfighter API is up
    ///
    /// # Example
    ///
    /// ```rust
    /// use stockfighter::Stockfighter;
    ///
    /// let sf = Stockfighter::new("fake api key");
    /// assert_eq!(true, sf.heartbeat().is_ok());
    /// ```
    pub fn heartbeat(&self) -> Result<()> {
        let client = Client::new();
        let mut res = try!(client
            .get("https://api.stockfighter.io/ob/api/heartbeat")
            .send());

        if res.status != StatusCode::Ok {
            return Err(StockfighterError::ApiDown);
        }

        let mut body = String::new();
        try!(res.read_to_string(&mut body));

        let hb: Heartbeat = try!(json::decode(&body));

        match hb.ok {
            true => Ok(()),
            false => Err(StockfighterError::ApiDown)
        }
    }

    pub fn venue_heartbeat(&self, venue: &str) -> Result<()> {
        let url = format!("https://api.stockfighter.io/ob/api/venues/{}/heartbeat", venue);
        let client = Client::new();
        let mut res = try!(client
            .get(&url)
            .send());

        if res.status != StatusCode::Ok {
            return Err(StockfighterError::VenueDown(venue.to_owned()));
        }

        let mut body = String::new();
        try!(res.read_to_string(&mut body));

        let hb: VenueHeartbeat = try!(json::decode(&body));

        match hb.ok {
            true => Ok(()),
            false => Err(StockfighterError::VenueDown(venue.to_owned()))
        }
    }

    pub fn quote(&self, venue: &str, stock: &str) -> Result<Quote> {

        let url = format!("https://api.stockfighter.io/ob/api/venues/{}/stocks/{}/quote", venue, stock);

        let client = Client::new();

        let mut res = try!(client
            .get(&url)
            .header(XStarfighterAuthorization(self.api_key.clone())) // TODO fix the use of clone here
            .send());

        if res.status != StatusCode::Ok {
            return Err(StockfighterError::ApiError);
        }

        let mut body = String::new();
        try!(res.read_to_string(&mut body));

        let quote = try!(json::decode::<Quote>(&body));

        match quote.ok {
            true => Ok(quote),
            false => Err(StockfighterError::ApiError)
        }
    }

    pub fn stocks_on_a_venue( &self, venue : &str ) -> Result<StockList> {
        
        let url = format!("https://api.stockfighter.io/ob/api/venues/{}/stocks", venue );

        let client = Client::new();

        let mut res = try!(client
            .get(&url)
            .header(XStarfighterAuthorization(self.api_key.clone())) // TODO fix the use of clone here
            .send());

        if res.status != StatusCode::Ok {
            return Err(StockfighterError::VenueDown(venue.to_owned()));
        }

        let mut body = String::new();
        try!(res.read_to_string(&mut body));

        let stocklist = try!(json::decode::<StockList>(&body));

        match stocklist.ok {
            true => Ok(stocklist),
            false => Err(StockfighterError::ApiError)
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_heartbeat() {
        let sf = Stockfighter::new("");
        assert_eq!(true, sf.heartbeat().is_ok());
    }

    #[test]
    fn test_venue() {
        let sf = Stockfighter::new("");
        assert_eq!(true, sf.venue_heartbeat("TESTEX").is_ok());
        match sf.venue_heartbeat("INVALID") {
            Err(StockfighterError::VenueDown(ref s)) if s == "INVALID" => {},
            _ => panic!()
        }
    }

    #[test]
    fn test_quote() {
        let sf = Stockfighter::new("");
        assert_eq!(true, sf.quote("TESTEX", "FOOBAR").is_ok());
        assert_eq!(true, sf.quote("INVALID", "FOOBAR").is_err());
        assert_eq!(true, sf.quote("TESTEX", "INVALID").is_err());
        assert_eq!(true, sf.quote("INVALID", "INVALID").is_err());
    }

    #[test]
    fn test_stocks_on_a_venue() {
        let sf = Stockfighter::new("");
        assert_eq!(true, sf.stocks_on_a_venue("TESTEX").is_ok());
        println!("{:?}", sf.stocks_on_a_venue("TESTEX") );
        match sf.stocks_on_a_venue("INVALID") {
            Err(StockfighterError::VenueDown(ref s)) if s == "INVALID" => {},
            _ => panic!()
        }
    }

}
