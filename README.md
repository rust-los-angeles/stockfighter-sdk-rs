# stockfighter-sdk-rs

Stockfighter.io SDK written in Rust

## Stockfighter API Documentation

 * [Stockfighter Developer Hub](https://starfighter.readme.io/)
 * [Stockfighter v1.0 API Documentation](https://starfighter.readme.io/v1.0/docs)

## Local Application Development

This SDK is a library and not suitable to solve the Stockfighter.io levels by itself. You can add this library to Cargo following the instructions on the [crates.io webpage](https://crates.io/crates/stockfighter-sdk-rs/), but it is not complete. If you are developing on this library and want to use it in your app, then you can use cargo to soure your local copy:

Create a `.cargo/config` file in some ancestor of your project’s directory (common places to put it is in the root of your code directory or in your home directory). Inside that file, put this:

```
paths = ["/path/to/project/stockfighter-sdk-rs"]
```

This array should be filled with directories that contain a Cargo.toml. In this instance, we’re just adding `stockfighter-sdk-rs`, so it will be the only one that’s overridden. This path must be an absolute path.

More information about local configuration can be found in the [configuration documentation](http://doc.crates.io/config.html).

## Mac SSL trouble:
```
$ brew install openssl
$ brew link --force openssl

$ export OPENSSL_INCLUDE_DIR=/usr/local/opt/openssl/include
$ export DEP_OPENSSL_INCLUDE=/usr/local/opt/openssl/include

$ cargo build
$ cargo run
```

## License

Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
