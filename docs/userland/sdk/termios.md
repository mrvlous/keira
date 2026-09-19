<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Terminal Control Interface (`<termios.h>`)

The `<termios.h>` header provides terminal I/O interfaces, line discipline modes, and attribute querying/configuration functions in Ring 3 userland.

---

## 1. Terminal Structure & Attribute Modes

### Structure Definition

```c
struct termios {
    tcflag_t c_iflag;      /* Input mode flags */
    tcflag_t c_oflag;      /* Output mode flags */
    tcflag_t c_cflag;      /* Control mode flags */
    tcflag_t c_lflag;      /* Local mode flags */
    cc_t     c_line;       /* Line discipline */
    cc_t     c_cc[NCCS];   /* Control characters (VINTR, VQUIT, VERASE, VKILL, VEOF) */
    speed_t  c_ispeed;     /* Input baud rate */
    speed_t  c_ospeed;     /* Output baud rate */
};
```

### Local Modes (`c_lflag`)
| Flag | Value | Description |
| :--- | :--- | :--- |
| `ICANON` | `0000002` | Canonical mode (line-buffered with backspace editing) |
| `ECHO` | `0000010` | Automatic echo of typed characters to terminal screen |
| `ECHOE` | `0000020` | Visual erase character on backspace |
| `ECHOK` | `0000040` | Kill character erases current line |
| `ISIG` | `0000001` | Enable asynchronous signal keys (`Ctrl+C` -> `SIGINT`) |

### Terminal Control Actions (`optional_actions`)
| Action | Value | Description |
| :--- | :--- | :--- |
| `TCSANOW` | `0` | Apply terminal attributes immediately |
| `TCSADRAIN` | `1` | Apply terminal attributes after pending output drains |
| `TCSAFLUSH` | `2` | Apply attributes after discarding unread input |

---

## 2. Function Reference

### `tcgetattr`
```c
int tcgetattr(int fd, struct termios *termios_p);
```
Queries current terminal parameters from descriptor `fd` via `SYS_IOCTL` (`TCGETS`). Returns `0` on success, or `-1` on error with `errno` set.

### `tcsetattr`
```c
int tcsetattr(int fd, int optional_actions, const struct termios *termios_p);
```
Configures terminal parameters on descriptor `fd` via `SYS_IOCTL` (`TCSETS`). Switches terminal between canonical line-buffered mode and raw character mode. Returns `0` on success, or `-1` on error.
