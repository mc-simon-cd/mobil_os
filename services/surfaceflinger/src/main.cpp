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

#include "composer.h"
#include "../wl/wl_compositor.h"
#include <iostream>
#include <cstdlib>
#include <cstring>
#include <thread>
#include <unistd.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/un.h>
#include <errno.h>

// Wrap C library headers to avoid name mangling
extern "C" {
#include "ipc/binder.h"
#include "ipc/parcel.h"
}

#define SERVICEMANAGER_SOCKET "/tmp/servicemanager.sock"
#define SURFACEFLINGER_SOCKET "/tmp/surfaceflinger.sock"

#define CMD_REGISTER_SERVICE 1

#define CMD_ALLOC_SURFACE   1
#define CMD_COMPOSITE       2
#define CMD_DESTROY_SURFACE 3

static Composer gComposer;

static void register_with_servicemanager() {
    std::cout << "[INFO] [SURFACEFLINGER] Registering with servicemanager..." << std::endl;
    
    int fd = ipc_connect(SERVICEMANAGER_SOCKET);
    if (fd < 0) {
        std::cerr << "[WARN] [SURFACEFLINGER] Servicemanager not running. Proceeding standalone." << std::endl;
        return;
    }

    parcel_t data;
    parcel_init(&data);
    parcel_write_string(&data, "mobile.surfaceflinger");
    parcel_write_string(&data, SURFACEFLINGER_SOCKET);

    parcel_t reply;
    parcel_init(&reply);

    if (ipc_send_transaction(fd, CMD_REGISTER_SERVICE, &data, &reply) == IPC_SUCCESS) {
        int32_t status = -1;
        parcel_read_int32(&reply, &status);
        if (status == 0) {
            std::cout << "[INFO] [SURFACEFLINGER] Registered successfully in system directory." << std::endl;
        } else {
            std::cerr << "[WARN] [SURFACEFLINGER] Registration failed with status: " << status << std::endl;
        }
    } else {
        std::cerr << "[WARN] [SURFACEFLINGER] IPC register transaction failed!" << std::endl;
    }

    close(fd);
    parcel_free(&data);
    parcel_free(&reply);
}

