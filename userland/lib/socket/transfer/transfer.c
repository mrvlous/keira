// SPDX-License-Identifier: GPL-2.0-only

#include <sys/socket.h>
#include <unistd.h>

ssize_t send(int sockfd, const void *buf, size_t len, int flags) {
    (void)flags;
    return write(sockfd, buf, len);
}

ssize_t recv(int sockfd, void *buf, size_t len, int flags) {
    (void)flags;
    return read(sockfd, buf, len);
}
