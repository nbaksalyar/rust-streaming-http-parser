#![feature(test)]

extern crate test;
extern crate http_muncher;

use test::Bencher;

use http_muncher::{ParserHandler, Parser};

#[bench]
fn bench_request_parser(b: &mut Bencher) {
    struct TestRequestParser;

    impl ParserHandler for TestRequestParser {
        fn on_url(&mut self, url: &[u8]) -> Option<u16> {
            assert_eq!(b"/say_hello", url);
            None
        }

        fn on_header_field(&mut self, hdr: &[u8]) -> Option<u16> {
            assert!(hdr == b"Host" || hdr == b"Content-Length");
            None
        }

        fn on_header_value(&mut self, val: &[u8]) -> Option<u16> {
            assert!(val == b"localhost.localdomain" || val == b"11");
            None
        }

        fn on_body(&mut self, body: &[u8]) -> Option<u16> {
            assert_eq!(body, b"Hello world");
            None
        }
    }

    let req = b"POST /say_hello HTTP/1.1\r\nContent-Length: 11\r\nHost: localhost.localdomain\r\n\r\nHello world";

    let mut handler = TestRequestParser;

    b.iter(|| {
        let mut parser = Parser::request(&mut handler);
        let parsed = parser.parse(req);

        assert!(parsed > 0);
        assert!(!parser.has_error());
        assert_eq!((1, 1), parser.http_version());
        assert_eq!("POST", parser.http_method());
    });
}
