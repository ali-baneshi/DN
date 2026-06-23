#include "rules.h"
<<<<<<< HEAD
#include <ctype.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    char *text;
    int line;
} LineView;

typedef struct {
    char name[64];
    bool locked;
    bool is_spin;
} LockState;

typedef struct {
    char ptr[64];
    int line;
    bool checked_after_deref;
} DerefState;

static void add_issue(IssueList *list, const char *rule, const char *severity,
                      const char *message, int line, int column,
                      const char *category) {
    if (list->count >= list->capacity) {
        size_t next = list->capacity == 0 ? 16 : list->capacity * 2;
        Issue *items = realloc(list->items, next * sizeof(Issue));
        if (!items) {
            return;
        }
        list->items = items;
        list->capacity = next;
    }

    Issue *issue = &list->items[list->count++];
    memset(issue, 0, sizeof(*issue));
    strncpy(issue->rule, rule, sizeof(issue->rule) - 1);
    strncpy(issue->severity, severity, sizeof(issue->severity) - 1);
    strncpy(issue->message, message, sizeof(issue->message) - 1);
    strncpy(issue->category, category, sizeof(issue->category) - 1);
=======
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
>>>>>>> feature/persistent-workers
    issue->line = line;
    issue->column = column;
}

<<<<<<< HEAD
static bool contains_word(const char *line, const char *word) {
    const char *hit = strstr(line, word);
    if (!hit) {
        return false;
    }
    size_t len = strlen(word);
    char before = hit == line ? ' ' : hit[-1];
    char after = hit[len];
    return !isalnum((unsigned char)before) && before != '_' &&
           !isalnum((unsigned char)after) && after != '_';
}

static int column_of(const char *line, const char *needle) {
    const char *hit = strstr(line, needle);
    return hit ? (int)(hit - line) + 1 : 1;
}

static int split_lines(const char *source, LineView *lines, int max_lines) {
    int count = 0;
    const char *start = source;
    int line_no = 1;
    while (*start && count < max_lines) {
        const char *end = strchr(start, '\n');
        size_t len = end ? (size_t)(end - start) : strlen(start);
        char *copy = malloc(len + 1);
        if (!copy) {
            break;
        }
        memcpy(copy, start, len);
        copy[len] = '\0';
        lines[count].text = copy;
        lines[count].line = line_no++;
        count++;
        if (!end) {
            break;
        }
        start = end + 1;
    }
    return count;
}

static void free_lines(LineView *lines, int count) {
    for (int i = 0; i < count; i++) {
        free(lines[i].text);
    }
}

static bool is_cleanup_line(const char *line) {
    const char *patterns[] = {
        "kfree(",     "free(",        "mutex_unlock(", "spin_unlock(",
        "up(",        "put_",         "release(",      "cleanup",
        "unlock(",    "destroy(",     "close(",        "rcu_read_unlock(",
    };
    for (size_t i = 0; i < sizeof(patterns) / sizeof(patterns[0]); i++) {
        if (strstr(line, patterns[i])) {
            return true;
        }
    }
    return false;
}

static void check_style_rules(LineView *lines, int count, IssueList *issues) {
    const char *keywords[] = {"if", "for", "while", "switch"};

    for (int i = 0; i < count; i++) {
        const char *line = lines[i].text;
        size_t len = strlen(line);

        if (len > 80 && line[0] != '#' && !strstr(line, "/*") && !strstr(line, "//")) {
            char msg[128];
            snprintf(msg, sizeof(msg), "Line exceeds 80 characters (%zu characters)", len);
            add_issue(issues, "line-length", "warning", msg, lines[i].line, 1, "style");
        }

        for (size_t col = 1; line[col]; col++) {
            if (line[col] == '\t' && line[col - 1] == ' ') {
                add_issue(issues, "space-before-tab", "error",
                          "Space before tab in indentation", lines[i].line,
                          (int)col + 1, "style");
                break;
            }
        }

        while (len > 0 && (line[len - 1] == ' ' || line[len - 1] == '\t')) {
            add_issue(issues, "trailing-whitespace", "warning",
                      "Trailing whitespace at end of line", lines[i].line, (int)len,
                      "style");
            break;
        }

        for (size_t k = 0; k < sizeof(keywords) / sizeof(keywords[0]); k++) {
            const char *hit = strstr(line, keywords[k]);
            if (hit && (hit == line || (!isalnum((unsigned char)hit[-1]) && hit[-1] != '_')) &&
                hit[strlen(keywords[k])] == '(') {
                add_issue(issues, "keyword-spacing", "warning",
                          "Missing space after keyword", lines[i].line,
                          (int)(hit - line) + 1, "style");
            }
        }

        if (strchr(line, '{') &&
            (contains_word(line, "if") || contains_word(line, "for") ||
             contains_word(line, "while") || contains_word(line, "switch") ||
             contains_word(line, "do"))) {
            add_issue(issues, "brace-style", "warning",
                      "Opening brace should follow kernel style", lines[i].line,
                      column_of(line, "{"), "style");
        }
    }
}

