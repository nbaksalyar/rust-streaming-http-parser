extern crate libc;

mod ffi;

use std::marker::Send;

use ffi::*;


struct ParserContext<'a, H: ParserHandler + 'a> {
    parser: &'a mut Parser,
    handler: &'a mut H
}

#[inline]
unsafe fn unwrap_context<'a, H: ParserHandler>(http: *mut HttpParser) -> &'a mut ParserContext<'a, H> {
    &mut *((*http).data as *mut ParserContext<H>)
}

macro_rules! notify_fn_wrapper {
    ( $callback:ident ) => ({
        extern "C" fn $callback<H: ParserHandler>(http: *mut HttpParser) -> libc::c_int {
            let mut context = unsafe { unwrap_context::<H>(http) };
            if context.handler.$callback(context.parser) { 0 } else { 1 }
        };

        $callback::<H>
    });
}

macro_rules! data_fn_wrapper {
    ( $callback:ident ) => ({
        extern "C" fn $callback<H: ParserHandler>(http: *mut HttpParser, data: *const u32, size: libc::size_t) -> libc::c_int {
            let slice = unsafe { std::slice::from_raw_parts(data as *const u8, size as usize) };
            let mut context = unsafe { unwrap_context::<H>(http) };
            if context.handler.$callback(context.parser, slice) { 0 } else { 1 }
        };

        $callback::<H>
    });
}

impl HttpParserSettings {
    fn new<H: ParserHandler>() -> HttpParserSettings {
        HttpParserSettings {
           on_url: data_fn_wrapper!(on_url),
           on_message_begin: notify_fn_wrapper!(on_message_begin),
           on_status: data_fn_wrapper!(on_status),
           on_header_field: data_fn_wrapper!(on_header_field),
           on_header_value: data_fn_wrapper!(on_header_value),
           on_headers_complete: notify_fn_wrapper!(on_headers_complete),
           on_body: data_fn_wrapper!(on_body),
           on_message_complete: notify_fn_wrapper!(on_message_complete),
           on_chunk_header: notify_fn_wrapper!(on_chunk_header),
           on_chunk_complete: notify_fn_wrapper!(on_chunk_complete)
       }
    }
}

// High level Rust interface

/// Used to define a set of callbacks in your code.
/// They would be called by the parser whenever new data is available.
/// You should bear in mind that the data might get in your callbacks in a partial form.
///
/// Return `bool` as a result of each function call - either
/// `true` for the "OK, go on" status, or `false` when you want to stop
/// the parser after the function call has ended.
///
/// All callbacks provide a default no-op implementation (i.e. they just return `true`).
///
#[allow(unused_variables)]
pub trait ParserHandler: Sized {
    /// Called when the URL part of a request becomes available.
    /// E.g. for `GET /forty-two HTTP/1.1` it will be called with `"/forty_two"` argument.
    ///
    /// It's not called in the response mode.
    fn on_url(&mut self, &mut Parser, &[u8]) -> bool { true }

    /// Called when a response status becomes available.
    ///
    /// It's not called in the request mode.
    fn on_status(&mut self, &mut Parser, &[u8]) -> bool { true }

    /// Called for each HTTP header key part.
    fn on_header_field(&mut self, &mut Parser, &[u8]) -> bool { true }

    /// Called for each HTTP header value part.
    fn on_header_value(&mut self, &mut Parser, &[u8]) -> bool { true }

    /// Called with body text as an argument when the new part becomes available.
    fn on_body(&mut self, &mut Parser, &[u8]) -> bool { true }

    /// Notified when all available headers have been processed.
    fn on_headers_complete(&mut self, &mut Parser) -> bool { true }

    /// Notified when the parser receives first bytes to parse.
    fn on_message_begin(&mut self, &mut Parser) -> bool { true }

    /// Notified when the parser has finished its job.
    fn on_message_complete(&mut self, &mut Parser) -> bool { true }

    fn on_chunk_header(&mut self, &mut Parser) -> bool { true }
    fn on_chunk_complete(&mut self, &mut Parser) -> bool { true }
}

fn http_method_name(method_code: u8) -> &'static str {
    unsafe {
        let method_str = http_method_str(method_code);
        let buf = std::ffi::CStr::from_ptr(method_str);
        return std::str::from_utf8(buf.to_bytes()).unwrap();
    }
}

fn _http_errno_name(errno: u8) -> &'static str {
    unsafe {
        let err_str = http_errno_name(errno);
        let buf = std::ffi::CStr::from_ptr(err_str);
        return std::str::from_utf8(buf.to_bytes()).unwrap();
    }
}

