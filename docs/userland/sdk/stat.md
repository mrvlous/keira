<!-- SPDX-License-Identifier: GPL-2.0-only -->

# File Metadata & Inode Attributes (`<sys/stat.h>`)

The `<sys/stat.h>` header defines file metadata structures, mode flags, and file inspection functions.

---

## 1. Structure Definitions

```c
struct stat {
    uint64_t st_dev;     /* ID of device containing file */
    uint64_t st_ino;     /* Inode number */
    mode_t   st_mode;    /* File type and permissions */
    uint32_t st_nlink;   /* Number of hard links */
    uid_t    st_uid;     /* User ID of owner */
    gid_t    st_gid;     /* Group ID of owner */
    uint64_t st_rdev;    /* Device ID (if special file) */
    off_t    st_size;    /* Total size in bytes */
    uint64_t st_blksize; /* Block size for filesystem I/O */
    uint64_t st_blocks;  /* Number of 512B blocks allocated */
    time_t   st_atime;   /* Time of last access */
    time_t   st_mtime;   /* Time of last modification */
    time_t   st_ctime;   /* Time of last status change */
};
```

---

## 2. Mode Macros & Flags

| Flag | Value | Description |
| :--- | :--- | :--- |
| `S_IFREG` | `0100000` | Regular file |
| `S_IFDIR` | `0040000` | Directory |
| `S_IFCHR` | `0020000` | Character special device |
| `S_IFBLK` | `0060000` | Block special device |
| `S_IRUSR` | `00400` | Owner read permission |
| `S_IWUSR` | `00200` | Owner write permission |
| `S_IXUSR` | `00100` | Owner execute permission |

---

## 3. Functions

### `stat`
```c
int stat(const char *pathname, struct stat *statbuf);
```
Queries metadata for the filesystem entry at `pathname`.

### `fstat`
```c
int fstat(int fd, struct stat *statbuf);
```
Queries metadata for the open file descriptor `fd`.
