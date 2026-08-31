<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Signal Handling Interface (`<signal.h>`)

The `<signal.h>` header provides signal numbers, signal set types, and signal handlers for asynchronous execution and fault handling.

---

## 1. Signal Numbers

| Signal | Value | Description | Default Action |
| :--- | :--- | :--- | :--- |
| `SIGHUP` | `1` | Hangup detected on controlling terminal | Terminate |
| `SIGINT` | `2` | Interrupt from keyboard (`^C`) | Terminate |
| `SIGQUIT` | `3` | Quit from keyboard | Terminate (Dump) |
| `SIGILL` | `4` | Illegal instruction | Terminate (Dump) |
| `SIGABRT` | `6` | Abort signal from `abort()` | Terminate (Dump) |
| `SIGFPE` | `8` | Floating point exception | Terminate (Dump) |
| `SIGKILL` | `9` | Kill signal (cannot be caught) | Terminate |
| `SIGSEGV` | `11` | Invalid memory reference | Terminate (Dump) |
| `SIGPIPE` | `13` | Broken pipe | Terminate |
| `SIGALRM` | `14` | Timer signal from `alarm()` | Terminate |
| `SIGTERM` | `15` | Termination signal | Terminate |

---

## 2. Function Reference

### `signal`
```c
sighandler_t signal(int signum, sighandler_t handler);
```
Registers `handler` as the dispatch routine for signal `signum`. Returns the previous handler.

### `raise`
```c
int raise(int sig);
```
Sends signal `sig` to the current calling process.
