<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Process State & Child Reaping (`<sys/wait.h>`)

The `<sys/wait.h>` header defines macros and function prototypes for parent processes waiting for child process state transitions and retrieving exit status codes.

---

## 1. Options & Flags

| Flag | Value | Description |
| :--- | :--- | :--- |
| `WNOHANG` | `1` | Return immediately without blocking if no child has transitioned |
| `WUNTRACED` | `2` | Also return status for stopped child processes |

---

## 2. Status Evaluation Macros

| Macro | Expression | Description |
| :--- | :--- | :--- |
| `WIFEXITED(status)` | `(((status) & 0x7f) == 0)` | Evaluates to non-zero if the child process terminated normally |
| `WEXITSTATUS(status)` | `(((status) & 0xff00) >> 8)` | Evaluates to the low-order 8 bits of the child process exit status code |
| `WIFSIGNALED(status)` | `(((signed char)(((status) & 0x7f) + 1) >> 1) > 0)` | Evaluates to non-zero if the child process was terminated by an unhandled signal |
| `WTERMSIG(status)` | `((status) & 0x7f)` | Evaluates to the signal number that caused process termination |

---

## 3. Function Reference

### `wait`
```c
pid_t wait(int *wstatus);
```
Suspends execution of the calling thread until one of its children terminates. Equivalent to `waitpid(-1, wstatus, 0)`.

### `waitpid`
```c
pid_t waitpid(pid_t pid, int *wstatus, int options);
```
Waits for state changes in a child process specified by `pid`:
* `pid > 0`: Wait for the specific child process whose process ID equals `pid`.
* `pid == -1`: Wait for any child process.
* `options`: Bitwise OR of flags (`WNOHANG`, `WUNTRACED`).
* If `wstatus` is non-null, stores the encoded exit status code. Returns child PID upon success, `0` if `WNOHANG` was requested and child has not changed state, or `-1` on error.
