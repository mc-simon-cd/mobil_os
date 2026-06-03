#include "init.h"
#include <string.h>

static property_t properties[MAX_PROPERTIES];
static int property_count = 0;

int property_init(void) {
    property_count = 0;
    
    // Write standard default system attributes
    property_set("ro.product.name", "Mobile OS");
    property_set("ro.product.device", "qemu-arm64");
    property_set("ro.build.version", "1.0-alpha");
    property_set("sys.boot_completed", "0");
    
    LOG_INFO("System property database initialized.");
    return 0;
}

int property_set(const char *key, const char *value) {
    if (!key || !value) return -1;

    // Search if property already exists
    for (int i = 0; i < property_count; i++) {
        if (strcmp(properties[i].key, key) == 0) {
            // Read-Only check (Keys beginning with 'ro.' cannot be written twice)
            if (strncmp(key, "ro.", 3) == 0) {
                LOG_WARN("Attempted write to read-only property '%s' rejected.", key);
                return -1;
            }
            strncpy(properties[i].value, value, sizeof(properties[i].value) - 1);
            properties[i].value[sizeof(properties[i].value) - 1] = '\0';
            return 0;
        }
    }

    // Key doesn't exist, create it
    if (property_count >= MAX_PROPERTIES) {
        LOG_ERROR("Property store limit reached!");
        return -1;
    }

    strncpy(properties[property_count].key, key, sizeof(properties[property_count].key) - 1);
    properties[property_count].key[sizeof(properties[property_count].key) - 1] = '\0';
    
    strncpy(properties[property_count].value, value, sizeof(properties[property_count].value) - 1);
    properties[property_count].value[sizeof(properties[property_count].value) - 1] = '\0';
    
    property_count++;
    return 0;
}

const char* property_get(const char *key) {
    if (!key) return NULL;
    
    for (int i = 0; i < property_count; i++) {
        if (strcmp(properties[i].key, key) == 0) {
            return properties[i].value;
        }
    }
    return NULL;
}
