#include <ctype.h>
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "rules.h"

#define PROTOCOL_VERSION "1.0.0"
#define MAX_BATCH_FILES 256
#define MAX_THREADS 4

typedef struct {
    char *path;
    char *language;
    char *content;
} FileRequest;

typedef struct {
    FileRequest *files;
    IssueList *results;
    size_t count;
    size_t next_index;
    pthread_mutex_t mutex;
} ScanQueue;

static char *dup_string(const char *input) {
    size_t len = strlen(input);
    char *copy = malloc(len + 1);
    if (!copy) {
        return NULL;
    }
    memcpy(copy, input, len + 1);
    return copy;
}

static char *read_all_stdin(void) {
    size_t capacity = 8192;
    size_t length = 0;
    char *buffer = malloc(capacity);
    if (!buffer) {
        return NULL;
    }

    int ch;
    while ((ch = getchar()) != EOF) {
        if (length + 1 >= capacity) {
            capacity *= 2;
            char *next = realloc(buffer, capacity);
            if (!next) {
                free(buffer);
                return NULL;
            }
            buffer = next;
        }
        buffer[length++] = (char)ch;
    }
    buffer[length] = '\0';
    return buffer;
}

static void json_escape_print(const char *s) {
    for (; *s; s++) {
        switch (*s) {
        case '\\':
            fputs("\\\\", stdout);
            break;
        case '"':
            fputs("\\\"", stdout);
            break;
        case '\n':
            fputs("\\n", stdout);
            break;
        case '\r':
            fputs("\\r", stdout);
            break;
        case '\t':
            fputs("\\t", stdout);
            break;
        default:
            putchar((unsigned char)*s);
            break;
        }
    }
}

static char *extract_string_field(const char *json, const char *key) {
    char pattern[128];
    snprintf(pattern, sizeof(pattern), "\"%s\"", key);
    const char *start = strstr(json, pattern);
    if (!start) {
        return NULL;
    }
    start = strchr(start + strlen(pattern), ':');
    if (!start) {
        return NULL;
    }
    start++;
    while (*start && isspace((unsigned char)*start)) {
        start++;
    }
    if (*start != '"') {
        return NULL;
    }
    start++;
    const char *end = start;
    while (*end && *end != '"') {
        if (*end == '\\' && end[1]) {
            end++;
        }
        end++;
    }
    size_t len = (size_t)(end - start);
    char *out = malloc(len + 1);
    if (!out) {
        return NULL;
    }
    size_t write_idx = 0;
    for (size_t read_idx = 0; read_idx < len; read_idx++) {
        char ch = start[read_idx];
        if (ch == '\\' && read_idx + 1 < len) {
            read_idx++;
            switch (start[read_idx]) {
            case 'n':
                out[write_idx++] = '\n';
                break;
            case 'r':
                out[write_idx++] = '\r';
                break;
            case 't':
                out[write_idx++] = '\t';
                break;
            case '\\':
                out[write_idx++] = '\\';
                break;
            case '"':
                out[write_idx++] = '"';
                break;
            default:
                out[write_idx++] = start[read_idx];
                break;
            }
        } else {
            out[write_idx++] = ch;
        }
    }
    out[write_idx] = '\0';
    return out;
}

static char *extract_params_object(const char *json) {
    const char *params = strstr(json, "\"params\"");
    if (!params) {
        return NULL;
    }
    const char *start = strchr(params, '{');
    if (!start) {
        return NULL;
    }
    int depth = 0;
    const char *end = start;
    while (*end) {
        if (*end == '{') {
            depth++;
        } else if (*end == '}') {
            depth--;
            if (depth == 0) {
                size_t len = (size_t)(end - start + 1);
                char *copy = malloc(len + 1);
                if (!copy) {
                    return NULL;
                }
                memcpy(copy, start, len);
                copy[len] = '\0';
                return copy;
            }
        }
        end++;
    }
    return NULL;
}

static int extract_scan_files(const char *json, FileRequest *files, size_t *count) {
    const char *files_key = strstr(json, "\"files\"");
    if (!files_key) {
        return -1;
    }
    const char *cursor = strchr(files_key, '[');
    if (!cursor) {
        return -1;
    }
    cursor++;
    *count = 0;

    while (*cursor && *cursor != ']') {
        while (*cursor && (isspace((unsigned char)*cursor) || *cursor == ',')) {
            cursor++;
        }
        if (*cursor != '{') {
            break;
        }
        const char *obj_start = cursor;
        int depth = 0;
        while (*cursor) {
            if (*cursor == '{') {
                depth++;
            } else if (*cursor == '}') {
                depth--;
                if (depth == 0) {
                    size_t len = (size_t)(cursor - obj_start + 1);
                    char *obj = malloc(len + 1);
                    if (!obj) {
                        return -1;
                    }
                    memcpy(obj, obj_start, len);
                    obj[len] = '\0';
                    if (*count >= MAX_BATCH_FILES) {
                        free(obj);
                        return -1;
                    }
                    files[*count].path = extract_string_field(obj, "path");
                    files[*count].language = extract_string_field(obj, "language");
                    files[*count].content = extract_string_field(obj, "content");
                    free(obj);
                    if (!files[*count].path || !files[*count].content) {
                        return -1;
                    }
                    (*count)++;
                    cursor++;
                    break;
                }
            }
            cursor++;
        }
    }
    return 0;
}

