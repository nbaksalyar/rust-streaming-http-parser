# HttpMuncher: Rust Streaming HTTP parser

![Build Status](https://travis-ci.org/nbaksalyar/rust-streaming-http-parser.svg?branch=master)
[![Build status](https://ci.appveyor.com/api/projects/status/2ihcgjco68t08uge?svg=true)](https://ci.appveyor.com/project/nbaksalyar/rust-streaming-http-parser)
[![](http://meritbadge.herokuapp.com/http-muncher)](https://crates.io/crates/http-muncher)

Rust wrapper for NodeJS [http-parser](https://github.com/nodejs/http-parser) library.

It's intended to be used as an HTTP/1.x protocol handler in Rust-based web servers.

## Motivation

Why not write a brand new HTTP parser from scratch in Rust or just use an existing crate such as **[httparse](https://github.com/seanmonstar/httparse)**?

Here's why:

* NodeJS HTTP parser library is based on a full-featured and robust [nginx](http://nginx.org)'s HTTP parser, and it's safe, fast, and lightweight by design;
* It's compatible with HTTP/1.1, including upgrade connections and chunked responses;
* I haven't found a decent HTTP parser that is capable of streamed parsing, i.e. the one that can eagerly use partial data that comes from a TCP socket;
* Rust's FFI has little to no overhead;
* In most cases, it's silly to reinvent the wheel;
* It was a lot of fun trying to learn Rust along the way. :)

## Usage

Add the library to your `Cargo.toml` dependencies section:

	[dependencies]
	http-muncher = "0.3"

Or, for the edge version:

    [dependencies]
    http-muncher = {git = "https://github.com/nbaksalyar/rust-streaming-http-parser"}

You can find usage example in [examples/parser.rs](examples/parser.rs) (run it by executing `cargo run --example parser`) and in the library tests.

## API documentation

You can find [API docs here](https://docs.rs/http-muncher/).

## Alternative libraries

* [http-parser-rs](https://github.com/magic003/http-parser-rs) - Rust port of NodeJS HTTP parser (without FFI usage).
* [httparse](https://github.com/seanmonstar/httparse) - pure Rust HTTP parser implementation.

## License

The MIT License (MIT)

Copyright (c) 2015 Nikita Baksalyar <<nikita.baksalyar@gmail.com>>
