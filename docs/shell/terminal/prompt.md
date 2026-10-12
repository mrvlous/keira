<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Shell Prompt & History Navigation

Keira implements a dual shell prompt architecture reflecting privilege separation:

1. **Ring 0 Supervisor Console**: Renders `ring0@<hostname>:<path># ` for emergency diagnostic and hardware recovery commands.
2. **Ring 3 Userspace Shell (`sh`)**: Renders `<hostname>:<path>$ ` (e.g. `keira:/$ `, `keira:/bin$ `) for unprivileged standard UNIX operations.

## Input Line Boundary Protection

Interactive shells in Keira enforce strict prompt boundary protection during line editing:

* **Foreground Userspace Jobs**: When a userland process (such as `sh` or an interactive CLI tool) reads from `/dev/tty`, backspace keypresses (`0x08` and `0x7F`) are mediated by the terminal line discipline. The console backspace routine is only invoked if a typed character was successfully removed from the active input buffer. If no input characters have been typed on the line, backspace events are ignored, preventing visual erasure or cursor retreat into the prompt string.
* **Supervisor Console**: In Ring 0 supervisor mode, keypress input is accumulated into an internal command buffer. Backspace operations check that `BUFFER_LEN > 0` before decrementing buffer state and adjusting the screen cursor, ensuring the supervisor prompt cannot be erased.
* **VGA Cursor Underflow Protection**: The low-level VGA text console driver bounds backspace cursor adjustments to `CURSOR_X > 0` and `TEXT_CURSOR_X > 0`. Backspace operations never wrap upward across row boundaries on column zero, guarding preceding terminal output from overwrite or visual corruption.

