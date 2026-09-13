<!-- SPDX-License-Identifier: GPL-2.0-only -->

# File Control & Open Flags (`<fcntl.h>`)

The `<fcntl.h>` header defines file access modes, creation flags, and seeking constants used with POSIX stream and file descriptor operations.

---

## 1. File Access & Creation Flags

| Constant | Value | Description |
| :--- | :--- | :--- |
| `O_RDONLY` | `0x0000` | Open for reading only |
| `O_WRONLY` | `0x0001` | Open for writing only |
| `O_RDWR` | `0x0002` | Open for reading and writing |
| `O_CREAT` | `0x0040` | Create file if it does not exist |
| `O_EXCL` | `0x0080` | Exclusive creation; fail if file exists |
| `O_TRUNC` | `0x0200` | Truncate file length to 0 |
| `O_APPEND` | `0x0400` | Set file offset to end of file prior to each write |

---

## 2. File Seeking Offsets

| Constant | Value | Description |
| :--- | :--- | :--- |
| `SEEK_SET` | `0` | Seek relative to beginning of file |
| `SEEK_CUR` | `1` | Seek relative to current file offset |
| `SEEK_END` | `2` | Seek relative to end of file |
