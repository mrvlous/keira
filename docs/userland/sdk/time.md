<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Real-Time Clock & Time (`<time.h>`)

The `<time.h>` header provides calendar time conversions, real-time clock queries, and string formatting functions.

---

## 1. Structure Definitions

```c
struct tm {
    int tm_sec;   /* Seconds (0-60) */
    int tm_min;   /* Minutes (0-59) */
    int tm_hour;  /* Hours (0-23) */
    int tm_mday;  /* Day of the month (1-31) */
    int tm_mon;   /* Month (0-11) */
    int tm_year;  /* Years since 1900 */
    int tm_wday;  /* Day of the week (0-6) */
    int tm_yday;  /* Day in the year (0-365) */
    int tm_isdst; /* Daylight saving time flag */
};
```

---

## 2. Function Reference

### `time`
```c
time_t time(time_t *tloc);
```
Returns system uptime / epoch seconds.

### `gmtime`
```c
struct tm *gmtime(const time_t *timep);
```
Converts epoch time `timep` into broken-down UTC calendar structure.

### `asctime`
```c
char *asctime(const struct tm *tm);
```
Converts broken-down time structure into standard ASCII string `YYYY-MM-DD HH:MM:SS UTC`.

### `clock_gettime`
```c
int clock_gettime(clockid_t clk_id, struct timespec *tp);
```
Retrieves the high-resolution time of the specified clock (`CLOCK_REALTIME` or `CLOCK_MONOTONIC`) into `struct timespec` via `SYS_CLOCK_GETTIME` (vector 66).

### `nanosleep`
```c
int nanosleep(const struct timespec *req, struct timespec *rem);
```
Suspends calling thread execution until the interval requested in `req` has elapsed via `SYS_NANOSLEEP` (vector 67).

---

## 3. High-Resolution Structure Definition

```c
struct timespec {
    time_t tv_sec;  /* Seconds */
    long   tv_nsec; /* Nanoseconds (0 - 999,999,999) */
};
```
