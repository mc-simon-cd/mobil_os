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

#define _GNU_SOURCE

#include "registry.h"
#include "ipc/binder.h"
#include "ipc/parcel.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/un.h>
#include <errno.h>

#define SOCKET_PATH "/tmp/servicemanager.sock"

#define CMD_REGISTER_SERVICE 1
#define CMD_GET_SERVICE      2

#define LOG_TAG "SERVICEMANAGER"
#define LOG_INFO(fmt, ...)  fprintf(stdout, "[INFO] [" LOG_TAG "] " fmt "\n", ##__VA_ARGS__)
#define LOG_WARN(fmt, ...)  fprintf(stderr, "[WARN] [" LOG_TAG "] " fmt "\n", ##__VA_ARGS__)
#define LOG_ERROR(fmt, ...) fprintf(stderr, "[ERR]  [" LOG_TAG "] " fmt "\n", ##__VA_ARGS__)

static void handle_client(int client_fd) {
    ipc_header_t header;
    ssize_t rec = read(client_fd, &header, sizeof(ipc_header_t));
    if (rec != sizeof(ipc_header_t)) {
        close(client_fd);
        return;
    }

    parcel_t data;
    parcel_init(&data);

    // Read payload parcel if size > 0
    if (header.data_size > 0) {
        char *buffer = malloc(header.data_size);
        if (buffer) {
            size_t read_bytes = 0;
            while (read_bytes < header.data_size) {
                ssize_t ret = read(client_fd, buffer + read_bytes, header.data_size - read_bytes);
                if (ret <= 0) {
                    free(buffer);
                    parcel_free(&data);
                    close(client_fd);
                    return;
                }
                read_bytes += ret;
            }
            parcel_write_raw(&data, buffer, header.data_size);
            free(buffer);
        }
    }

    parcel_t reply;
    parcel_init(&reply);

    // Command dispatch router
    if (header.code == CMD_REGISTER_SERVICE) {
        char name[64] = "";
        char path[128] = "";
        
        if (parcel_read_string(&data, name, sizeof(name)) == 0 &&
            parcel_read_string(&data, path, sizeof(path)) == 0) {
            
            int status = registry_register(name, path);
            LOG_INFO("Registered: '%s' -> %s", name, path);
            parcel_write_int32(&reply, status);
        } else {
            LOG_ERROR("Malformed register request received!");
            parcel_write_int32(&reply, -1);
        }
    } 
    else if (header.code == CMD_GET_SERVICE) {
        char name[64] = "";
        if (parcel_read_string(&data, name, sizeof(name)) == 0) {
            const char *resolved = registry_get(name);
            LOG_INFO("Resolved query: '%s' -> %s", name, resolved ? resolved : "NULL");
            parcel_write_string(&reply, resolved ? resolved : "");
        } else {
            LOG_ERROR("Malformed get request received!");
            parcel_write_string(&reply, "");
        }
    }
    else {
        LOG_WARN("Unknown command transaction code: %d", header.code);
        parcel_write_int32(&reply, -1);
    }

    // Send response back to the caller client
    ipc_header_t reply_header;
    reply_header.code = 0;
    reply_header.data_size = (uint32_t)reply.size;

    ssize_t sent = write(client_fd, &reply_header, sizeof(ipc_header_t));
    if (sent == sizeof(ipc_header_t) && reply_header.data_size > 0) {
        write(client_fd, reply.data, reply.size);
    }

    parcel_free(&data);
    parcel_free(&reply);
    close(client_fd);
}

int main(int argc, char *argv[]) {
    (void)argc;
    (void)argv;

    LOG_INFO("Starting IPC Service Manager...");

    // Unlink old sockets if present
    unlink(SOCKET_PATH);

    int server_fd = socket(AF_UNIX, SOCK_STREAM, 0);
    if (server_fd < 0) {
        LOG_ERROR("Failed to create UNIX domain socket: %s", strerror(errno));
        return 1;
    }

    struct sockaddr_un addr;
    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    strncpy(addr.sun_path, SOCKET_PATH, sizeof(addr.sun_path) - 1);

    if (bind(server_fd, (struct sockaddr *)&addr, sizeof(addr)) != 0) {
        LOG_ERROR("Failed to bind socket to %s: %s", SOCKET_PATH, strerror(errno));
        close(server_fd);
        return 1;
    }

    // Set permission masks so system & ui users can connect dynamically
    chmod(SOCKET_PATH, 0666);

    if (listen(server_fd, 8) != 0) {
        LOG_ERROR("Failed to listen on socket: %s", strerror(errno));
        close(server_fd);
        return 1;
    }

    registry_init();
    LOG_INFO("Listening for IPC service registrations at %s", SOCKET_PATH);

    while (1) {
        int client_fd = accept(server_fd, NULL, NULL);
        if (client_fd >= 0) {
            handle_client(client_fd);
        }
    }

    close(server_fd);
    unlink(SOCKET_PATH);
    return 0;
}
