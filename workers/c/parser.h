#ifndef PARSER_H
#define PARSER_H

#include <tree_sitter/api.h>

// Initialize tree-sitter parser for C
TSTree *parse_c_file(const char *source_code, size_t length);

// Cleanup parser resources
void cleanup_parser(void);

// Get the C language
TSLanguage *tree_sitter_c(void);

#endif // PARSER_H
