#ifndef REGISTRY_H
#define REGISTRY_H

#define MAX_REGISTRY_ENTRIES 32

typedef struct {
    char name[64];
    char socket_path[128];
} service_entry_t;

// Registry lifecycle database hooks
int registry_init(void);
int registry_register(const char *name, const char *path);
const char* registry_get(const char *name);

#endif // REGISTRY_H