static void print_findings(IssueList *issues) {
    for (size_t i = 0; i < issues->count; i++) {
        if (i) {
            printf(",");
        }
        printf("{\"severity\":\"");
        json_escape_print(issues->items[i].severity);
        printf("\",\"rule\":\"");
        json_escape_print(issues->items[i].rule);
        printf("\",\"message\":\"");
        json_escape_print(issues->items[i].message);
        printf("\",\"line\":%d,\"column\":%d,\"category\":\"", issues->items[i].line,
               issues->items[i].column);
        json_escape_print(issues->items[i].category);
        printf("\"}");
    }
}

static void print_single_response(const char *request_id, IssueList *issues) {
    printf("{\"schema_version\":\"2\",\"protocol_version\":\"%s\",\"request_id\":\"",
           PROTOCOL_VERSION);
    json_escape_print(request_id ? request_id : "");
    printf("\",\"status\":\"ok\",\"findings\":[");
    print_findings(issues);
    printf("]}\n");
}

static void print_batch_response(const char *request_id, FileRequest *files, IssueList *results,
                                 size_t count) {
    printf("{\"schema_version\":\"2\",\"protocol_version\":\"%s\",\"request_id\":\"",
           PROTOCOL_VERSION);
    json_escape_print(request_id ? request_id : "");
    printf("\",\"status\":\"ok\",\"results\":[");
    for (size_t i = 0; i < count; i++) {
        if (i) {
            printf(",");
        }
        printf("{\"path\":\"");
        json_escape_print(files[i].path);
        printf("\",\"findings\":[");
        print_findings(&results[i]);
        printf("]}");
    }
    printf("]}\n");
}

static void *scan_worker(void *arg) {
    ScanQueue *queue = (ScanQueue *)arg;
    for (;;) {
        size_t index;
        pthread_mutex_lock(&queue->mutex);
        if (queue->next_index >= queue->count) {
            pthread_mutex_unlock(&queue->mutex);
            return NULL;
        }
        index = queue->next_index++;
        pthread_mutex_unlock(&queue->mutex);
        queue->results[index] = check_all_rules(queue->files[index].content);
    }
}

static void free_file_request(FileRequest *file) {
    free(file->path);
    free(file->language);
    free(file->content);
    file->path = NULL;
    file->language = NULL;
    file->content = NULL;
}

int main(void) {
    char *input = read_all_stdin();
    char *request_id = NULL;
    char *method = NULL;
    char *content = NULL;

    if (!input) {
        return 1;
    }

    request_id = extract_string_field(input, "request_id");
    method = extract_string_field(input, "method");
    content = extract_string_field(input, "content");

    if (!request_id) {
        request_id = dup_string("");
    }
    if (!method) {
        method = dup_string("");
    }

    if (strcmp(method, "hello") == 0) {
        printf("{\"schema_version\":\"2\",\"protocol_version\":\"%s\",\"request_id\":\"",
               PROTOCOL_VERSION);
        json_escape_print(request_id);
        printf("\",\"status\":\"ok\",\"findings\":[]}\n");
    } else if (strcmp(method, "analyze_file") == 0) {
        if (!content) {
            content = dup_string("");
        }
        IssueList issues = check_all_rules(content);
        print_single_response(request_id, &issues);
        free_issue_list(&issues);
    } else if (strcmp(method, "scan_files") == 0) {
        FileRequest files[MAX_BATCH_FILES];
        IssueList results[MAX_BATCH_FILES];
        size_t count = 0;
        memset(files, 0, sizeof(files));
        memset(results, 0, sizeof(results));
        char *params = extract_params_object(input);

        if (!params || extract_scan_files(params, files, &count) != 0) {
            printf("{\"schema_version\":\"2\",\"protocol_version\":\"%s\",\"request_id\":\"",
                   PROTOCOL_VERSION);
            json_escape_print(request_id);
            printf("\",\"status\":\"error\",\"findings\":[],\"error\":\"invalid scan_files request\"}\n");
            free(params);
        } else {
            ScanQueue queue;
            pthread_t threads[MAX_THREADS];
            size_t thread_count = count < MAX_THREADS ? count : MAX_THREADS;

            queue.files = files;
            queue.results = results;
            queue.count = count;
            queue.next_index = 0;
            pthread_mutex_init(&queue.mutex, NULL);

            for (size_t i = 0; i < thread_count; i++) {
                pthread_create(&threads[i], NULL, scan_worker, &queue);
            }
            for (size_t i = 0; i < thread_count; i++) {
                pthread_join(threads[i], NULL);
            }
            pthread_mutex_destroy(&queue.mutex);

            print_batch_response(request_id, files, results, count);
            for (size_t i = 0; i < count; i++) {
                free_issue_list(&results[i]);
                free_file_request(&files[i]);
            }
            free(params);
        }
    } else {
        printf("{\"schema_version\":\"2\",\"protocol_version\":\"%s\",\"request_id\":\"",
               PROTOCOL_VERSION);
        json_escape_print(request_id);
        printf("\",\"status\":\"error\",\"findings\":[],\"error\":\"unknown request\"}\n");
    }

    free(input);
    free(request_id);
    free(method);
    free(content);
    return 0;
}
