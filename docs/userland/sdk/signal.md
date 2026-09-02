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

### `kill`
```c
int kill(pid_t pid, int sig);
```
Dispatches asynchronous POSIX signal `sig` to the process identified by `pid` via `SYS_KILL` (vector 22).

---

## 3. Kernel Trampoline & Signal Registration

Userland signals can be registered directly with the Ring 0 kernel via `sys_sigaction` (vector 64). When a signal is delivered to a task, the kernel interrupts user execution, pushes a trampoline stack frame, and switches execution to the registered signal handler. Upon return, `sys_sigreturn` (vector 65) restores the saved processor register context.
