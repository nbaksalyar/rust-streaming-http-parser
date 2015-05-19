#![allow(dead_code)]
extern crate libc;

use std::fmt;
use std::ptr;
use std::str;

type HttpCallback = extern fn(*mut HttpParser) -> *mut libc::c_int;
type HttpDataCallback = extern fn(*mut HttpParser, *const u32, libc::size_t) -> *mut libc::c_int;

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
    fn new() -> HttpParser {
        HttpParser {
            _internal_state: 0,
            _nread: 0,
            _content_length: 0,
            _extended_status: 0,
            http_major: 0,
            http_minor: 0,
            data: 0 as *mut libc::c_void
        }
    }

    fn status_code(&self) -> u16 {
        unsafe {
            let flags = http_get_struct_flags(self as *const _);
            return (flags & 0xFFFF) as u16;
        }
    }

    fn method_code(&self) -> u8 {
        unsafe {
            let flags = http_get_struct_flags(self as *const _);
            return ((flags >> 16) & 0xFF) as u8;
        }
    }

    fn http_errno(&self) -> u8 {
        unsafe {
            let flags = http_get_struct_flags(self as *const _);
            return ((flags >> 24) & 0x7F) as u8;
        }
    }

    fn upgrade(&self) -> bool {
        unsafe {
            let flags = http_get_struct_flags(self as *const _);
            return ((flags >> 31) & 0x01) == 1;
        }
    }

    fn http_method_str(&self) -> &str {
        unsafe {
            let method_str = http_method_str(self.method_code());
            let buf = std::ffi::CStr::from_ptr(method_str);
            return str::from_utf8(buf.to_bytes()).unwrap();
        }
    }

    fn http_errno_name(&self) -> &str {
        unsafe {
            let method_str = http_errno_name(self.http_errno());
            let buf = std::ffi::CStr::from_ptr(method_str);
            return str::from_utf8(buf.to_bytes()).unwrap();
        }
    }

    fn http_errno_description(&self) -> &str {
        unsafe {
            let method_str = http_errno_description(self.http_errno());
            let buf = std::ffi::CStr::from_ptr(method_str);
            return str::from_utf8(buf.to_bytes()).unwrap();
        }
    }
}

impl fmt::Debug for HttpParser {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        return write!(fmt, "status_code: {}\n\
                            method: 0x{:X}, {}\n\
                            errno: 0x{:X}, {}, {}\n\
                            upgrade: {}\n\
                            http_version: {}.{}",
                      self.status_code(), self.method_code(), self.http_method_str(), self.http_errno(), self.http_errno_name(), self.http_errno_description(), self.upgrade(), self.http_major, self.http_minor);
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
        extern "C" fn $callback(http: *mut HttpParser) -> *mut libc::c_int {
            unsafe {
                unwrap_parser(http).handler.$callback();
            };
            return 0 as *mut libc::c_int;
        };
        $callback
    });
}

