// SPDX-License-Identifier: GPL-2.0-only

#include <errno.h>
#include <sys/socket.h>
#include <sys/syscall.h>

int socket(int domain, int type, int protocol) {
    int ret = sys_socket(domain, type, protocol);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return ret;
}

int socketpair(int domain, int type, int protocol, int sv[2]) {
    int ret = sys_socketpair(domain, type, protocol, sv);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return 0;
}

int connect(int sockfd, const struct sockaddr *addr, socklen_t addrlen) {
    int ret = sys_connect(sockfd, (const void *)addr, (size_t)addrlen);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return ret;
}
