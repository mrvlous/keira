<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Terminal Interface (`userland/lib/termios/`)

Controls terminal line disciplines and character attributes (`userland/include/termios.h`).

---

## Terminal Modes

* **Canonical Mode**: Input buffered line-by-line; character editing (`backspace`) enabled.
* **Raw Mode**: Characters delivered immediately to application without echo or line buffering (used by `kvi` editor and shell input loop).

---

## APIs

```c
int tcgetattr(int fd, struct termios *termios_p);
int tcsetattr(int fd, int optional_actions, const struct termios *termios_p);
```
