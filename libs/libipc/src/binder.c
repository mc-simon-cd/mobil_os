#include "ipc/binder.h"
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>
#include <string.h>
#include <errno.h>

// Forward declaration of parcel internal grow utility
int parcel_write_raw(parcel_t *p, const void *data, size_t len);

int ipc_connect(const char *socket_path) {
    if (!socket_path) return -1;

    int fd = socket(AF_UNIX, SOCK_STREAM, 0);
    if (fd < 0) return -1;

    struct sockaddr_un addr;
    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    strncpy(addr.sun_path, socket_path, sizeof(addr.sun_path) - 1);

    if (connect(fd, (struct sockaddr *)&addr, sizeof(addr)) != 0) {
        close(fd);
        return -1;
    }

    return fd;
}

int ipc_send_transaction(int fd, int32_t code, parcel_t *data, parcel_t *reply) {
    if (fd < 0) return IPC_ERROR;

    // 1. Prepare and send IPC Transaction header
    ipc_header_t header;
    header.code = code;
    header.data_size = data ? (uint32_t)data->size : 0;

    ssize_t sent = write(fd, &header, sizeof(ipc_header_t));
    if (sent != sizeof(ipc_header_t)) {
        return IPC_ERROR;
    }

    // 2. Send parcel payload if size > 0
    if (header.data_size > 0 && data && data->data) {
        size_t written = 0;
        while (written < header.data_size) {
            ssize_t ret = write(fd, data->data + written, header.data_size - written);
            if (ret <= 0) {
                return IPC_ERROR;
            }
            written += ret;
        }
    }

    // 3. Read transaction response header
    ipc_header_t reply_header;
    ssize_t received = read(fd, &reply_header, sizeof(ipc_header_t));
    if (received != sizeof(ipc_header_t)) {
        return IPC_ERROR;
    }

    // 4. Read response payload parcel if size > 0
    if (reply_header.data_size > 0 && reply) {
        parcel_reset_write(reply);
        
        // Dynamically receive payload bytes in chunks
        char temp_buffer[128];
        size_t read_bytes = 0;
        
        while (read_bytes < reply_header.data_size) {
            size_t to_read = reply_header.data_size - read_bytes;
            if (to_read > sizeof(temp_buffer)) {
                to_read = sizeof(temp_buffer);
            }
            
            ssize_t ret = read(fd, temp_buffer, to_read);
            if (ret <= 0) {
                return IPC_ERROR;
            }
            
            if (parcel_write_raw(reply, temp_buffer, ret) != 0) {
                return IPC_ERROR;
            }
            read_bytes += ret;
        }
    }

    return IPC_SUCCESS;
}