fn _http_errno_description(errno: u8) -> &'static str {
    unsafe {
        let err_str = http_errno_description(errno);
        let buf = std::ffi::CStr::from_ptr(err_str);
        return std::str::from_utf8(buf.to_bytes()).unwrap();
    }
}

/// The main parser interface.
///
/// # Example
/// ```ignore
/// struct MyHandler;
/// impl ParserHandler for MyHandler {
///    fn on_header_field(&self, header: &[u8]) -> bool {
///        println!("{}: ", header);
///        true
///    }
///    fn on_header_value(&self, value: &[u8]) -> bool {
///        println!("\t {}", value);
///        true
///    }
/// }
///
/// let http_request = b"GET / HTTP/1.0\r\n\
///                      Content-Length: 0\r\n\r\n";
///
/// Parser::request(&MyHandler).parse(http_request);
/// ```

#[allow(dead_code)]
pub struct Parser {
    state: HttpParser,
    parser_type: ParserType,
    flags: u32
}

unsafe impl Send for Parser { }

impl Parser {
    /// Creates a new parser instance for an HTTP response.
    ///
    /// Provide it with your `ParserHandler` trait implementation as an argument.
    pub fn response() -> Parser {
        Parser {
            parser_type: ParserType::HttpResponse,
            state: HttpParser::new(ParserType::HttpResponse),
            flags: 0
        }
    }

    /// Creates a new parser instance for an HTTP request.
    ///
    /// Provide it with your `ParserHandler` trait implementation as an argument.
    pub fn request() -> Parser {
        Parser {
            parser_type: ParserType::HttpRequest,
            state: HttpParser::new(ParserType::HttpRequest),
            flags: 0
        }
    }

    /// Creates a new parser instance to handle both HTTP requests and responses.
    ///
    /// Provide it with your `ParserHandler` trait implementation as an argument.
    pub fn request_and_response() -> Parser {
        Parser {
            parser_type: ParserType::HttpBoth,
            state: HttpParser::new(ParserType::HttpBoth),
            flags: 0
        }
    }

    /// Parses the provided `data` and returns a number of bytes read.
    pub fn parse<'a, H: ParserHandler>(&mut self, handler: &mut H, data: &[u8]) -> usize {
        unsafe {
            let mut context = ParserContext { parser: self, handler: handler };

            context.parser.state.data = &mut context as *mut _ as *mut libc::c_void;

            let size = http_parser_execute(&mut context.parser.state as *mut _,
                                           &HttpParserSettings::new::<H>() as *const _,
                                           data.as_ptr(),
                                           data.len() as libc::size_t) as usize;

            context.parser.flags = http_get_struct_flags(&context.parser.state as *const _);

            size
        }
    }

    /// Returns an HTTP request or response version.
    pub fn http_version(&self) -> (u16, u16) {
        (self.state.http_major, self.state.http_minor)
    }

    /// Returns an HTTP response status code (think *404*).
    pub fn status_code(&self) -> u16 {
        return (self.flags & 0xFFFF) as u16
    }

    /// Returns an HTTP method static string (`GET`, `POST`, and so on).
    pub fn http_method(&self) -> &'static str {
        let method_code = ((self.flags >> 16) & 0xFF) as u8;
        return http_method_name(method_code);
    }

    fn http_errnum(&self) -> u8 {
        return ((self.flags >> 24) & 0x7F) as u8
    }

    /// Checks if the last `parse` call was finished successfully.
    /// Returns `true` if it wasn't.
    pub fn has_error(&self) -> bool {
        self.http_errnum() != 0x00
    }

    /// In case of a parsing error returns its mnemonic name.
    pub fn error(&self) -> &'static str {
        _http_errno_name(self.http_errnum())
    }

    /// In case of a parsing error returns its description.
    pub fn error_description(&self) -> &'static str {
        _http_errno_description(self.http_errnum())
    }

    /// Checks if an upgrade protocol (e.g. WebSocket) was requested.
    pub fn is_upgrade(&self) -> bool {
        return ((self.flags >> 31) & 0x01) == 1;
    }

    /// Checks if it was the final body chunk.
    pub fn is_final_chunk(&self) -> bool {
        return self.state.http_body_is_final() == 1;
    }

    /// If `should_keep_alive()` in the `on_headers_complete` or `on_message_complete` callback
    /// returns 0, then this should be the last message on the connection.
    /// If you are the server, respond with the "Connection: close" header.
    /// If you are the client, close the connection.
    pub fn should_keep_alive(&self) -> bool {
        self.state.http_should_keep_alive() == 1
    }
}

