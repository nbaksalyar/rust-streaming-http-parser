#![allow(unused_variables)]

extern crate http_muncher;
use std::str;

// Include the 2 main public interfaces of the crate
use http_muncher::{Parser, ParserHandler};

// Now let's define a new listener for parser events:
struct MyHandler;

impl ParserHandler for MyHandler {
    // Now we can define our callbacks here.
    //
    // Let's try to handle headers: the following callback function will be
    // called when parser founds a header chunk in the HTTP stream.
    fn on_header_field(&mut self, parser: &mut Parser, header: &[u8]) -> bool {
        // Print the received header key part
        println!("{}: ", str::from_utf8(header).unwrap());

        // We have nothing to say to parser, and we'd like
        // it to continue its work - so let's return "true".
        true
    }

    // And let's print the header value chunks in a similar vein:
    fn on_header_value(&mut self, parser: &mut Parser, value: &[u8]) -> bool {
        println!("\t{}", str::from_utf8(value).unwrap());
        true
    }
}

fn main() {
    // Now we can create a parser instance with our callbacks handler:
    let mut callbacks_handler = MyHandler {};
    let mut parser = Parser::request();

    // Let's define a mock HTTP request:
    let http_request = "GET / HTTP/1.1\r\n\
                        Content-Type: text/plain\r\n\
                        Content-Length: 2\r\n\
                        Hello: World\r\n\r\n\
                        Hi";

    // And now we're ready to go!
    parser.parse(&mut callbacks_handler, http_request.as_bytes());

    // Now that callbacks have been called, we can introspect
    // the parsing results. For instance, print the HTTP version:
    let (http_major, http_minor) = parser.http_version();
    println!("\nHTTP v.{}.{}", http_major, http_minor);
}

// Now execute "cargo run", and as a result you should see this output:

// Content-Type:
//	 text/plain
// Content-Length:
//	 0
// Hello:
// 	 World
//
// HTTP v1.1

// ... and the rest is almost the same - have fun experimenting!
