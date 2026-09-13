<!-- SPDX-License-Identifier: GPL-2.0-only -->

# BSD Socket API (`<sys/socket.h>`)

The `<sys/socket.h>` header provides standard BSD socket interfaces for network communication endpoints, protocol connections, and data transfer in Ring 3 userland.

---

## 1. Domain, Type & Protocol Constants

| Constant | Value | Description |
| :--- | :--- | :--- |
| `AF_UNIX` | `1` | Local Unix communication domain |
| `AF_INET` | `2` | IPv4 Internet protocols |
| `AF_INET6` | `10` | IPv6 Internet protocols |
| `SOCK_STREAM` | `1` | Sequenced, reliable, two-way connection-based byte streams (TCP) |
| `SOCK_DGRAM` | `2` | Connectionless, unreliable datagrams (UDP) |
| `SOCK_RAW` | `3` | Raw network protocol access |

---

## 2. Structures

### `struct sockaddr_in`
```c
struct in_addr {
    uint32_t s_addr;
};

struct sockaddr_in {
    uint16_t sin_family;
    uint16_t sin_port;
    struct in_addr sin_addr;
    char sin_zero[8];
};
```

---

## 3. Function Reference

### `socket`
```c
int socket(int domain, int type, int protocol);
```
Creates an unbound network communication endpoint in the specified domain and returns a file descriptor, or `-1` on error.

### `connect`
```c
int connect(int sockfd, const struct sockaddr *addr, socklen_t addrlen);
```
Connects the socket referenced by `sockfd` to the target address specified by `addr`. Returns `0` on success, or `-1` on error.

### `send`
```c
ssize_t send(int sockfd, const void *buf, size_t len, int flags);
```
Transmits a message from `buf` of length `len` across the connected socket. Returns the number of bytes transmitted, or `-1` on error.

### `recv`
```c
ssize_t recv(int sockfd, void *buf, size_t len, int flags);
```
Receives messages from a socket into `buf`. Returns the number of bytes received, or `-1` on error.
