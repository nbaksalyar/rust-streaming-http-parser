#![allow(dead_code)]
extern crate libc;

use std::fmt;
use std::ptr;
use std::str;

type HttpCallback = extern fn(*mut HttpParser) -> libc::c_int;
type HttpDataCallback = extern fn(*mut HttpParser, *const u32, libc::size_t) -> libc::c_int;

#[repr(C)]
#[derive(Clone, Copy)]
enum ParserType {
    HttpRequest,
    HttpResponse,
    HttpBoth
}

#[repr(C)]
struct HttpParser {
    // Private Interface
    _internal_state: libc::uint32_t,
    _nread: libc::uint32_t,
    _content_length: libc::uint64_t,

    // Read-Only
    http_major: libc::c_ushort,
    http_minor: libc::c_ushort,
    _extended_status: libc::uint32_t,

    // Public Interface
    data: *mut libc::c_void
}

impl HttpParser {
    fn new(parser_type: ParserType) -> HttpParser {
        let mut p = HttpParser {
            _internal_state: 0,
            _nread: 0,
            _content_length: 0,
            _extended_status: 0,
            http_major: 0,
            http_minor: 0,
            data: 0 as *mut libc::c_void
        };
        unsafe { http_parser_init(&mut p as *mut _, parser_type); }
        return p;
    }

    fn http_body_is_final(&self) -> libc::c_int {
        unsafe { return http_body_is_final(self); }
    }
}

#[repr(C)]
struct HttpParserSettings {
    on_message_begin: HttpCallback,
    on_url: HttpDataCallback,
    on_status: HttpDataCallback,
    on_header_field: HttpDataCallback,
    on_header_value: HttpDataCallback,
    on_headers_complete: HttpCallback,
    on_body: HttpDataCallback,
    on_message_complete: HttpCallback,
    on_chunk_header: HttpCallback,
    on_chunk_complete: HttpCallback
}

#[inline]
unsafe fn unwrap_parser<'a>(http: *mut HttpParser) -> &'a mut Parser<'a> {
    return &mut *((*http).data as *mut Parser);
}

macro_rules! notify_fn_wrapper {
    ( $callback:ident ) => ({
        extern "C" fn $callback(http: *mut HttpParser) -> libc::c_int {
            unsafe {
                let result = unwrap_parser(http).handler.$callback();
                if result.is_none() {
                    return 0;
                } else {
                    return result.unwrap() as libc::c_int;
                }
            };
        };
        $callback
    });
}

macro_rules! data_fn_wrapper {
    ( $callback:ident ) => ({
        extern "C" fn $callback(http: *mut HttpParser, data: *const u32, size: libc::size_t) -> libc::c_int {
            unsafe {
                let mut dst = Vec::<u8>::with_capacity(size as usize);
                dst.set_len(size as usize);
                ptr::copy(data as *const u8, dst.as_mut_ptr(), size as usize);

                let data = String::from_utf8(dst).unwrap();

                let result = unwrap_parser(http).handler.$callback(&data);
                if result.is_none() {
                    return 0;
                } else {
                    return result.unwrap() as libc::c_int;
                }
            };
        };

        $callback
    });
}

static CALLBACK_WRAPPERS: HttpParserSettings = HttpParserSettings {
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
};

extern "C" {
    fn http_parser_version() -> u32;
    fn http_parser_init(parser: *mut HttpParser, parser_type: ParserType);
    fn http_parser_settings_init(settings: *mut HttpParserSettings);
    fn http_parser_execute(parser: *mut HttpParser, settings: *const HttpParserSettings, data: *const u8, len: libc::size_t) -> libc::size_t;
    fn http_method_str(method_code: u8) -> *const libc::c_char;
    fn http_errno_name(http_errno: u8) -> *const libc::c_char;
    fn http_errno_description(http_errno: u8) -> *const libc::c_char;
    fn http_body_is_final(parser: *const HttpParser) -> libc::c_int;

    // Helper function to predictably use aligned bit-field struct
    fn http_get_struct_flags(parser: *const HttpParser) -> u32;
}

// High level Rust interface

/// Used to define a set of callbacks in your code.
/// They would be called by the parser whenever new data is available.
/// You should bear in mind that the data might get in your callbacks in a partial form.
///
/// Return `Option` as a result of each function call - either
/// `None` for the "OK, go on" status, or `Some(1)` when you want to stop
/// the parser after the function call is ended.
///
/// All callbacks provide a default no-op implementation (i.e. they just return `None`).
///
pub trait ParserHandler {
    /// Called when the URL part of a request becomes available.
    /// E.g. for `GET /forty-two HTTP/1.1` it will be called with `"/forty_two"` argument.
    ///
    /// It's not called in the response mode.
    fn on_url(&self, &String) -> Option<u16> { None }

