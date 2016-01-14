extern crate stockfighter;

use stockfighter::{test_api,test_venue};

fn main() {
    test_api();
    test_venue("TESTEX");
    test_venue("NOTANEX");
}

