extern crate stockfighter;
extern crate env_logger;

use stockfighter::Stockfighter;
use stockfighter::StockfighterError;

#[test]
fn test_heartbeat() {
    let sf = Stockfighter::new("");
    assert!(sf.heartbeat().is_ok());
}

#[test]
fn test_venue() {
    let sf = Stockfighter::new("");
    assert!(sf.venue_heartbeat("TESTEX").is_ok());
    match sf.venue_heartbeat("INVALID") {
        Err(StockfighterError::VenueDown(ref s)) if s == "INVALID" => {},
        _ => panic!()
    }
}

#[test]
fn test_quote() {
    let sf = Stockfighter::new("");
    assert!(sf.quote("TESTEX", "FOOBAR").is_ok());
    assert!(sf.quote("INVALID", "FOOBAR").is_err());
    assert!(sf.quote("TESTEX", "INVALID").is_err());
    assert!(sf.quote("INVALID", "INVALID").is_err());
}

#[test]
fn test_stocks_on_a_venue() {
    let sf = Stockfighter::new("");
    assert!(sf.stocks_on_a_venue("TESTEX").is_ok());
    println!("{:?}", sf.stocks_on_a_venue("TESTEX") );
    match sf.stocks_on_a_venue("INVALID") {
        Err(StockfighterError::VenueDown(ref s)) if s == "INVALID" => {},
        _ => panic!()
    }
}

#[test]
fn test_orderbook_for_stock() {
    let sf = Stockfighter::new("");
    assert!(sf.orderbook_for_stock("TESTEX", "FOOBAR").is_ok());
    match sf.orderbook_for_stock("INVALID", "FOOBAR") {
        Err(StockfighterError::ApiError) => {},
        _ => panic!()
    }
    match sf.orderbook_for_stock("TESTEX", "INVALID") {
        Err(StockfighterError::ApiError) => {},
        _ => panic!()
    }
}

#[test]
fn test_existing_order_status() {
    // TODO Create an is_ok test when we figure out how to test this without an API key.
    // As of now an is_ok test will pass with an existing order and an API key.
    let sf = Stockfighter::new("");
    assert!(sf.existing_order_status(1212, "TESTEX", "INVALID").is_err());
}

#[test]
fn test_status_for_all_orders() {
    // TODO Create an is_ok test when we figure out how to test this without an API key.
    // As of now an is_ok test will pass with an existing order and an API key.
    let sf = Stockfighter::new("");
    assert!(sf.status_for_all_orders("TESTEX", "BA12DFEI12").is_err());
}

#[test]
fn test_status_for_all_orders_on_a_stock() {
    // TODO Create an is_ok test when we figure out how to test this without an API key.
    // As of now an is_ok test will pass with an existing order and an API key.
    let sf = Stockfighter::new("");
    assert!(sf.status_for_all_orders_on_a_stock("TESTEX", "BA12DFEI12", "INVALID").is_err());
}

#[test]
#[ignore] // this test will block forever
fn test_ticker_tape_venue_with() {
    let _ = env_logger::init();

    let sf = Stockfighter::new("");
    let handle = sf.ticker_tape_venue_with("EXB123456", "TESTEX", |quote| println!("{:?}", quote));
    let _ = handle.unwrap().join();
}

#[test]
fn test_ticker_tape_venue_stock_with() {
    let _ = env_logger::init();

    let sf = Stockfighter::new("");
    let handle = sf.ticker_tape_venue_stock_with("EXB123456", "TESTEX", "FOOBAR", |quote| println!("{:?}", quote));
    let _ = handle.unwrap().join();
}
