#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include "rules.h"

#define BUFFER_SIZE 8192

typedef struct {
    char *content;
    size_t length;
} FileContent;

static FileContent read_file(const char *path) {
    FileContent fc = {NULL, 0};
    FILE *file = fopen(path, "r");
    if (!file) {
        return fc;
    }

    fseek(file, 0, SEEK_END);
    fc.length = ftell(file);
    fseek(file, 0, SEEK_SET);

    fc.content = malloc(fc.length + 1);
    if (!fc.content) {
        fclose(file);
        return fc;
    }

    fread(fc.content, 1, fc.length, file);
    fc.content[fc.length] = '\0';
    fclose(file);

    return fc;
}

static void print_json_issue(const Issue *issue) {
    printf("    {\n");
    printf("      \"rule\": \"%s\",\n", issue->rule);
    printf("      \"severity\": \"%s\",\n", issue->severity);
    printf("      \"message\": \"%s\",\n", issue->message);
    printf("      \"line\": %d,\n", issue->line);
    printf("      \"column\": %d\n", issue->column);
    printf("    }");
}

int main(int argc, char *argv[]) {
    if (argc != 2) {
        fprintf(stderr, "Usage: %s <file.c>\n", argv[0]);
        return 1;
    }

    const char *filepath = argv[1];
    FileContent fc = read_file(filepath);
    
    if (!fc.content) {
        fprintf(stderr, "Error: Cannot read file %s\n", filepath);
        return 1;
    }

    // Run rules
    IssueList issues = check_all_rules(fc.content);

    // Output JSON
    printf("{\n");
    printf("  \"file\": \"%s\",\n", filepath);
    printf("  \"issues\": [\n");
    
    for (size_t i = 0; i < issues.count; i++) {
        print_json_issue(&issues.items[i]);
        if (i < issues.count - 1) {
            printf(",");
        }
        printf("\n");
    }
    
    printf("  ]\n");
    printf("}\n");

    // Cleanup
    free_issue_list(&issues);
    free(fc.content);

    return 0;
}