    /// Called when a response status becomes available.
    ///
    /// It's not called in the request mode.
    fn on_status(&self, &String) -> Option<u16> { None }

    /// Called for each HTTP header key part.
    fn on_header_field(&self, &String) -> Option<u16> { None }

    /// Called for each HTTP header value part.
    fn on_header_value(&self, &String) -> Option<u16> { None }

    /// Called with body text as an argument when the new part becomes available.
    fn on_body(&self, &String) -> Option<u16> { None }

    /// Notified when all available headers have been processed.
    fn on_headers_complete(&self) -> Option<u16> { None }

    /// Notified when the parser receives first bytes to parse.
    fn on_message_begin(&self) -> Option<u16> { None }

    /// Notified when the parser has finished its job.
    fn on_message_complete(&self) -> Option<u16> { None }

    fn on_chunk_header(&self) -> Option<u16> { None }
    fn on_chunk_complete(&self) -> Option<u16> { None }
}

fn http_method_name(method_code: u8) -> String {
    unsafe {
        let method_str = http_method_str(method_code);
        let buf = std::ffi::CStr::from_ptr(method_str);
        return str::from_utf8(buf.to_bytes()).unwrap().to_string();
    }
}

fn _http_errno_name(errno: u8) -> String {
    unsafe {
        let err_str = http_errno_name(errno);
        let buf = std::ffi::CStr::from_ptr(err_str);
        return str::from_utf8(buf.to_bytes()).unwrap().to_string();
    }
}

fn _http_errno_description(errno: u8) -> String {
    unsafe {
        let err_str = http_errno_description(errno);
        let buf = std::ffi::CStr::from_ptr(err_str);
        return str::from_utf8(buf.to_bytes()).unwrap().to_string();
    }
}

/// The main parser interface.
///
/// # Example
/// ```ignore
/// struct MyHandler;
/// impl ParserHandler for MyHandler {
///    fn on_header_field(&self, header: &String) -> Option<u16> {
///        println!("{}: ", header);
///        None
///    }
///    fn on_header_value(&self, value: &String) -> Option<u16> {
///        println!("\t {}", value);
///        None
///    }
/// }
/// 
/// let http_request = "GET / HTTP/1.0\r\n\
///                     Content-Length: 0\r\n\r\n";
///
/// Parser::request(&MyHandler).parse(http_request.as_bytes());
/// ```

pub struct Parser<'a> {
    handler: &'a ParserHandler,
    state: HttpParser,
    flags: u32
}

