<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Directory Traversal (`<dirent.h>`)

The `<dirent.h>` header provides functions and types for scanning directory trees on mounted FAT16 and USTAR filesystems.

---

## 1. Structure Definitions

```c
struct dirent {
    uint64_t d_ino;
    off_t    d_off;
    uint16_t d_reclen;
    uint8_t  d_type;
    char     d_name[256];
};

typedef struct DIR DIR;
```

---

## 2. Function Reference

### `opendir`
```c
DIR *opendir(const char *name);
```
Opens directory stream for path `name`.

### `readdir`
```c
struct dirent *readdir(DIR *dirp);
```
Reads next directory entry from `dirp`. Returns `NULL` on EOF.

### `closedir`
```c
int closedir(DIR *dirp);
```
Closes directory stream `dirp`.
