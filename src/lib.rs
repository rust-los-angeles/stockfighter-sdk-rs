#[macro_use]
extern crate hyper;
extern crate rustc_serialize;

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

#[derive(RustcDecodable, RustcEncodable, Debug)]
#[allow(non_snake_case)]
pub struct Quote {
    ok: bool,
    symbol: String,
    venue: String,
    bid: Option<usize>,
    ask: Option<usize>,
    bidSize: Option<usize>,
    askSize: Option<usize>,
    bidDepth: Option<usize>,
    askDepth: Option<usize>,
    last: usize,
    lastSize: Option<usize>,
    lastTrade: Option<String>,
    quoteTime: Option<String>,
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
#[allow(non_snake_case)] //when I make it order_type it doesn't work
pub struct Order {
    account: String,
    venue: String,
    stock: String,
    price: usize,
    qty: usize,
    direction: OrderDirection,
    orderType: String
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct Fill {
    price: Option<usize>,
    qty: Option<usize>,
    ts: Option<String>
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
#[allow(non_camel_case_types)]
pub enum OrderDirection {
    buy,
    sell,
}

// https://starfighter.readme.io/docs/place-new-order#order-types
// Note that in the Order and OrderStatus structs the orderType is
// represented as a string. This is because some of the valid API
// values are invalid symbols in rust (e.g. "fill-or-kill") and
// rustc_serialize autoserialization doesn't support field renaming.
pub enum OrderType {
    Limit,
    Market,
    FillOrKill,
    ImmediateOrCancel,
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
#[allow(non_snake_case)]
pub struct OrderStatus {
    ok: bool,
    symbol: Option<String>,
    venue: Option<String>,
    direction: Option<OrderDirection>,
    originalQty: Option<usize>,
    qty: Option<usize>,
    price: Option<usize>,
    orderType: Option<String>,
    id: Option<usize>,
    account: Option<String>,
    ts: Option<String>,
    fills: Option<Vec<Fill>>,
    totalFilled: Option<usize>,
    open: Option<bool>
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

#[derive(RustcDecodable, RustcEncodable, Debug)]
#[allow(non_snake_case)]
pub struct BidAsk {
    price: usize,
    qty: usize,
    isBuy: bool
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct OrderbookList {
    ok: bool,
    venue: String,
    symbol: String,
    bids: Option<Vec< BidAsk >>,
    asks: Option<Vec< BidAsk >>,
    ts: String
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct StockOrdersStatuses {
    ok: bool,
    venue: String,
    orders: Vec< Order >
}

#[derive(Debug)]
pub enum StockfighterError {
    ApiDown,
    VenueDown(String), // Also means unknown venue
    ApiError,
    Hyper(hyper::error::Error),
    JsonDecoder(rustc_serialize::json::DecoderError),
    JsonEncoder(rustc_serialize::json::EncoderError),
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

impl From<rustc_serialize::json::EncoderError> for StockfighterError {
    fn from(err: rustc_serialize::json::EncoderError) -> Self {
        StockfighterError::JsonEncoder(err)
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
            StockfighterError::JsonEncoder(ref err) => write!(f, "{}", err),
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
            StockfighterError::JsonEncoder(ref err) => err.description(),
            StockfighterError::Io(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            StockfighterError::Hyper(ref err) => Some(err as &Error),
            StockfighterError::JsonDecoder(ref err) => Some(err as &Error),
            StockfighterError::JsonEncoder(ref err) => Some(err as &Error),
            StockfighterError::Io(ref err) => Some(err as &Error),
            _ => None
        }
    }
}

pub type Result<T> = result::Result<T, StockfighterError>;

pub struct Stockfighter {
    api_key: String,
    client: Client,
}

impl Stockfighter {

    pub fn new<S>(api_key: S) -> Stockfighter where S: Into<String> {
        Stockfighter { api_key: api_key.into(), client: Client::new() }
    }

    /// Check that the Stockfighter API is up
    ///
    /// # Example
    ///
    /// ```rust
    /// use stockfighter::Stockfighter;
    ///
    /// let sf = Stockfighter::new("fake api key");
    /// assert!(sf.heartbeat().is_ok());
    /// ```
    pub fn heartbeat(&self) -> Result<()> {
        let mut res = try!(self.client
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

    /// Check that a specific venue is up
    ///
    /// # Example
    ///
    /// ```rust
    /// use stockfighter::Stockfighter;
    ///
    /// let sf = Stockfighter::new("fake api key");
    /// assert!(sf.venue_heartbeat("TESTEX").is_ok());
    /// ```
    pub fn venue_heartbeat(&self, venue: &str) -> Result<()> {
        let url = format!("https://api.stockfighter.io/ob/api/venues/{}/heartbeat", venue);
        let mut res = try!(self.client
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

    /// Get a quick look at the most recent trade information for a stock.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stockfighter::Stockfighter;
    ///
    /// let sf = Stockfighter::new("fake api key");
    /// assert!(sf.quote("TESTEX", "FOOBAR").is_ok());
    /// ```
    pub fn quote(&self, venue: &str, stock: &str) -> Result<Quote> {

        let url = format!("https://api.stockfighter.io/ob/api/venues/{}/stocks/{}/quote", venue, stock);

        let mut res = try!(self.client
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

    /// List the stocks available for trading on a venue
    ///
    /// # Example
    ///
    /// ```rust
    /// use stockfighter::Stockfighter;
    ///
    /// let sf = Stockfighter::new("fake api key");
    /// assert!(sf.stocks_on_a_venue("TESTEX").is_ok());
    /// ```
    pub fn stocks_on_a_venue(&self, venue: &str) -> Result<StockList> {

        let url = format!("https://api.stockfighter.io/ob/api/venues/{}/stocks", venue );

        let mut res = try!(self.client
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

    /// Get the orderbook for a particular stock
    ///
    /// # Example
    ///
    /// ```rust
    /// use stockfighter::Stockfighter;
    ///
    /// let sf = Stockfighter::new("fake api key");
    /// assert!(sf.orderbook_for_stock("TESTEX", "FOOBAR").is_ok());
    /// ```
    pub fn orderbook_for_stock(&self, venue: &str, symbol: &str) -> Result<OrderbookList> {
        let url = format!("https://api.stockfighter.io/ob/api/venues/{}/stocks/{}", venue, symbol);

        let mut res = try!(self.client
            .get(&url)
            .header(XStarfighterAuthorization(self.api_key.clone())) // TODO fix the use of clone here
            .send());

        println!("{:?}", res);
        if res.status != StatusCode::Ok {
            return Err(StockfighterError::ApiError);
        }

        let mut body = String::new();
        try!(res.read_to_string(&mut body));

        let orderbook = try!(json::decode::<OrderbookList>(&body));

        match orderbook.ok {
            true => Ok(orderbook),
            false => Err(StockfighterError::ApiError)
        }
    }

    /// Post a new order
    ///
    /// # Example
    /// # Note that this tests for failure, due to the fake api
    /// key. With a real key, this example should pass.
    /// ```rust
    /// use stockfighter::{Stockfighter, OrderDirection, OrderType};
    ///
    /// let sf = Stockfighter::new("fake api key");
    /// assert!(sf.new_order("EXB123456", "TESTEX", "FOOBAR", 10000, 42,
    ///                                 OrderDirection::buy, OrderType::Limit).is_err());
    /// ```
    pub fn new_order(&self, account: &str, venue: &str, stock: &str, price: usize, qty: usize,
                     direction: OrderDirection, order_type: OrderType) -> Result<OrderStatus> {
        let url = format!("https://api.stockfighter.io/ob/api/venues/{}/stocks/{}/orders", venue, stock);

        let ot = match order_type {
            OrderType::Limit => "limit",
            OrderType::Market => "market",
            OrderType::FillOrKill => "fill-or-kill",
            OrderType::ImmediateOrCancel => "immediate-or-cancel"
        }.to_string();
        let order = Order {account: account.to_string(), venue: venue.to_string(), stock: stock.to_string(),
                           price: price, qty: qty, direction: direction, orderType: ot};
        let order_encoded = try!(json::encode(&order)).to_string();
        let mut res = try!(
            self.client
                .post(&url)
                .header(XStarfighterAuthorization(self.api_key.clone()))
                .body(&order_encoded)
                .send()
        );
        if res.status != StatusCode::Ok {
            return Err(StockfighterError::ApiError);
        }
        let mut body = String::new();
        try!(res.read_to_string(&mut body));
        let order_status = try!(json::decode::<OrderStatus>(&body));
        match order_status.ok {
            true => Ok(order_status),
            false => Err(StockfighterError::ApiError)
        }
    }

    pub fn existing_order_status(&self, id: usize, venue: &str, stock: &str) -> Result<OrderStatus> {
        let url = format!("https://api.stockfighter.io/ob/api/venues/{}/stocks/{}/orders/{}", venue, stock, id);

        let mut res = try!(
            self.client
                .get(&url)
                .header(XStarfighterAuthorization(self.api_key.clone())) // TODO fix clone here.
                .send()
        );

        if res.status != StatusCode::Ok {
            return Err(StockfighterError::ApiError);
        }

        let mut body = String::new();
        try!(res.read_to_string(&mut body));

        let order_status = try!(json::decode::<OrderStatus>(&body));

        match order_status.ok {
            true => Ok(order_status),
            false => Err(StockfighterError::ApiError)
        }
    }

    /// [Get the Status For All Orders](https://starfighter.readme.io/docs/status-for-all-orders)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use stockfighter::Stockfighter;
    ///
    /// let sf = Stockfighter::new("fake api key");
    /// assert!(sf.status_for_all_orders("TESTEX", "EXB123456").is_ok());
    /// ```
    pub fn status_for_all_orders(&self, venue: &str, account: &str) -> Result<StockOrdersStatuses> {

        let url = format!("https://api.stockfighter.io/ob/api/venues/{}/accounts/{}/orders", venue, account);

        let mut res = try!(self.client
                           .get(&url)
                           .header(XStarfighterAuthorization(self.api_key.clone())) // TODO fix the use of clone here
                           .send());

        if res.status != StatusCode::Ok {
            return Err(StockfighterError::VenueDown(venue.to_owned()));
        }

        let mut body = String::new();
        try!(res.read_to_string(&mut body));

        let stock_statuses = try!(json::decode::<StockOrdersStatuses>(&body));

        match stock_statuses.ok {
            true => Ok(stock_statuses),
            false => Err(StockfighterError::ApiError)
        }
    }

    /// [Get the Status For All Orders In A Stock](https://starfighter.readme.io/docs/status-for-all-orders-in-a-stock)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use stockfighter::Stockfighter;
    ///
    /// let sf = Stockfighter::new("fake api key");
    /// assert!(sf.status_for_all_orders_on_a_stock("TESTEX", "EXB123456", "FOOBAR").is_ok());
    /// ```
    pub fn status_for_all_orders_on_a_stock(&self, venue: &str, account: &str, stock: &str) -> Result<StockOrdersStatuses> {

        let url = format!("https://api.stockfighter.io/ob/api/venues/{}/accounts/{}/stocks/{}/orders", venue, account, stock );

        let mut res = try!(self.client
                           .get(&url)
                           .header(XStarfighterAuthorization(self.api_key.clone())) // TODO fix the use of clone here
                           .send());

        if res.status != StatusCode::Ok {
            return Err(StockfighterError::VenueDown(venue.to_owned()));
        }

        let mut body = String::new();
        try!(res.read_to_string(&mut body));

        let stock_statuses = try!(json::decode::<StockOrdersStatuses>(&body));

        match stock_statuses.ok {
            true => Ok(stock_statuses),
            false => Err(StockfighterError::ApiError)
        }
    }

    /// [Cancel An Order](https://starfighter.readme.io/docs/cancel-an-order)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use stockfighter::Stockfighter;
    ///
    /// let sf = Stockfighter::new("fake api key");
    /// assert!(sf.cancel_an_order("TESTEX", "FOOBAR", 1).is_ok());
    /// ```
    pub fn cancel_an_order(&self, venue: &str, stock: &str, order: usize) -> Result<OrderStatus> {
        let url = format!("https://api.stockfighter.io/ob/api/venues/{}/stocks/{}/orders/{}", venue, stock, order );

        let mut res = try!(self.client
        .delete(&url)
        .header(XStarfighterAuthorization(self.api_key.clone())) // TODO fix the use of clone here
        .send());

        if res.status != StatusCode::Ok {
            return Err(StockfighterError::ApiError);
        }

        let mut body = String::new();
        try!(res.read_to_string(&mut body));

        let order_status = try!(json::decode::<OrderStatus>(&body));

        match order_status.ok {
            true => Ok(order_status),
            false => Err(StockfighterError::ApiError)
        }
    }
}
