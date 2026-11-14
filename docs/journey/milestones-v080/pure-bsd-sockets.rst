.. SPDX-License-Identifier: GPL-2.0-only


Milestone 54: Pure Freestanding Monolithic Kernel Demarcation, Layer 7 HTTP Ring 0 Purge, BSD Socket Networking Engine & Canonical FHS Purification
===================================================================================================================================================

1. Executive Summary
====================

Milestone 54 purges residual distribution-level preconceptions from Keira Kernel, aligning Ring 0 strictly with the classical monolithic kernel architecture exemplified by Linux (``torvalds/linux``). Under this model, Ring 0 is restricted strictly to an austere hardware supervisor, memory manager, process scheduler, Virtual File System (VFS) and low-level protocol engine (Layers 2-4: Ethernet, ARP, IPv4, ICMP, UDP, TCP and UNIX domain sockets).

All Layer 7 application protocols (specifically HTTP/1.1 client request generation, URL parsing, header inspection, user agent constants and response dechunking) are permanently purged from Ring 0. The BSD socket subsystem in ``crates/net`` is expanded so ``socket()``, ``connect()``, ``write()`` and ``read()`` handle external TCP networking natively. The userspace ``fetch`` core utility (``userland/netutils/fetch.c``) is re-engineered as a freestanding Ring 3 client using standard POSIX sockets and ``<arpa/inet.h>``. Rootfs distribution artifacts (such as ``/src``) are purged in favor of canonical Filesystem Hierarchy Standard (FHS) paths (``/usr/src``).

Through Milestone 54, Keira achieves:

1. **Layer 7 HTTP Ring 0 Eradication**: Complete removal of ``crates/net/src/tcp/stream/http.rs``, ``HTTP_USER_AGENT`` constants, ``fetch_http``, ``fetch_http_stream`` and ``dechunk_in_place``. Over 262 KB of kernel BSS memory (``STREAM_DOWNLOAD_BUFFER``) was permanently reclaimed.
2. **Syscall 17 Deprecation**: Vector 17 (``SYS_HTTP`` / ``SYS_HTTP_GET``) is deprecated and mapped directly to ``ENOSYS``, enforcing the invariant that kernels do not expose application protocol system calls.
3. **Pure L4 BSD Socket TCP Transport**: ``crates/net/src/socket/table.rs`` routes non-loopback ``AF_INET`` stream socket payloads through Layer 4 ``tcp_send_and_receive``, queuing received network payloads directly into socket descriptor buffers for transparent ``read()`` retrieval.
4. **Freestanding ``<arpa/inet.h>`` C Standard Library**: Integration of ``<arpa/inet.h>`` with freestanding implementations of ``htons``, ``ntohs``, ``htonl``, ``ntohl``, ``inet_addr``, ``inet_aton`` and ``inet_ntoa`` in ``userland/libc/src/socket/inet/inet.c``.
5. **Freestanding Ring 3 HTTP Fetch Utility**: Re-engineering of ``userland/netutils/fetch.c`` as a pure userspace client constructing HTTP/1.1 requests, connecting via standard BSD stream sockets, receiving response payloads and writing to disk or standard output.
6. **Canonical FHS Rootfs Purification**: Removal of ad-hoc ``/src`` directory at rootfs root. Sample compiler programs are placed in canonical ``/usr/src/main.c`` and ``userland/etc/init.sh`` copies from ``/usr/src/main.c`` to ``/tmp/main.c``.
7. **VFS Decoupling in Shell Startup**: ``crates/shell/src/terminal/boot.rs`` utilizes ``keira_fs::vfs::read_file`` rather than direct FAT driver calls, preserving filesystem abstraction boundaries.

2. Architecture & Design
========================

.. code-block:: text

   flowchart TD
       subgraph Userspace["Ring 3 Userspace (Freestanding Core Utilities)"]
           FETCH_BIN["/bin/fetch.elf (Ring 3 HTTP Client)"]
           KCC_BIN["/bin/kcc.elf (Native C Compiler)"]
           INIT_SH["/etc/init.sh (Boot Script)"]
           LIBC_INET["libc.a: <arpa/inet.h>, <sys/socket.h>"]
           FETCH_BIN -->|"socket(), connect(), write(), read(), close()"| LIBC_INET
           INIT_SH -->|"cat /usr/src/main.c > /tmp/main.c"| KCC_BIN
       end

       subgraph SyscallBoundary["POSIX Syscall ABI"]
           SYS_ENTRY["INT 0x80 / SYSCALL"]
           ROUTER["Syscall Router"]
           VEC17["Vector 17 (SYS_HTTP_GET) -> -ENOSYS"]
           LIBC_INET --> SYS_ENTRY
           SYS_ENTRY --> ROUTER
           ROUTER -->|"Vector 17"| VEC17
       end

       subgraph KernelEngine["Ring 0 Pure Monolithic Kernel"]
           SOCK_MGR["Socket Manager (crates/net::socket)"]
           TCP_ENGINE["Pure L4 TCP Transport (crates/net::tcp)"]
           NIC_DRIVER["Intel e1000 Driver (crates/net::driver::e1000)"]
           VFS_ROUTER["Virtual File System (crates/fs::vfs)"]
           ROUTER -->|"SYS_SOCKET, SYS_CONNECT"| SOCK_MGR
           ROUTER -->|"SYS_WRITE, SYS_READ"| VFS_ROUTER
           VFS_ROUTER -->|"Socket FD I/O"| SOCK_MGR
           SOCK_MGR -->|"External AF_INET TCP"| TCP_ENGINE
           TCP_ENGINE -->|"Raw Ethernet Frame DMA"| NIC_DRIVER
       end

       subgraph RootFSPurity["Canonical Filesystem Hierarchy Standard (FHS)"]
           ROOT_DIR["Root /"]
           USR_SRC["/usr/src/main.c (Standard FHS Source Tree)"]
           PURGED_SRC["/src (Distro Artifact Permanently Purged)"]
           ROOT_DIR --> USR_SRC
           ROOT_DIR -.->|"PURGED"| PURGED_SRC
       end

