<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Time & Clock Functions (`userland/lib/time/`)

Provides POSIX time representations and timer utilities (`userland/include/time.h`).

---

## Key Routines

* `time(time_t *tloc)`: Retrieves current Unix epoch timestamp in seconds.
* `clock_gettime(clockid_t clk_id, struct timespec *tp)`: High-precision timestamp via `CLOCK_REALTIME` or `CLOCK_MONOTONIC`.
* `nanosleep(const struct timespec *req, struct timespec *rem)`: High-resolution task suspension.
