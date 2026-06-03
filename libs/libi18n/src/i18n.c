#include "i18n.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    char *key;
    char *value;
} translation_entry_t;

static translation_entry_t *g_entries = NULL;
static size_t g_count = 0;
static size_t g_capacity = 0;

static char *trim(char *str) {
    if (!str) return NULL;
    // Trim leading whitespace
    while (*str == ' ' || *str == '\t' || *str == '\r' || *str == '\n') {
        str++;
    }
    if (*str == '\0') return str;
    // Trim trailing whitespace
    char *end = str + strlen(str) - 1;
    while (end > str && (*end == ' ' || *end == '\t' || *end == '\r' || *end == '\n')) {
        *end = '\0';
        end--;
    }
    return str;
}

static int add_entry(const char *key, const char *value) {
    if (!key || !value) return -1;
    
    if (g_count >= g_capacity) {
        size_t new_capacity = g_capacity == 0 ? 32 : g_capacity * 2;
        translation_entry_t *new_entries = realloc(g_entries, new_capacity * sizeof(translation_entry_t));
        if (!new_entries) return -1;
        g_entries = new_entries;
        g_capacity = new_capacity;
    }
    
    char *dup_key = strdup(key);
    char *dup_value = strdup(value);
    if (!dup_key || !dup_value) {
        free(dup_key);
        free(dup_value);
        return -1;
    }
    
    g_entries[g_count].key = dup_key;
    g_entries[g_count].value = dup_value;
    g_count++;
    return 0;
}

int i18n_init(const char *locale_dir, const char *locale_name) {
    // 1. Clean up existing resources if initialized
    i18n_free();
    
    if (!locale_dir || !locale_name) return -1;
    
    // 2. Construct file path
    char file_path[256];
    snprintf(file_path, sizeof(file_path), "%s/%s.txt", locale_dir, locale_name);
    
    FILE *file = fopen(file_path, "r");
    if (!file) {
        return -1;
    }
    
    // 3. Parse file line by line
    char line_buffer[512];
    while (fgets(line_buffer, sizeof(line_buffer), file)) {
        char *trimmed = trim(line_buffer);
        // Ignore comments and empty lines
        if (trimmed[0] == '\0' || trimmed[0] == '#') {
            continue;
        }
        
        char *separator = strchr(trimmed, '=');
        if (separator) {
            *separator = '\0';
            char *key = trim(trimmed);
            char *value = trim(separator + 1);
            if (key && value && *key != '\0') {
                add_entry(key, value);
            }
        }
    }
    
    fclose(file);
    return 0;
}

const char *i18n_get(const char *key) {
    if (!key || !g_entries) return key;
    for (size_t i = 0; i < g_count; i++) {
        if (strcmp(g_entries[i].key, key) == 0) {
            return g_entries[i].value;
        }
    }
    return key; // Graceful fallback to the key itself
}

void i18n_free(void) {
    if (g_entries) {
        for (size_t i = 0; i < g_count; i++) {
            free(g_entries[i].key);
            free(g_entries[i].value);
        }
        free(g_entries);
        g_entries = NULL;
    }
    g_count = 0;
    g_capacity = 0;
}
