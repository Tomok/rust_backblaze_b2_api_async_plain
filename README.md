# backblaze b2 api wrapper


** THIS IS CURRENTLY Work in Progress **



[![Rust](https://github.com/Tomok/rust_backblaze_b2_api_async_plain/actions/workflows/rust.yml/badge.svg)](https://github.com/Tomok/rust_backblaze_b2_api_async_plain/actions/workflows/rust.yml)

This is a very thin wrapper around [backblazes b2 api](https://www.backblaze.com/b2/docs/).
Some highlights:
* individual return types to prevent errors at compile time (for example an upload url is it's own type so that the compiler ensures that you can only pass an upload url recevied via the right methods to calls needing an upload url instead of an api url)
* individual error types so that compile time checks can be used to ensure you have captured all known error cases
* [serde](https://serde.rs) support

### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Serde by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
</sub>
