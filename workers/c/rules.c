#include "rules.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <ctype.h>

static void add_issue(IssueList *list, const char *rule, const char *severity, 
                     const char *message, int line, int column) {
    if (list->count >= list->capacity) {
        list->capacity = list->capacity == 0 ? 16 : list->capacity * 2;
        list->items = realloc(list->items, list->capacity * sizeof(Issue));
    }

    Issue *issue = &list->items[list->count++];
    strncpy(issue->rule, rule, sizeof(issue->rule) - 1);
    issue->rule[sizeof(issue->rule) - 1] = '\0';
    strncpy(issue->severity, severity, sizeof(issue->severity) - 1);
    issue->severity[sizeof(issue->severity) - 1] = '\0';
    strncpy(issue->message, message, sizeof(issue->message) - 1);
    issue->message[sizeof(issue->message) - 1] = '\0';
    issue->line = line;
    issue->column = column;
}

// Rule 1: Line length exceeds 80 characters
static void check_line_length(const char *source_code, IssueList *issues) {
    int line = 1;
    int line_length = 0;

    for (const char *p = source_code; *p; p++) {
        if (*p == '\n') {
            if (line_length > 80) {
                char msg[256];
                snprintf(msg, sizeof(msg), "Line exceeds 80 characters (%d characters)", line_length);
                add_issue(issues, "line-length", "warning", msg, line, 1);
            }
            line++;
            line_length = 0;
        } else {
            line_length++;
        }
    }

    // Check last line if no trailing newline
    if (line_length > 80) {
        char msg[256];
        snprintf(msg, sizeof(msg), "Line exceeds 80 characters (%d characters)", line_length);
        add_issue(issues, "line-length", "warning", msg, line, 1);
    }
}

// Rule 2: Space before tab in indentation
static void check_space_before_tab(const char *source_code, IssueList *issues) {
    int line = 1;
    int column = 1;
    bool at_line_start = true;
    bool seen_space = false;

    for (const char *p = source_code; *p; p++) {
        if (*p == '\n') {
            line++;
            column = 1;
            at_line_start = true;
            seen_space = false;
        } else if (at_line_start) {
            if (*p == ' ') {
                seen_space = true;
                column++;
            } else if (*p == '\t') {
                if (seen_space) {
                    add_issue(issues, "space-before-tab", "error", 
                             "Space before tab in indentation", line, column);
                }
                column++;
            } else {
                at_line_start = false;
            }
        } else {
            column++;
        }
    }
}

// Rule 3: Trailing whitespace
static void check_trailing_whitespace(const char *source_code, IssueList *issues) {
    int line = 1;
    const char *line_start = source_code;

    for (const char *p = source_code; *p; p++) {
        if (*p == '\n') {
            // Check backwards for trailing whitespace
            const char *end = p - 1;
            while (end >= line_start && (*end == ' ' || *end == '\t')) {
                end--;
            }
            if (end < p - 1) {
                add_issue(issues, "trailing-whitespace", "warning", 
                         "Trailing whitespace at end of line", line, 1);
            }
            line++;
            line_start = p + 1;
        }
    }
}

// Rule 4: Missing space after keywords (if, for, while, switch)
static void check_keyword_spacing(const char *source_code, IssueList *issues) {
    const char *keywords[] = {"if", "for", "while", "switch"};
    const size_t num_keywords = sizeof(keywords) / sizeof(keywords[0]);
    
    int line = 1;
    int column = 1;
    const char *line_start = source_code;

    for (const char *p = source_code; *p; p++) {
        if (*p == '\n') {
            line++;
            column = 1;
            line_start = p + 1;
            continue;
        }

        // Check if we're at the start of a keyword
        for (size_t i = 0; i < num_keywords; i++) {
            size_t kw_len = strlen(keywords[i]);
            
            // Check if keyword matches and is a whole word
            if (strncmp(p, keywords[i], kw_len) == 0) {
                // Check it's not part of a larger identifier
                bool is_word_start = (p == source_code || !isalnum(*(p-1)) && *(p-1) != '_');
                bool is_word_end = !isalnum(*(p + kw_len)) && *(p + kw_len) != '_';
                
                if (is_word_start && is_word_end) {
                    char next_char = *(p + kw_len);
                    // Should have space or newline after keyword, not '('
                    if (next_char == '(') {
                        char msg[256];
                        snprintf(msg, sizeof(msg), "Missing space after '%s' keyword", keywords[i]);
                        add_issue(issues, "keyword-spacing", "warning", msg, line, column);
                    }
                }
            }
        }
        
        column++;
    }
}

// Rule 5: Braces on same line (K&R style for kernel)
static void check_brace_style(const char *source_code, IssueList *issues) {
    int line = 1;
    int column = 1;
    const char *line_start = source_code;
    
    // Track if we just saw a control statement
    const char *keywords[] = {"if", "for", "while", "switch"};
    const size_t num_keywords = sizeof(keywords) / sizeof(keywords[0]);

    for (const char *p = source_code; *p; p++) {
        if (*p == '\n') {
            line++;
            column = 1;
            line_start = p + 1;
            continue;
        }

        // Look for opening brace at start of line (after whitespace)
        if (*p == '{') {
            // Check if this line only has whitespace before the brace
            const char *check = line_start;
            bool only_whitespace = true;
            while (check < p) {
                if (*check != ' ' && *check != '\t') {
                    only_whitespace = false;
                    break;
                }
                check++;
            }

            if (only_whitespace && line > 1) {
                // Check previous line for control keywords
                const char *prev_line = line_start - 2; // Skip the \n
                while (prev_line > source_code && *(prev_line - 1) != '\n') {
                    prev_line--;
                }
                
                // Look for keywords in previous line
                for (size_t i = 0; i < num_keywords; i++) {
                    if (strstr(prev_line, keywords[i]) != NULL) {
                        add_issue(issues, "brace-style", "warning", 
                                 "Opening brace should be on same line as statement", 
                                 line, column);
                        break;
                    }
                }
            }
        }
        
        column++;
    }
}

IssueList check_all_rules(const char *source_code) {
    IssueList issues = {NULL, 0, 0};

    // Run all checks
    check_line_length(source_code, &issues);
    check_space_before_tab(source_code, &issues);
    check_trailing_whitespace(source_code, &issues);
    check_keyword_spacing(source_code, &issues);
    check_brace_style(source_code, &issues);

    return issues;
}

void free_issue_list(IssueList *list) {
    if (list->items) {
        free(list->items);
        list->items = NULL;
    }
    list->count = 0;
    list->capacity = 0;
}
