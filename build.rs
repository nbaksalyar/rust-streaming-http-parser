extern crate gcc;

fn main() {
    gcc::compile_library("libhttp_parser.a", &["http-parser/http_parser.c"]);
}
