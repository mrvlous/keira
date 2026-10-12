<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Terminal Line Discipline

The Keira terminal subsystem implements a POSIX-compatible line discipline layer that arbitrates raw keyboard input from character devices and delivers parsed, edited data streams to userland and supervisor shells.

## Operating Modes

* **Canonical Mode (`ICANON`)**: Line-buffered input processing with in-kernel line editing. Input is buffered until a line delimiter (`\n` or `\r`) is encountered. Edit control characters such as backspace (`0x08`, `\b`) and delete (`0x7F`) remove the preceding character from the uncommitted line buffer.
* **Raw Mode**: Character-by-character immediate delivery without echoing, signal generation or line buffering.

## Line Editing and Prompt Boundary Protection

In canonical mode, the line discipline tracks the active line length (`LINE_LEN`) and provides verification of erase operations:

1. **Character Consumption**: Alphanumeric and symbol characters append to the internal input line buffer and increment `LINE_LEN`.
2. **Erase Verification**: When an erase character (`0x08`, `0x7F` or `VERASE`) is received:
   * If `LINE_LEN > 0`, the line discipline pops the trailing character, decrements `LINE_LEN`, sets the erase flag (`LAST_BACKSPACE_POPPED = true`) and returns `true`.
   * If `LINE_LEN == 0`, the line discipline leaves the buffer unchanged, clears the erase flag (`LAST_BACKSPACE_POPPED = false`) and returns `false`.
3. **Boundary Isolation**: Because the interactive shell prompt (`keira:/$ ` or `ring0@keira:/# `) is rendered via standard output rather than buffered within the input line discipline, maintaining strict `LINE_LEN == 0` erase rejection guarantees that backspaces cannot underflow into or erase the interactive prompt.
4. **Signal and Newline Reset**: Encountering a newline or interrupt signal (`VINTR`, `Ctrl+C`) flushes uncommitted line state and resets `LINE_LEN` to zero.

