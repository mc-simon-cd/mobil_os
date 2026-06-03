#ifndef IPC_PARCEL_H
#define IPC_PARCEL_H

#include <stddef.h>
#include <stdint.h>

// Dynamically sized packaging buffer
typedef struct {
    char *data;          // Raw byte array
    size_t capacity;     // Total allocated memory in bytes
    size_t size;         // Total written bytes
    size_t read_pos;     // Cursor index for read operations
} parcel_t;

// Lifecycle handlers
void parcel_init(parcel_t *p);
void parcel_free(parcel_t *p);

// Write operations (Serialize)
int parcel_write_int32(parcel_t *p, int32_t val);
int parcel_write_uint32(parcel_t *p, uint32_t val);
int parcel_write_string(parcel_t *p, const char *str);
int parcel_write_raw(parcel_t *p, const void *data, size_t len);

// Read operations (Deserialize)
int parcel_read_int32(parcel_t *p, int32_t *val);
int parcel_read_uint32(parcel_t *p, uint32_t *val);
int parcel_read_string(parcel_t *p, char *buf, size_t buf_len);
int parcel_read_raw(parcel_t *p, void *buf, size_t len);

// Utility handlers
void parcel_reset_read(parcel_t *p);
void parcel_reset_write(parcel_t *p);

#endif // IPC_PARCEL_H
