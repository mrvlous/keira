<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# POSIX File Descriptors and Userland Stream I/O

## Overview

Keira Kernel implements standard POSIX-compliant file descriptor semantics and file control flags for Ring 3 applications and kernel subsystems.

```
+-------------------------------------------------------------+
|               POSIX File Descriptor Table (0..1024)         |
+-------------------------------------------------------------+
| FD 0 : STDIN  (Keyboard / TTY stream)                       |
| FD 1 : STDOUT (VGA Console / Serial stream)                 |
| FD 2 : STDERR (Diagnostic syslog / TTY stream)              |
| FD 3+: Dynamic Files, Sockets, Pipes, and Epoll instances   |
+-------------------------------------------------------------+
```

## File Control Access Modes (`fcntl.h`)

The kernel supports standard POSIX bitmasks for file creation and stream manipulation:

* `O_RDONLY (0)`: Open for reading only.
* `O_WRONLY (1)`: Open for writing only.
* `O_RDWR (2)`: Open for both reading and writing.
* `O_CREAT (64)`: Create file if it does not exist on VFS.
* `O_TRUNC (512)`: Truncate existing file size to 0 bytes upon open.
* `O_APPEND (1024)`: Append writes to the end of the file.

## Core I/O System Calls

* `sys_open(path, flags)` (Syscall 6): Allocate file descriptor.
* `sys_read(fd, buf, count)` (Syscall 7): Read stream data.
* `sys_write(fd, buf, count)` (Syscall 8): Write stream data.
* `sys_close(fd)` (Syscall 9): Release file descriptor table entry.
* `sys_seek(fd, offset)` (Syscall 10): Adjust file offset pointer.
