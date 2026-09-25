<!-- SPDX-License-Identifier: GPL-2.0-only -->

# POSIX Signal Handling (`userland/lib/signal/`)

Manages asynchronous process notifications and signal handler registration (`userland/include/signal.h`).

---

## Signal Interfaces

* `signal(int signum, sighandler_t handler)`: Installs basic signal handler.
* `sigaction(int signum, const struct sigaction *act, struct sigaction *oldact)`: Installs detailed handler with signal masks and flags (`SA_RESTART`, `SA_SIGINFO`).
* `kill(pid_t pid, int sig)`: Sends signal to destination process.
* Supported signals include: `SIGHUP`, `SIGINT`, `SIGQUIT`, `SIGILL`, `SIGKILL`, `SIGSEGV`, `SIGPIPE`, `SIGALRM`, `SIGTERM`.
