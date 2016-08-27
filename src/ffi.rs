//! This module provides an interface to the NodeJS http-parser library.

use libc;
use std::mem;

pub type HttpCallback = extern fn(*mut HttpParser) -> libc::c_int;
pub type HttpDataCallback = extern fn(*mut HttpParser, *const u32, libc::size_t) -> libc::c_int;

#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub enum ParserType {
    HttpRequest,
    HttpResponse,
    HttpBoth
}

#[repr(C)]
pub struct HttpParser {
    // Private Interface
    _internal_state: libc::uint32_t,
    _nread: libc::uint32_t,
    _content_length: libc::uint64_t,

    // Read-Only
    pub http_major: libc::c_ushort,
    pub http_minor: libc::c_ushort,
    pub _extended_status: libc::uint32_t,

    // Public Interface
    pub data: *mut libc::c_void
}

unsafe impl Send for HttpParser { }

impl HttpParser {
    pub fn new(parser_type: ParserType) -> HttpParser {
        let mut p: HttpParser = unsafe { mem::uninitialized() };
        unsafe { http_parser_init(&mut p as *mut _, parser_type); }
        return p;
    }

    pub fn http_body_is_final(&self) -> libc::c_int {
        unsafe { return http_body_is_final(self); }
    }

    pub fn http_should_keep_alive(&self) -> libc::c_int {
        unsafe { http_should_keep_alive(self) }
    }
}

#[repr(C)]
pub struct HttpParserSettings {
    pub on_message_begin: HttpCallback,
    pub on_url: HttpDataCallback,
    pub on_status: HttpDataCallback,
    pub on_header_field: HttpDataCallback,
    pub on_header_value: HttpDataCallback,
    pub on_headers_complete: HttpCallback,
    pub on_body: HttpDataCallback,
    pub on_message_complete: HttpCallback,
    pub on_chunk_header: HttpCallback,
    pub on_chunk_complete: HttpCallback
}

#[allow(dead_code)]
extern "C" {
    pub fn http_parser_version() -> u32;
    pub fn http_parser_init(parser: *mut HttpParser, parser_type: ParserType);
    pub fn http_parser_settings_init(settings: *mut HttpParserSettings);
    pub fn http_parser_execute(parser: *mut HttpParser, settings: *const HttpParserSettings, data: *const u8, len: libc::size_t) -> libc::size_t;
    pub fn http_method_str(method_code: u8) -> *const libc::c_char;
    pub fn http_errno_name(http_errno: u8) -> *const libc::c_char;
    pub fn http_errno_description(http_errno: u8) -> *const libc::c_char;
    pub fn http_body_is_final(parser: *const HttpParser) -> libc::c_int;

    // Helper function to predictably use aligned bit-field struct
    pub fn http_get_struct_flags(parser: *const HttpParser) -> u32;

    pub fn http_should_keep_alive(parser: *const HttpParser) -> libc::c_int;
}
