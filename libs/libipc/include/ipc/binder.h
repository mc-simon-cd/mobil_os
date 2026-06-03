#ifndef IPC_BINDER_H
#define IPC_BINDER_H

#include "parcel.h"
#include <stdint.h>

#define IPC_SUCCESS 0
#define IPC_ERROR   -1

// Standard transaction command structures
typedef struct {
    int32_t code;          // Command operation integer
    uint32_t data_size;    // Payload byte size following header
} ipc_header_t;

// Client-facing routines
int ipc_connect(const char *socket_path);
int ipc_send_transaction(int fd, int32_t code, parcel_t *data, parcel_t *reply);

#endif // IPC_BINDER_H
