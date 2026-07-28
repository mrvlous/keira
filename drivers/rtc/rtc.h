/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef KEIRA_DRIVERS_RTC_RTC_H
#define KEIRA_DRIVERS_RTC_RTC_H

#include <stdint.h>

/**
 * Real-Time Clock (CMOS RTC) Driver Interface
 */

/**
 * struct rtc_time_t - CMOS Real-Time Clock date and time payload.
 * @second: Current second (0-59).
 * @minute: Current minute (0-59).
 * @hour: Current hour (0-23 in 24-hour format).
 * @day: Current day of the month (1-31).
 * @month: Current month index (1-12).
 * @year: Current absolute year (e.g. 2026).
 */
typedef struct {
    uint8_t second;
    uint8_t minute;
    uint8_t hour;
    uint8_t day;
    uint8_t month;
    uint16_t year;
} rtc_time_t;

/**
 * rtc_init - Initialize CMOS RTC hardware subsystem.
 */
void rtc_init(void);

/**
 * rtc_get_time - Read current date and time registers from CMOS RTC.
 * @time: Pointer to rtc_time_t payload structure to populate.
 */
void rtc_get_time(rtc_time_t *time);

#endif /* KEIRA_DRIVERS_RTC_RTC_H */