impl std::fmt::Debug for Parser {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let (version_major, version_minor) = self.http_version();
        return write!(fmt, "status_code: {}\n\
                            method: {}\n\
                            error: {}, {}\n\
                            upgrade: {}\n\
                            http_version: {}.{}",
                      self.status_code(),
                      self.http_method(),
                      self.error(), self.error_description(),
                      self.is_upgrade(),
                      version_major, version_minor);
    }
}

/// Returns a version of the underlying `http-parser` library.
pub fn version() -> (u32, u32, u32) {
    let version = unsafe {
        http_parser_version()
    };

    let major = (version >> 16) & 255;
    let minor = (version >> 8) & 255;
    let patch = version & 255;

    (major, minor, patch)
}

#[cfg(test)]
mod tests {
    use super::{version, ParserHandler, Parser};

    #[test]
    fn test_version() {
        assert_eq!((2, 7, 1), version());
    }

    #[test]
    fn test_request_parser() {
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

        let mut parser = Parser::request();
        let parsed = parser.parse(&mut handler, req);

        assert!(parsed > 0);
        assert!(!parser.has_error());
        assert_eq!((1, 1), parser.http_version());
        assert_eq!("POST", parser.http_method());
    }

    #[test]
    fn test_response_parser() {
        struct TestResponseParser;

        impl ParserHandler for TestResponseParser {
            fn on_status(&mut self, _: &mut Parser, status: &[u8]) -> bool {
                assert_eq!(b"OK", status);
                true
            }

            fn on_header_field(&mut self, _: &mut Parser, hdr: &[u8]) -> bool {
                assert_eq!(b"Host", hdr);
                true
            }

            fn on_header_value(&mut self, _: &mut Parser, val: &[u8]) -> bool {
                assert_eq!(b"localhost.localdomain", val);
                true
            }
        }

        let req = b"HTTP/1.1 200 OK\r\nHost: localhost.localdomain\r\n\r\n";

        let mut handler = TestResponseParser;

        let mut parser = Parser::response();
        let parsed = parser.parse(&mut handler, req);

        assert!(parsed > 0);
        assert!(!parser.has_error());
        assert_eq!((1, 1), parser.http_version());
        assert_eq!(200, parser.status_code());
    }

    #[test]
    fn test_ws_upgrade() {
        struct DummyHandler;

        impl ParserHandler for DummyHandler {};

        let req = b"GET / HTTP/1.1\r\nConnection: Upgrade\r\nUpgrade: websocket\r\n\r\n";

        let mut handler = DummyHandler;

        let mut parser = Parser::request();
        parser.parse(&mut handler, req);

        assert_eq!(parser.is_upgrade(), true);
    }

    #[test]
    fn test_error_status() {
        struct DummyHandler {
            url_parsed: bool,
        }

        impl ParserHandler for DummyHandler {
            fn on_url(&mut self, _: &mut Parser, _: &[u8]) -> bool {
                self.url_parsed = true;
                false
            }

            fn on_header_field(&mut self, _: &mut Parser, _: &[u8]) -> bool {
                panic!("This callback shouldn't be executed!");
            }
        }

        let req = b"GET / HTTP/1.1\r\nHeader: hello\r\n\r\n";

        let mut handler = DummyHandler { url_parsed: false };

        let mut parser = Parser::request();
        parser.parse(&mut handler, req);

        assert!(handler.url_parsed);
    }

    #[test]
    fn test_streaming() {
        struct DummyHandler;

        impl ParserHandler for DummyHandler {};

        let req = b"GET / HTTP/1.1\r\nHeader: hello\r\n\r\n";

        let mut handler = DummyHandler;
        let mut parser = Parser::request();

        parser.parse(&mut handler, &req[0..10]);

        assert_eq!(parser.http_version(), (0, 0));
        assert!(!parser.has_error());

        parser.parse(&mut handler, &req[10..]);

        assert_eq!(parser.http_version(), (1, 1));
    }

    #[test]
    fn test_catch_error() {
        struct DummyHandler;

        impl ParserHandler for DummyHandler {};

        let req = b"UNKNOWN_METHOD / HTTP/3.0\r\nAnswer: 42\r\n\r\n";

        let mut handler = DummyHandler;
        let mut parser = Parser::request();

        parser.parse(&mut handler, req);

        assert!(parser.has_error());
        assert_eq!(parser.error(), "HPE_INVALID_METHOD");
    }
}