static void handle_client(int client_fd) {
    ipc_header_t header;
    ssize_t rec = read(client_fd, &header, sizeof(ipc_header_t));
    if (rec != sizeof(ipc_header_t)) {
        close(client_fd);
        return;
    }

    parcel_t data;
    parcel_init(&data);

    // Read client payload
    if (header.data_size > 0) {
        char *buffer = (char *)malloc(header.data_size);
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

    // Command transaction router
    if (header.code == CMD_ALLOC_SURFACE) {
        int32_t w = 0, h = 0, fmt = 0;
        if (parcel_read_int32(&data, &w) == 0 &&
            parcel_read_int32(&data, &h) == 0 &&
            parcel_read_int32(&data, &fmt) == 0) {
            
            int32_t sid = gComposer.allocateSurface(w, h, fmt);
            parcel_write_int32(&reply, sid);
        } else {
            std::cerr << "[ERR] [SURFACEFLINGER] Malformed CMD_ALLOC_SURFACE request!" << std::endl;
            parcel_write_int32(&reply, -1);
        }
    }
    else if (header.code == CMD_COMPOSITE) {
        int status = gComposer.compositeLayers();
        parcel_write_int32(&reply, status);
    }
    else if (header.code == CMD_DESTROY_SURFACE) {
        int32_t sid = -1;
        if (parcel_read_int32(&data, &sid) == 0) {
            int status = gComposer.destroySurface(sid);
            parcel_write_int32(&reply, status);
        } else {
            std::cerr << "[ERR] [SURFACEFLINGER] Malformed CMD_DESTROY_SURFACE request!" << std::endl;
            parcel_write_int32(&reply, -1);
        }
    }
    else {
        std::cerr << "[WARN] [SURFACEFLINGER] Unknown CMD transaction code: " << header.code << std::endl;
        parcel_write_int32(&reply, -1);
    }

    // Send reply header and payload back
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
    bool use_wayland = false;
    for (int i = 1; i < argc; ++i) {
        if (strcmp(argv[i], "--wayland") == 0) {
            use_wayland = true;
            break;
        }
    }

    if (use_wayland) {
        std::cout << "[INFO] [SURFACEFLINGER] Starting Surfaceflinger in Wayland Compositor mode..." << std::endl;
        if (!wl_compositor_init()) {
            std::cerr << "[ERR] [SURFACEFLINGER] Failed to initialize Wayland compositor. Exiting." << std::endl;
            return 1;
        }

        // Also register with servicemanager and open IPC socket for touch injection
        register_with_servicemanager();
        unlink(SURFACEFLINGER_SOCKET);
        int wl_ipc_fd = socket(AF_UNIX, SOCK_STREAM, 0);
        if (wl_ipc_fd >= 0) {
            struct sockaddr_un wl_addr;
            memset(&wl_addr, 0, sizeof(wl_addr));
            wl_addr.sun_family = AF_UNIX;
            strncpy(wl_addr.sun_path, SURFACEFLINGER_SOCKET, sizeof(wl_addr.sun_path) - 1);
            if (bind(wl_ipc_fd, (struct sockaddr *)&wl_addr, sizeof(wl_addr)) == 0) {
                chmod(SURFACEFLINGER_SOCKET, 0666);
                listen(wl_ipc_fd, 8);
                std::cout << "[INFO] [SURFACEFLINGER] Wayland IPC touch socket: " << SURFACEFLINGER_SOCKET << std::endl;

                // Background thread: accept touch injection IPC requests
                std::thread ipc_thread([wl_ipc_fd]() {
                    while (true) {
                        int cfd = accept(wl_ipc_fd, NULL, NULL);
                        if (cfd < 0) break;

                        ipc_header_t hdr;
                        ssize_t rec = read(cfd, &hdr, sizeof(hdr));
                        if (rec == (ssize_t)sizeof(hdr)) {
                            parcel_t data;
                            parcel_init(&data);
                            if (hdr.data_size > 0) {
                                char *buf = (char *)malloc(hdr.data_size);
                                if (buf) {
                                    size_t rb = 0;
                                    while (rb < hdr.data_size) {
                                        ssize_t r = read(cfd, buf + rb, hdr.data_size - rb);
                                        if (r <= 0) break;
                                        rb += r;
                                    }
                                    parcel_write_raw(&data, buf, hdr.data_size);
                                    free(buf);
                                }
                            }

                            // CMD_SEND_INPUT_EVENT = 4 (Wayland touch injection)
                            if (hdr.code == 4) {
                                int32_t ev_type = 0, ev_code = 0, ev_val = 0;
                                parcel_read_int32(&data, &ev_type);
                                parcel_read_int32(&data, &ev_code);
                                parcel_read_int32(&data, &ev_val);
                                wl_compositor_inject_touch(ev_type, ev_code, ev_val);
                            }

                            parcel_t reply;
                            parcel_init(&reply);
                            parcel_write_int32(&reply, 0);
                            ipc_header_t rh;
                            rh.code = 0;
                            rh.data_size = (uint32_t)reply.size;
                            write(cfd, &rh, sizeof(rh));
                            if (reply.size > 0) write(cfd, reply.data, reply.size);
                            parcel_free(&reply);
                            parcel_free(&data);
                        }
                        close(cfd);
                    }
                    close(wl_ipc_fd);
                });
                ipc_thread.detach();
            } else {
                close(wl_ipc_fd);
            }
        }

        wl_compositor_run();
        wl_compositor_cleanup();
        return 0;
    }

    std::cout << "[INFO] [SURFACEFLINGER] Starting Surfaceflinger Graphics Daemon in legacy PPM mode..." << std::endl;

    // Registers display path in background
    register_with_servicemanager();

    unlink(SURFACEFLINGER_SOCKET);

    int server_fd = socket(AF_UNIX, SOCK_STREAM, 0);
    if (server_fd < 0) {
        std::cerr << "[ERR] [SURFACEFLINGER] Failed to create socket: " << strerror(errno) << std::endl;
        return 1;
    }

    struct sockaddr_un addr;
    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    strncpy(addr.sun_path, SURFACEFLINGER_SOCKET, sizeof(addr.sun_path) - 1);

    if (bind(server_fd, (struct sockaddr *)&addr, sizeof(addr)) != 0) {
        std::cerr << "[ERR] [SURFACEFLINGER] Failed to bind to " << SURFACEFLINGER_SOCKET << ": " << strerror(errno) << std::endl;
        close(server_fd);
        return 1;
    }

    chmod(SURFACEFLINGER_SOCKET, 0666);

    if (listen(server_fd, 8) != 0) {
        std::cerr << "[ERR] [SURFACEFLINGER] Failed to listen: " << strerror(errno) << std::endl;
        close(server_fd);
        return 1;
    }

    std::cout << "[INFO] [SURFACEFLINGER] Graphics Server listening at " << SURFACEFLINGER_SOCKET << std::endl;

    while (1) {
        int client_fd = accept(server_fd, NULL, NULL);
        if (client_fd >= 0) {
            handle_client(client_fd);
        }
    }

    close(server_fd);
    unlink(SURFACEFLINGER_SOCKET);
    return 0;
}
