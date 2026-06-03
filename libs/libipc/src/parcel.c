/*
 * Copyright 2026 mcsimon
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#include "ipc/parcel.h"
#include <stdlib.h>
#include <string.h>

#define INITIAL_CAPACITY 64

void parcel_init(parcel_t *p) {
    if (!p) return;
    p->data = malloc(INITIAL_CAPACITY);
    p->capacity = INITIAL_CAPACITY;
    p->size = 0;
    p->read_pos = 0;
}

void parcel_free(parcel_t *p) {
    if (!p) return;
    free(p->data);
    p->data = NULL;
    p->capacity = 0;
    p->size = 0;
    p->read_pos = 0;
}

static int parcel_ensure_capacity(parcel_t *p, size_t needed) {
    size_t new_size = p->size + needed;
    if (new_size <= p->capacity) {
        return 0;
    }

    size_t new_capacity = p->capacity * 2;
    while (new_capacity < new_size) {
        new_capacity *= 2;
    }

    char *new_data = realloc(p->data, new_capacity);
    if (!new_data) {
        return -1;
    }

    p->data = new_data;
    p->capacity = new_capacity;
    return 0;
}

int parcel_write_raw(parcel_t *p, const void *data, size_t len) {
    if (!p || !data || len == 0) return -1;
    
    if (parcel_ensure_capacity(p, len) != 0) {
        return -1;
    }

    memcpy(p->data + p->size, data, len);
    p->size += len;
    return 0;
}

int parcel_write_int32(parcel_t *p, int32_t val) {
    return parcel_write_raw(p, &val, sizeof(int32_t));
}

int parcel_write_uint32(parcel_t *p, uint32_t val) {
    return parcel_write_raw(p, &val, sizeof(uint32_t));
}

int parcel_write_string(parcel_t *p, const char *str) {
    if (!p) return -1;
    if (!str) {
        // Write string length 0
        uint32_t len = 0;
        return parcel_write_uint32(p, len);
    }

    uint32_t len = (uint32_t)strlen(str) + 1; // Include null terminator
    if (parcel_write_uint32(p, len) != 0) {
        return -1;
    }

    return parcel_write_raw(p, str, len);
}

int parcel_read_raw(parcel_t *p, void *buf, size_t len) {
    if (!p || !buf || len == 0) return -1;
    
    if (p->read_pos + len > p->size) {
        return -1; // Out of bounds read attempt
    }

    memcpy(buf, p->data + p->read_pos, len);
    p->read_pos += len;
    return 0;
}

int parcel_read_int32(parcel_t *p, int32_t *val) {
    return parcel_read_raw(p, val, sizeof(int32_t));
}

int parcel_read_uint32(parcel_t *p, uint32_t *val) {
    return parcel_read_raw(p, val, sizeof(uint32_t));
}

int parcel_read_string(parcel_t *p, char *buf, size_t buf_len) {
    if (!p || !buf || buf_len == 0) return -1;

    uint32_t len;
    if (parcel_read_uint32(p, &len) != 0) {
        return -1;
    }

    if (len == 0) {
        buf[0] = '\0';
        return 0;
    }

    if (p->read_pos + len > p->size) {
        return -1; // Insufficient remaining bytes
    }

    if (len > buf_len) {
        return -1; // Buffer too small for string target
    }

    return parcel_read_raw(p, buf, len);
}

void parcel_reset_read(parcel_t *p) {
    if (p) p->read_pos = 0;
}

void parcel_reset_write(parcel_t *p) {
    if (p) {
        p->size = 0;
        p->read_pos = 0;
    }
}
