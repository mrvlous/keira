<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `touch.elf` Freestanding Core Utility

The `touch.elf` executable (`/bin/touch.elf` and `/bin/touch`) is a freestanding Ring 3 core utility that creates empty files and updates file modification timestamps.

---

## 1. Architectural Purpose

File node creation is handled with minimal kernel overhead via `open(path, O_WRONLY | O_CREAT)`:

1. **Empty File Creation**: Instantiates empty files on the virtual filesystem if the target path does not exist.
2. **Timestamp Refresh**: Updates access and modification timestamps on pre-existing files without modifying their contents.
3. **No-Create Guard**: Supports `-c` (`--no-create`) to suppress file creation if the target path does not exist.

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-c` | `--no-create` | Do not create files that do not exist |
| `-h` | `--help` | Displays usage summary and exits with status 0 |

---

## 3. Usage Examples

```bash
# Create an empty file:
keira:/# touch /tmp/sample.txt

# Verify file creation:
keira:/# ls -a /tmp
.  ..  sample.txt

# Touch an existing file with timestamp refresh:
keira:/# touch /tmp/sample.txt

# Touch with no-create flag on nonexistent file:
keira:/# touch -c /tmp/nonexistent.txt
```

---

## 4. Technical Specifications

- **Source Path**: `userland/bin/touch/main.c`
- **Binary Locations**: `/bin/touch.elf`, `/bin/touch`
- **System Call**: Syscall 6 (`SYS_OPEN`) with `O_CREAT`
- **Privilege Level**: Ring 3 unprivileged
