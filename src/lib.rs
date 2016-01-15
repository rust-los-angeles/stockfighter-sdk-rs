#[macro_use]
extern crate hyper;
extern crate rustc_serialize;

use std::io::Read;

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

#[derive(Debug, PartialEq, Eq)]
pub enum StockfighterError {
    ApiDown,
    VenueDown(String), // Also means unknown venue
    ApiError
}

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
    pub fn heartbeat(&self) -> Result<(), StockfighterError> {
        let client = Client::new();
        let mut res = client
            .get("https://api.stockfighter.io/ob/api/heartbeat")
            .send()
            .unwrap();

        if res.status != StatusCode::Ok {
            return Err(StockfighterError::ApiDown);
        }

        let mut body = String::new();
        res.read_to_string(&mut body).unwrap();

        let hb: Heartbeat = json::decode(&body).unwrap();

        match hb.ok {
            true => Ok(()),
            false => Err(StockfighterError::ApiDown)
        }
    }

    pub fn venue_heartbeat(&self, venue: &str) -> Result<(), StockfighterError> {
        let url = format!("https://api.stockfighter.io/ob/api/venues/{}/heartbeat", venue);
        let client = Client::new();
        let mut res = client
            .get(&url)
            .send()
            .unwrap();

        if res.status != StatusCode::Ok {
            return Err(StockfighterError::VenueDown(venue.to_owned()));
        }

        let mut body = String::new();
        res.read_to_string(&mut body).unwrap();

        let hb: VenueHeartbeat = json::decode(&body).unwrap();

        match hb.ok {
            true => Ok(()),
            false => Err(StockfighterError::VenueDown(venue.to_owned()))
        }
    }

    pub fn quote(&self, venue: &str, stock: &str) -> Result<Quote, StockfighterError> {

        let url = format!("https://api.stockfighter.io/ob/api/venues/{}/stocks/{}/quote", venue, stock);

        let client = Client::new();

        let mut res = client
            .get(&url)
            .header(XStarfighterAuthorization(self.api_key.clone())) // TODO fix the use of clone here
            .send()
            .unwrap();

        if res.status != StatusCode::Ok {
            return Err(StockfighterError::ApiError);
        }

        let mut body = String::new();
        res.read_to_string(&mut body).unwrap();

        let quote: Quote = json::decode(&body).unwrap();

        match quote.ok {
            true => Ok(quote),
            false => Err(StockfighterError::ApiError)
        }
    }
    pub fn stocks_on_a_venue( &self, venue : &str ) -> Result<StockList, StockfighterError> {
        
        let url = format!("https://api.stockfighter.io/ob/api/venues/{}/stocks", venue );

        let client = Client::new();

        let mut res = client
            .get(&url)
            .header(XStarfighterAuthorization(self.api_key.clone())) // TODO fix the use of clone here
            .send()
            .unwrap();

        if res.status != StatusCode::Ok {
            return Err(StockfighterError::VenueDown(venue.to_owned()));
        }

        let mut body = String::new();
        res.read_to_string(&mut body).unwrap();

        let stocklist: StockList = json::decode(&body).unwrap();

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
        assert_eq!(StockfighterError::VenueDown("INVALID".to_owned()), sf.venue_heartbeat("INVALID").unwrap_err());
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
        assert_eq!(StockfighterError::VenueDown("INVALID".to_owned()), sf.stocks_on_a_venue("INVALID").unwrap_err());
    }

}
