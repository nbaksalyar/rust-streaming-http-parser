# HttpMuncher: Rust Streaming HTTP parser

![Build Status](https://travis-ci.org/nbaksalyar/rust-streaming-http-parser.svg?branch=master)
[![](http://meritbadge.herokuapp.com/http-muncher)](https://crates.io/crates/http-muncher)

Rust wrapper for Joyent's [http-parser](https://github.com/joyent/http-parser) library.

It's intended to be used as an HTTP/1.x protocol handler in Rust-based web servers.

## Motivation

Why not write a brand new HTTP parser from scratch in Rust or just use an existing crate such as **[httparse](https://github.com/seanmonstar/httparse)**?

Here's why:

* Joyent's library is based on a full-featured and robust [nginx](http://nginx.org)'s HTTP parser, and it's safe, fast, and lightweight by design;
* It's compatible with HTTP/1.1, including upgrade connections and chunked responses;
* I haven't found a decent HTTP parser that is capable of streamed parsing, i.e. the one that can eagerly use partial data that comes from a TCP socket;
* Rust's FFI has little to no overhead;
* In most cases, it's silly to reinvent the wheel;
* It was a lot of fun trying to learn Rust along the way. :)

## Usage

Add the library to your `Cargo.toml` dependencies section:

	[dependencies]
	http-muncher = "0.2.*"

Or, for the edge version:

	[dependency.http-muncher]
	git = "https://github.com/nbaksalyar/rust-streaming-http-parser"

And then you can use it this way:

```Rust
extern crate http_muncher;

// Include the 2 main public interfaces of the crate
use http_muncher::{Parser, ParserHandler};

// Now let's define a new listener for parser events:
struct MyHandler;
impl ParserHandler for MyHandler {

    // Now we can define our callbacks here.
    //
    // Let's try to handle headers: the following callback function will be
    // called when parser founds a header in the HTTP stream.

    fn on_header_field(&self, header: &[u8]) -> Option<u16> {
        // Print the received header key
        println!("{}: ", header);

        // We have nothing to say to parser, and we'd like
        // it to continue its work - so let's return "None".
        None
    }

    // And let's print the header values in a similar vein:
    fn on_header_value(&self, value: &[u8]) -> Option<u16> {
        println!("\t {}", value);
        None
    }
}

fn main() {
    // Now we can create a parser instance with our callbacks handler:
    let callbacks_handler = MyHandler;
    let mut parser = Parser::request(&callbacks_handler);

    // Let's define a mock HTTP request:
    let http_request = "GET / HTTP/1.0\r\n\
                        Content-Type: text/plain\r\n\
                        Content-Length: 0\r\n\
                        Hello: World\r\n\r\n";

    // And now we're ready to go!
    parser.parse(http_request.as_bytes());

    // Now that callbacks have been called, we can introspect
    // the parsing results - for instance, print the HTTP version:
    let (http_major, http_minor) = parser.http_version();
    println!("{}.{}", http_major, http_minor);
}

// Now execute "cargo run", and as a result you should see this output:

// Content-Type: 
//	 text/plain
// Content-Length: 
//	 0
// Hello: 
// 	 World

// ... and the rest isf almost the same - have fun experimenting!
```

Some more basic usage examples can be found in the library tests as well.

## API documentation

You can find [API docs here](http://nbaksalyar.github.io/rust-streaming-http-parser/).

## License

The MIT License (MIT)

Copyright (c) 2015 Nikita Baksalyar <<nikita.baksalyar@gmail.com>>
