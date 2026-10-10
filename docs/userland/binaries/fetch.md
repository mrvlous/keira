<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `fetch.elf` Freestanding HTTP Client Utility

The `fetch.elf` executable (`/bin/fetch.elf` and `/bin/fetch`) is a freestanding Ring 3 network utility that retrieves resources over HTTP from network endpoints.

---

## 1. Architectural Purpose

In accordance with UNIX purity and the Keira v0.7.0 pure kernel slimming roadmap, high-level network client operations are decoupled from the Ring 0 supervisor console and implemented as freestanding unprivileged Ring 3 binaries:

1. **Ring 3 Privilege Isolation**: Executes entirely in unprivileged Ring 3, requesting network resources exclusively through the kernel network system call boundary.
2. **Resource Retrieval & Disk Streaming**: Downloads HTTP payloads to standard output or streams them directly to disk via `-o <file>`.
3. **HTTP Header Inspection**: Provides `-I` and `--head` flags to inspect server HTTP response status lines and headers without displaying payload bodies.
4. **Pipeline Interoperability**: Integrates cleanly into shell pipelines (e.g. `/bin/fetch http://api.example.com/data | /bin/cat`).

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-o <file>` | *N/A* | Writes received payload to specified local file instead of standard output |
| `-I` | `--head` | Displays HTTP response status line and headers only |
| `-v` | `--verbose` | Displays verbose network connection and payload transfer telemetry |
| `-h` | `--help` | Displays usage reference and exits with status 0 |

---

## 3. Usage Examples

### Fetching Payloads to Standard Output

```bash
# Retrieve a remote resource to stdout:
keira:/# fetch http://example.com/

# Retrieve via canonical /bin/fetch alias:
keira:/# /bin/fetch http://example.com/
```

### Writing Payloads to File

```bash
# Save remote payload to local disk file:
keira:/# fetch -o /tmp/payload.txt http://example.com/

# Inspect saved payload with cat:
keira:/# cat /tmp/payload.txt
```

### Inspecting HTTP Response Headers

```bash
# Display only status line and headers:
keira:/# fetch -I http://example.com/
```

### Verbose Network Diagnostics

```bash
# Verbose transfer diagnostics:
keira:/# fetch -v http://example.com/
```

---

## 4. Technical Specifications

- **Source Path**: `userland/bin/fetch/main.c`
- **Binary Locations**: `/bin/fetch.elf`, `/bin/fetch`
- **Privilege Level**: Ring 3 unprivileged
- **Dependencies**: Freestanding `libc.a` (`<fcntl.h>`, `<stdio.h>`, `<stdlib.h>`, `<string.h>`, `<unistd.h>`, `<sys/syscall.h>`)
- **System Calls**: `sys_http_get` (Vector 83), `sys_open`, `sys_write`, `sys_close` and `sys_exit`
- **Buffer Size**: 4096-byte transfer buffer