impl<'a> Parser<'a> {
    /// Creates a new parser instance for an HTTP response.
    ///
    /// Provide it with your `ParserHandler` trait implementation as an argument.
    pub fn response(handler: &'a ParserHandler) -> Parser<'a> {
        Parser {
            handler: handler,
            state: HttpParser::new(ParserType::HttpResponse),
            flags: 0
        }
    }

    /// Creates a new parser instance for an HTTP request.
    ///
    /// Provide it with your `ParserHandler` trait implementation as an argument.
    pub fn request(handler: &'a ParserHandler) -> Parser<'a> {
        Parser {
            handler: handler,
            state: HttpParser::new(ParserType::HttpRequest),
            flags: 0
        }
    }

    /// Creates a new parser instance to handle both HTTP requests and responses.  
    ///
    /// Provide it with your `ParserHandler` trait implementation as an argument.
    pub fn request_and_response(handler: &'a ParserHandler) -> Parser<'a> {
        Parser {
            handler: handler,
            state: HttpParser::new(ParserType::HttpBoth),
            flags: 0
        }
    }

    /// Parses the provided `data` and returns a number of bytes read.
    pub fn parse(&mut self, data: &[u8]) -> usize {
        unsafe {
            self.state.data = self as *mut _ as *mut libc::c_void;

            let size = http_parser_execute(&mut self.state as *mut _,
                                           &CALLBACK_WRAPPERS as *const _,
                                           data.as_ptr(),
                                           data.len() as u64) as usize;

            self.flags = http_get_struct_flags(&self.state as *const _);

            return size;
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

    /// Returns an HTTP method string (`GET`, `POST`, and so on).
    pub fn http_method(&self) -> String {
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
    pub fn error(&self) -> String {
        _http_errno_name(self.http_errnum())
    }

    /// In case of a parsing error returns its description.
    pub fn error_description(&self) -> String {
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
}

impl<'a> fmt::Debug for Parser<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
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
pub fn version() -> String {
    unsafe {
        let v = http_parser_version();

        let major = (v >> 16) & 255;
        let minor = (v >> 8) & 255;
        let patch = v & 255;

        return fmt::format(format_args!("{}.{}.{}", major, minor, patch));
    }
}

#[cfg(test)]
mod tests {
    use super::{version, ParserHandler, Parser};

    #[test]
    fn test_version() {
        assert_eq!("2.5.0", version());
    }

    #[test]
    fn test_request_parser() {
        struct TestRequestParser;
        impl ParserHandler for TestRequestParser {
            fn on_url(&self, url: &String) -> Option<u16> {
                assert_eq!("/say_hello", url);
                None

            }
            fn on_header_field(&self, hdr: &String) -> Option<u16> {
                assert!(hdr == "Host" || hdr == "Content-Length");
                None
            }
            fn on_header_value(&self, val: &String) -> Option<u16> {
                assert!(val == "localhost.localdomain" || val == "11");
                None
            }
            fn on_body(&self, body: &String) -> Option<u16> {
                assert_eq!(body, "Hello world");
                None
            }
        }

        let req = "POST /say_hello HTTP/1.1\r\nContent-Length: 11\r\nHost: localhost.localdomain\r\n\r\nHello world";

        let handler = TestRequestParser;

        let mut parser = Parser::request(&handler);
        let parsed = parser.parse(req.as_bytes());

        assert!(parsed > 0);
        assert!(!parser.has_error());
        assert_eq!((1, 1), parser.http_version());
        assert_eq!("POST", parser.http_method());
    }

    #[test]
    fn test_response_parser() {
        struct TestResponseParser;
        impl ParserHandler for TestResponseParser {
            fn on_status(&self, status: &String) -> Option<u16> {
                assert_eq!("OK", status);
                None
            }
            fn on_header_field(&self, hdr: &String) -> Option<u16> {
                assert_eq!("Host", hdr);
                None
            }
            fn on_header_value(&self, val: &String) -> Option<u16> {
                assert_eq!("localhost.localdomain", val);
                None
            }
        }

        let req = "HTTP/1.1 200 OK\r\nHost: localhost.localdomain\r\n\r\n";

        let handler = TestResponseParser;

        let mut parser = Parser::response(&handler);
        let parsed = parser.parse(req.as_bytes());

        assert!(parsed > 0);
        assert!(!parser.has_error());
        assert_eq!((1, 1), parser.http_version());
        assert_eq!(200, parser.status_code());
    }

    #[test]
    fn test_ws_upgrade() {
        struct DummyHandler;
        impl ParserHandler for DummyHandler {};

        let req = "GET / HTTP/1.1\r\nConnection: Upgrade\r\nUpgrade: websocket\r\n\r\n";

        let handler = DummyHandler;

        let mut parser = Parser::request(&handler);
        parser.parse(req.as_bytes());

        assert_eq!(parser.is_upgrade(), true);
    }

    #[test]
    fn test_error_status() {
        struct DummyHandler;
        impl ParserHandler for DummyHandler {
            fn on_url(&self, _: &String) -> Option<u16> {
                Some(1)
            }
            fn on_header_field(&self, _: &String) -> Option<u16> {
                panic!("This callback shouldn't be executed!");
            }
        }

        let req = "GET / HTTP/1.1\r\nHeader: hello\r\n\r\n";

        Parser::request(&DummyHandler).parse(req.as_bytes());
    }

    #[test]
    fn test_streaming() {
        struct DummyHandler;
        impl ParserHandler for DummyHandler {};

        let req = "GET / HTTP/1.1\r\nHeader: hello\r\n\r\n";

        let handler = DummyHandler;
        let mut parser = Parser::request(&handler);

        parser.parse(&req[0 .. 10].as_bytes());
        assert_eq!(parser.http_version(), (0, 0));
        assert!(!parser.has_error());

        parser.parse(&req[10 ..].as_bytes());
        assert_eq!(parser.http_version(), (1, 1));
    }

    #[test]
    fn test_catch_error() {
        struct DummyHandler;
        impl ParserHandler for DummyHandler {};

        let req = "UNKNOWN_METHOD / HTTP/3.0\r\nAnswer: 42\r\n\r\n";

        let handler = DummyHandler;
        let mut parser = Parser::request(&handler);

        parser.parse(&req.as_bytes());
        assert!(parser.has_error());
        assert_eq!(parser.error(), "HPE_INVALID_METHOD");
    }
}
