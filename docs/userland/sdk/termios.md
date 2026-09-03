<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Terminal Control & Line Discipline (`<termios.h>`)

The `<termios.h>` header provides functions and structures for configuring terminal line discipline, switching between canonical and raw input modes, and adjusting terminal attributes.

---

## 1. Structure Definition

```c
struct termios {
    tcflag_t c_iflag;    /* Input modes */
    tcflag_t c_oflag;    /* Output modes */
    tcflag_t c_cflag;    /* Control modes */
    tcflag_t c_lflag;    /* Local modes */
    cc_t c_line;         /* Line discipline */
    cc_t c_cc[NCCS];     /* Special control characters */
    speed_t c_ispeed;    /* Input baud rate */
    speed_t c_ospeed;    /* Output baud rate */
};
```

---

## 2. Terminal Mode Constants

| Constant | Description |
| :--- | :--- |
| `ICANON` | Canonical input mode (line-buffered with editing) |
| `ECHO` | Echo input characters |
| `ISIG` | Enable signals (`Ctrl+C` -> `SIGINT`, `Ctrl+\` -> `SIGQUIT`) |
| `TCSANOW` | Apply terminal attribute changes immediately |
| `TCSADRAIN` | Apply changes after all queued output has been transmitted |
| `TCSAFLUSH` | Apply changes after draining output and discarding unread input |

---

## 3. Function Reference

### `tcgetattr`
```c
int tcgetattr(int fd, struct termios *termios_p);
```
Queries current terminal parameters associated with file descriptor `fd` and stores them in `termios_p`.

### `tcsetattr`
```c
int tcsetattr(int fd, int optional_actions, const struct termios *termios_p);
```
Configures terminal attributes according to `termios_p` using the specified action (`TCSANOW`, `TCSADRAIN`, `TCSAFLUSH`).
