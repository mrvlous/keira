<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Device & Terminal Control (`<sys/ioctl.h>`)

The `<sys/ioctl.h>` header provides device control requests, specifically for querying terminal window dimensions.

---

## 1. Structure Definition

```c
struct winsize {
    unsigned short ws_row;    /* Rows, in characters (e.g. 25) */
    unsigned short ws_col;    /* Columns, in characters (e.g. 80) */
    unsigned short ws_xpixel; /* Horizontal size, in pixels (640) */
    unsigned short ws_ypixel; /* Vertical size, in pixels (400) */
};
```

---

## 2. Request Codes

| Request | Value | Description |
| :--- | :--- | :--- |
| `TIOCGWINSZ` | `0x5413` | Query window size structure into `struct winsize *` |
| `TIOCSWINSZ` | `0x5414` | Set window size parameters |

---

## 3. Function Reference

### `ioctl`
```c
int ioctl(int fd, unsigned long request, ...);
```
Performs the device-specific control operation `request` on the open file descriptor `fd` via `SYS_IOCTL` (vector 73).
