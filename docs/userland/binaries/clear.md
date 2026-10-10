<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `clear.elf` Freestanding Core Utility

The `clear.elf` executable (`/bin/clear.elf` and `/bin/clear`) is a freestanding Ring 3 core utility that clears the terminal display.

---

## 1. Architectural Purpose

Terminal display manipulation adheres to standard ANSI escape sequence protocols:

1. **ANSI Control Sequences**: Emits `\033[2J` (erase display) followed by `\033[H` (cursor home) to standard output.
2. **Supervisor Independence**: Decouples terminal wiping from supervisor console internal VGA routines, enabling shell scripts and userspace programs to control terminal presentation uniformly.
3. **Purity**: Zero internal kernel hooks required; standard `write(STDOUT_FILENO, ...)` suffices.

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-h` | `--help` | Displays usage summary and exits with status 0 |

---

## 3. Usage Examples

```bash
# Clear screen:
keira:/# clear

# Clear screen via canonical alias:
keira:/# /bin/clear
```

---

## 4. Technical Specifications

- **Source Path**: `userland/bin/clear/main.c`
- **Binary Locations**: `/bin/clear.elf`, `/bin/clear`
- **Privilege Level**: Ring 3 unprivileged
- **Escape Sequence**: `\033[2J\033[H`
