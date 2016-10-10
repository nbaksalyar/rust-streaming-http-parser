#![feature(test)]

extern crate test;
extern crate http_muncher;

use test::Bencher;

use http_muncher::{ParserHandler, Parser};

#[bench]
fn bench_request_parser(b: &mut Bencher) {
    struct TestRequestParser;

    impl ParserHandler for TestRequestParser {
        fn on_url(&mut self, _: &mut Parser, url: &[u8]) -> bool {
            assert_eq!(b"/say_hello", url);
            true
        }

        fn on_header_field(&mut self, _: &mut Parser, hdr: &[u8]) -> bool {
            assert!(hdr == b"Host" || hdr == b"Content-Length");
            true
        }

        fn on_header_value(&mut self, _: &mut Parser, val: &[u8]) -> bool {
            assert!(val == b"localhost.localdomain" || val == b"11");
            true
        }

        fn on_body(&mut self, _: &mut Parser, body: &[u8]) -> bool {
            assert_eq!(body, b"Hello world");
            true
        }
    }

    let req = b"POST /say_hello HTTP/1.1\r\nContent-Length: 11\r\nHost: localhost.localdomain\r\n\r\nHello world";

    let mut handler = TestRequestParser;

    b.iter(move || {
        let mut parser = Parser::request();
        let parsed = parser.parse(&mut handler, req);

        assert!(parsed > 0);
        assert!(!parser.has_error());
        assert_eq!((1, 1), parser.http_version());
        assert_eq!("POST", parser.http_method());
    });
}
