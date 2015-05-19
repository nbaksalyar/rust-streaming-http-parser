
#include "../http-parser/http_parser.h"

/*
Realigns a bit field struct in a predictable way.
*/
uint32_t http_get_struct_flags(const http_parser *state) {
  return state->status_code |
    (state->method << 16) |
    (state->http_errno << 24) |
    (state->upgrade << 31);
}
