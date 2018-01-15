extern crate cc;

fn main() {
    cc::Build::new().file("http-parser/http_parser.c").file("src/struct_adapter.c").compile("http_parser");
}
