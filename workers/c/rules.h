#ifndef RULES_H
#define RULES_H

#include <stddef.h>

typedef struct {
    char rule[64];
    char severity[16];
    char message[256];
    int line;
    int column;
} Issue;

typedef struct {
    Issue *items;
    size_t count;
    size_t capacity;
} IssueList;

// Check all rules and return list of issues
IssueList check_all_rules(const char *source_code);

// Free issue list
void free_issue_list(IssueList *list);

#endif // RULES_H
