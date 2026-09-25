<!-- SPDX-License-Identifier: GPL-2.0-only -->

# BSD Sockets Interface (`userland/lib/socket/`)

Standard POSIX socket API for network and local IPC communication (`userland/include/socket.h`).

---

## Supported APIs

```c
int socket(int domain, int type, int protocol);
int bind(int sockfd, const struct sockaddr *addr, socklen_t addrlen);
int connect(int sockfd, const struct sockaddr *addr, socklen_t addrlen);
int listen(int sockfd, int backlog);
int accept(int sockfd, struct sockaddr *addr, socklen_t *addrlen);
ssize_t send(int sockfd, const void *buf, size_t len, int flags);
ssize_t recv(int sockfd, void *buf, size_t len, int flags);
```

* Domains: `AF_INET` (IPv4), `AF_UNIX` (Local IPC).
* Types: `SOCK_STREAM` (TCP), `SOCK_DGRAM` (UDP), `SOCK_RAW` (Raw IP).