3. Engineering Implementation Details
=====================================

3.1 Layer 7 HTTP Purge from Ring 0
----------------------------------

Prior versions embedded an HTTP client inside ``crates/net/src/tcp/stream/http.rs`` with hardcoded ``HTTP_USER_AGENT`` strings and response chunk decoding. This design departed from the classical Unix kernel philosophy. In Milestone 54:

- ``crates/net/src/tcp/stream/http.rs`` was deleted.
- ``HTTP_USER_AGENT`` constants in ``crates/net/src/lib.rs`` were eliminated.
- ``fetch_stream_download``, ``dechunk_in_place`` and ``STREAM_DOWNLOAD_BUFFER`` were purged from ``crates/net/src/tcp/stream.rs``.
- ``handle_http_get`` was removed from ``crates/syscall/src/dispatcher/handlers/net.rs`` and syscall vector 17 was mapped directly to ``errno_to_ret(ENOSYS)``.

3.2 Pure L4 TCP Socket Transport
--------------------------------

To enable userspace network applications to perform HTTP, HTTPS, DNS and custom protocol transactions without in-kernel application helpers, ``crates/net/src/socket/table.rs`` was upgraded:

- When ``send_socket`` receives data for an ``AF_INET`` stream socket where ``!is_loopback_addr(&slot.remote_ip)``, it delegates transmission to Layer 4 ``tcp_send_and_receive``.
- Response data frames received via the NIC DMA engine are copied into ``slot.rx_buf``, updating ``slot.rx_len``.
- Subsequent ``read()`` or ``recv()`` calls from userland pull bytes directly from ``slot.rx_buf``.

3.3 Userspace <arpa/inet.h> and Re-Engineered fetch
---------------------------------------------------

To support standard networking in C userland:

- ``<arpa/inet.h>`` was introduced in ``userland/libc/include/arpa/inet.h`` with definitions for ``in_addr_t``, ``htons``, ``ntohs``, ``htonl``, ``ntohl``, ``inet_addr``, ``inet_aton`` and ``inet_ntoa``.
- ``userland/libc/src/socket/inet/inet.c`` provides freestanding byte swapping and IPv4 address string parsing.
- ``userland/netutils/fetch.c`` was completely re-architected to create an ``AF_INET`` stream socket, connect to the target host and port, format an HTTP/1.1 request in userspace memory, transmit via ``write()``, receive via ``read()`` and handle output files or headers.

3.4 Canonical FHS Rootfs Standardization
----------------------------------------

Rootfs generation in ``Makefile`` and ``userland/etc/init.sh`` was purified:

- The artificial ``/src`` directory was purged from root ``/``.
- Source code is placed in canonical ``/usr/src/main.c``.
- ``userland/etc/init.sh`` populates ``/tmp/main.c`` from ``/usr/src/main.c``.
- ``crates/shell/src/terminal/boot.rs`` reads ``/etc/hostname`` through the VFS router rather than directly calling the FAT driver.

4. Verification and Telemetry Validation
========================================

The implementation underwent full automated validation across dual architectures:

1. **Unit Testing**: All 250+ unit tests across all 12 kernel crates passed with zero errors and zero warnings (``make test-unit``).
2. **Dual-Architecture Compilation**: Both ``x86_64`` (Long Mode) and ``i686`` (Protected Mode) built kernel binaries, userland suites and ISO images with zero errors and zero warnings (``make full``).
3. **FHS Rootfs Verification**: ``ls /`` confirmed the complete absence of ``/src``, showing canonical directories ``bin``, ``dev``, ``proc``, ``sys``, ``etc``, ``include``, ``lib``, ``mnt``, ``usr``, ``tmp`` and ``var``. ``ls /usr/src`` verified ``main.c``.
4. **Live Network Fetch**: ``/bin/fetch.elf -v http://10.0.2.2:8888/keira_api.json`` transmitted an HTTP/1.1 request over standard BSD stream sockets and retrieved the JSON payload from the host server.
5. **Native Compiler Execution**: ``/bin/kcc.elf /tmp/main.c -o /tmp/main.elf`` compiled ``/tmp/main.c`` and executed ``/tmp/main.elf`` cleanly in Ring 3.
6. **I/O Multiplexing Verification**: Standalone ``/bin/test_select.elf`` passed all 5 test cases.
7. **Supervisor Monitor Transition**: ``exit`` transitioned cleanly from userspace shell to Ring 0 supervisor console, running ``selectinfo`` with live telemetry before re-launching ``sh``.
8. **Master ABI Test Suite**: ``/bin/test_abi.elf`` passed all 39 tests and 1,000 automated boundary fuzzing vectors.
