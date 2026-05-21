#include "parser.h"
#include <stdlib.h>

static TSParser *parser = NULL;

TSTree *parse_c_file(const char *source_code, size_t length) {
    if (!parser) {
        parser = ts_parser_new();
        ts_parser_set_language(parser, tree_sitter_c());
    }

    return ts_parser_parse_string(parser, NULL, source_code, length);
}

void cleanup_parser(void) {
    if (parser) {
        ts_parser_delete(parser);
        parser = NULL;
    }
}