static void parse_call_argument(const char *line, const char *fn, char *out, size_t out_len) {
    const char *hit = strstr(line, fn);
    if (!hit) {
        out[0] = '\0';
        return;
    }
    const char *start = strchr(hit, '(');
    if (!start) {
        out[0] = '\0';
        return;
    }
    start++;
    while (*start == ' ' || *start == '&' || *start == '*') {
        start++;
    }
    size_t idx = 0;
    while (start[idx] &&
           (isalnum((unsigned char)start[idx]) || start[idx] == '_' || start[idx] == '-')) {
        if (idx + 1 < out_len) {
            out[idx] = start[idx];
        }
        idx++;
    }
    if (idx >= out_len) {
        idx = out_len - 1;
    }
    out[idx] = '\0';
}

static int find_lock(LockState *locks, int lock_count, const char *name) {
    for (int i = 0; i < lock_count; i++) {
        if (strcmp(locks[i].name, name) == 0) {
            return i;
        }
    }
    return -1;
}

static void check_kernel_rules(LineView *lines, int count, IssueList *issues) {
    LockState locks[64];
    DerefState derefs[128];
    int lock_count = 0;
    int deref_count = 0;
    int rcu_depth = 0;
    bool saw_init = false;
    bool saw_exit = false;
    bool saw_non_init_call_in_init = false;
    bool saw_init_call_in_non_init = false;

    for (int i = 0; i < count; i++) {
        const char *line = lines[i].text;

        if (strstr(line, "__init")) {
            saw_init = true;
        }
        if (strstr(line, "__exit")) {
            saw_exit = true;
        }
        if (strstr(line, "__init") && strstr(line, "_exit(")) {
            saw_non_init_call_in_init = true;
        }
        if (strstr(line, "__exit") && strstr(line, "_init(")) {
            saw_init_call_in_non_init = true;
        }

        if (strstr(line, "goto out;")) {
            bool cleanup = false;
            for (int j = i + 1; j < count && j < i + 8; j++) {
                if (strstr(lines[j].text, "out:")) {
                    for (int k = j + 1; k < count && k < j + 6; k++) {
                        if (is_cleanup_line(lines[k].text)) {
                            cleanup = true;
                            break;
                        }
                        if (contains_word(lines[k].text, "return")) {
                            break;
                        }
                    }
                    break;
                }
            }
            if (!cleanup) {
                add_issue(issues, "goto-out-without-cleanup", "medium",
                          "goto out reaches a label without visible cleanup",
                          lines[i].line, column_of(line, "goto"), "reliability");
            }
        }

        if (strstr(line, "spin_lock(") || strstr(line, "spin_lock_irqsave(") ||
            strstr(line, "spin_lock_bh(") || strstr(line, "mutex_lock(")) {
            char name[64];
            bool is_spin = strstr(line, "spin_lock") != NULL;
            parse_call_argument(line, is_spin ? "spin_lock" : "mutex_lock", name, sizeof(name));
            if (name[0] != '\0') {
                int idx = find_lock(locks, lock_count, name);
                if (idx < 0 && lock_count < (int)(sizeof(locks) / sizeof(locks[0]))) {
                    idx = lock_count++;
                    memset(&locks[idx], 0, sizeof(locks[idx]));
                    strncpy(locks[idx].name, name, sizeof(locks[idx].name) - 1);
                }
                if (idx >= 0) {
                    locks[idx].locked = true;
                    locks[idx].is_spin = is_spin;
                }
            }
        }

        if (strstr(line, "spin_unlock(") || strstr(line, "spin_unlock_irqrestore(") ||
            strstr(line, "spin_unlock_bh(") || strstr(line, "mutex_unlock(")) {
            char name[64];
            bool is_spin = strstr(line, "spin_unlock") != NULL;
            parse_call_argument(line, is_spin ? "spin_unlock" : "mutex_unlock", name, sizeof(name));
            int idx = find_lock(locks, lock_count, name);
            if (idx >= 0) {
                locks[idx].locked = false;
            }
        }

        if (contains_word(line, "return")) {
            for (int l = 0; l < lock_count; l++) {
                if (locks[l].locked) {
                    add_issue(issues, "return-without-unlock", "high",
                              "Return occurs while a lock is still held",
                              lines[i].line, column_of(line, "return"), "concurrency");
                    break;
                }
            }
        }

        if (strstr(line, "msleep(") || strstr(line, "schedule(") ||
            strstr(line, "might_sleep(")) {
            for (int l = 0; l < lock_count; l++) {
                if (locks[l].locked && locks[l].is_spin) {
                    add_issue(issues, "sleeping-in-atomic", "high",
                              "Sleeping or blocking call appears in atomic context",
                              lines[i].line, 1, "concurrency");
                    break;
                }
            }
        }

        const char *arrow = strstr(line, "->");
        if (arrow) {
            const char *start = arrow;
            while (start > line &&
                   (isalnum((unsigned char)start[-1]) || start[-1] == '_' || start[-1] == ')')) {
                start--;
            }
            if (start < arrow && deref_count < (int)(sizeof(derefs) / sizeof(derefs[0]))) {
                size_t len = (size_t)(arrow - start);
                if (len >= sizeof(derefs[deref_count].ptr)) {
                    len = sizeof(derefs[deref_count].ptr) - 1;
                }
                memcpy(derefs[deref_count].ptr, start, len);
                derefs[deref_count].ptr[len] = '\0';
                derefs[deref_count].line = lines[i].line;
                derefs[deref_count].checked_after_deref = false;
                deref_count++;
            }
        }

        for (int d = 0; d < deref_count; d++) {
            if ((strstr(line, "if (") || strstr(line, "if(")) &&
                (strstr(line, derefs[d].ptr) && (strstr(line, "NULL") || strstr(line, "!")))) {
                if (derefs[d].line < lines[i].line && !derefs[d].checked_after_deref) {
                    add_issue(issues, "null-deref-before-check", "high",
                              "Pointer is dereferenced before it is checked for NULL",
                              derefs[d].line, 1, "correctness");
                    derefs[d].checked_after_deref = true;
                }
            }
        }

        if (strstr(line, "rcu_read_lock(")) {
            rcu_depth++;
        }
        if (rcu_depth > 0 && strstr(line, "->") && !strstr(line, "rcu_dereference(") &&
            !strstr(line, "ptr = rcu_dereference(") &&
            !strstr(line, "list_for_each_entry_rcu") && !strstr(line, "hlist_for_each_entry_rcu")) {
            add_issue(issues, "RCU-missing-annotation", "medium",
                      "RCU-protected pointer dereference should use rcu_dereference",
                      lines[i].line, 1, "concurrency");
        }
        if (strstr(line, "rcu_read_unlock(") && rcu_depth > 0) {
            rcu_depth--;
        }

        if (strstr(line, "printk(") && !strstr(line, "KERN_")) {
            add_issue(issues, "printk-without-level", "warning",
                      "printk should include a KERN_ log level", lines[i].line, 1,
                      "style");
        }
        if (strstr(line, "BUG_ON(")) {
            add_issue(issues, "BUG_ON-usage", "medium",
                      "Prefer WARN_ON when the condition is recoverable",
                      lines[i].line, 1, "reliability");
        }
    }

    if ((saw_init && saw_exit) || saw_non_init_call_in_init || saw_init_call_in_non_init) {
        add_issue(issues, "missing-__init-__exit", "medium",
                  "Inconsistent __init/__exit annotation markers in this file", 1, 1,
                  "lifecycle");
=======
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
>>>>>>> feature/persistent-workers
    }
}

IssueList check_all_rules(const char *source_code) {
<<<<<<< HEAD
    IssueList issues = {0};
    LineView lines[8192];
    int count = split_lines(source_code, lines, 8192);
    check_style_rules(lines, count, &issues);
    check_kernel_rules(lines, count, &issues);
    free_lines(lines, count);
=======
    IssueList issues = {NULL, 0, 0};

    // Run all checks
    check_line_length(source_code, &issues);
    check_space_before_tab(source_code, &issues);
    check_trailing_whitespace(source_code, &issues);
    check_keyword_spacing(source_code, &issues);
    check_brace_style(source_code, &issues);

>>>>>>> feature/persistent-workers
    return issues;
}

void free_issue_list(IssueList *list) {
<<<<<<< HEAD
    free(list->items);
    list->items = NULL;
=======
    if (list->items) {
        free(list->items);
        list->items = NULL;
    }
>>>>>>> feature/persistent-workers
    list->count = 0;
    list->capacity = 0;
}
