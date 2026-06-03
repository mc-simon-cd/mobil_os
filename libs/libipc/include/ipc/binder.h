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