macro_rules! data_fn_wrapper {
    ( $callback:ident ) => ({
        extern "C" fn $callback(http: *mut HttpParser, data: *const u32, size: libc::size_t) -> *mut libc::c_int {
            unsafe {
                let mut dst = Vec::<u8>::with_capacity(size as usize);
                dst.set_len(size as usize);
                ptr::copy(data as *const u8, dst.as_mut_ptr(), size as usize);

                let data = String::from_utf8(dst).unwrap();

                unwrap_parser(http).handler.$callback(&data);
            };
            return 0 as *mut libc::c_int;
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

    // Helper function to predictably use aligned bit-field struct
    fn http_get_struct_flags(parser: *const HttpParser) -> u32;
}

// High level Rust interface
pub trait ParserHandler {
    fn on_url(&self, &String) { }
    fn on_status(&self, &String) { }
    fn on_header_field(&self, &String) { }
    fn on_header_value(&self, &String) { }
    fn on_body(&self, &String) { }
    fn on_message_begin(&self) { }
    fn on_headers_complete(&self) { }
    fn on_message_complete(&self) { }
    fn on_chunk_header(&self) { }
    fn on_chunk_complete(&self) { }
}

pub struct Parser<'a> {
    handler: &'a ParserHandler,
    state: HttpParser,
    parser_type: ParserType
}

impl<'a> Parser<'a> {
    pub fn response(handler: &'a ParserHandler) -> Parser<'a> {
        Parser {
            handler: handler,
            state: HttpParser::new(),
            parser_type: ParserType::HttpResponse
        }
    }

    pub fn request(handler: &'a ParserHandler) -> Parser<'a> {
        Parser {
            handler: handler,
            state: HttpParser::new(),
            parser_type: ParserType::HttpRequest
        }
    }

    pub fn request_and_response(handler: &'a ParserHandler) -> Parser<'a> {
        Parser {
            handler: handler,
            state: HttpParser::new(),
            parser_type: ParserType::HttpBoth
        }
    }

    pub fn http_version(&self) -> (u16, u16) { return (self.state.http_major, self.state.http_minor) }
    pub fn http_status_code(&self) -> u16 { return self.state.status_code(); }
    pub fn is_upgrade(&self) -> bool { return self.state.upgrade() }
    pub fn http_method(&self) -> &str { return self.state.http_method_str(); }
}

pub fn parse(parser: &mut Parser, data: &[u8]) -> usize {
    unsafe {
        http_parser_init(&mut parser.state as *mut _, parser.parser_type);

        parser.state.data = &mut (*parser) as *mut _ as *mut libc::c_void;

        return http_parser_execute(&mut parser.state as *mut _,
                                         &CALLBACK_WRAPPERS as *const _,
                                         data.as_ptr(),
                                         data.len() as u64) as usize;
    }
}

pub fn version() -> String {
    unsafe {
        let v = http_parser_version();

        let major = (v >> 16) & 255;
        let minor = (v >> 8) & 255;
        let patch = v & 255;

        return fmt::format(format_args!("{}.{}.{}", major, minor, patch));
    }
}

#[test]
fn test_version() {
    assert_eq!("2.5.0", version());
}

#[test]
fn test_request_parser() {
    struct TestRequestParser;
    impl ParserHandler for TestRequestParser {
        fn on_url(&self, url: &String) { assert_eq!("/say_hello", url); }
        fn on_header_field(&self, hdr: &String) { assert!(hdr == "Host" || hdr == "Content-Length"); }
        fn on_header_value(&self, val: &String) { assert!(val == "localhost.localdomain" || val == "11"); }
        fn on_body(&self, body: &String) { assert_eq!(body, "Hello world"); }
    }

    let req = "POST /say_hello HTTP/1.1\r\nContent-Length: 11\r\nHost: localhost.localdomain\r\n\r\nHello world";

    let handler = TestRequestParser;
    let mut parser = Parser::request(&handler);
    let parsed = parse(&mut parser, req.as_bytes());

    assert!(parsed > 0);
    assert_eq!((1, 1), parser.http_version());
    assert_eq!("POST", parser.http_method());
}

#[test]
fn test_response_parser() {
    struct TestResponseParser;
    impl ParserHandler for TestResponseParser {
        fn on_status(&self, status: &String) { assert_eq!("OK", status); }
        fn on_header_field(&self, hdr: &String) { assert_eq!("Host", hdr); }
        fn on_header_value(&self, val: &String) { assert_eq!("localhost.localdomain", val); }
    }

    let req = "HTTP/1.1 200 OK\r\nHost: localhost.localdomain\r\n\r\n";

    let handler = TestResponseParser;
    let mut parser = Parser::response(&handler);
    let parsed = parse(&mut parser, req.as_bytes());

    assert!(parsed > 0);
    assert_eq!((1, 1), parser.http_version());
    assert_eq!(200, parser.http_status_code());
}

#[test]
fn test_ws_upgrade() {
    struct DummyHandler;
    impl ParserHandler for DummyHandler {};

    let req = "GET / HTTP/1.1\r\nConnection: Upgrade\r\nUpgrade: websocket\r\n\r\n";

    let handler = DummyHandler;
    let mut parser = Parser::request(&handler);
    parse(&mut parser, req.as_bytes());

    assert_eq!(parser.is_upgrade(), true);
}
