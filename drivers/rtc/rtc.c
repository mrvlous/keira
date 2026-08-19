/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

/**
 * Real-Time Clock (CMOS RTC) Driver Implementation
 */

#include "rtc.h"

#include "regs.h"

#include <asm/io.h>

/**
 * cmos_read - Read a value from CMOS register.
 * @reg: CMOS register index to select.
 *
 * Return: Byte value read from the register.
 */
static uint8_t cmos_read(uint8_t reg) {
    outb(CMOS_ADDRESS_PORT, reg);
    io_wait();
    return inb(CMOS_DATA_PORT);
}

/**
 * rtc_update_in_progress - Check if CMOS RTC is currently updating time values.
 *
 * Return: Non-zero if update is in progress, 0 otherwise.
 */
static int rtc_update_in_progress(void) {
    return cmos_read(RTC_REG_STATUS_A) & 0x80;
}

/**
 * bcd_to_bin - Convert binary-coded decimal (BCD) to standard binary integer.
 * @bcd: BCD value.
 *
 * Return: Decoded binary representation.
 */
static uint8_t bcd_to_bin(uint8_t bcd) {
    return ((bcd >> 4) * 10) + (bcd & 0x0F);
}

/**
 * rtc_init - Initialize CMOS RTC hardware subsystem.
 */
void rtc_init(void) {
}

/**
 * rtc_get_time - Read current date and time registers from CMOS RTC.
 * @time: Pointer to rtc_time_t payload structure to populate.
 */
void rtc_get_time(rtc_time_t *time) {
    while (rtc_update_in_progress()) {
    }

    uint8_t second = cmos_read(RTC_REG_SECONDS);
    uint8_t minute = cmos_read(RTC_REG_MINUTES);
    uint8_t hour = cmos_read(RTC_REG_HOURS);
    uint8_t day = cmos_read(RTC_REG_DAY);
    uint8_t month = cmos_read(RTC_REG_MONTH);
    uint8_t year = cmos_read(RTC_REG_YEAR);

    while (rtc_update_in_progress()) {
    }
    uint8_t second2 = cmos_read(RTC_REG_SECONDS);
    uint8_t minute2 = cmos_read(RTC_REG_MINUTES);
    uint8_t hour2 = cmos_read(RTC_REG_HOURS);
    uint8_t day2 = cmos_read(RTC_REG_DAY);
    uint8_t month2 = cmos_read(RTC_REG_MONTH);
    uint8_t year2 = cmos_read(RTC_REG_YEAR);

    if (second != second2 || minute != minute2 || hour != hour2 || day != day2 || month != month2 ||
        year != year2) {
        while (rtc_update_in_progress()) {
        }
        second = cmos_read(RTC_REG_SECONDS);
        minute = cmos_read(RTC_REG_MINUTES);
        hour = cmos_read(RTC_REG_HOURS);
        day = cmos_read(RTC_REG_DAY);
        month = cmos_read(RTC_REG_MONTH);
        year = cmos_read(RTC_REG_YEAR);
    }

    uint8_t status_b = cmos_read(RTC_REG_STATUS_B);

    if (!(status_b & 0x04)) {
        second = bcd_to_bin(second);
        minute = bcd_to_bin(minute);
        hour = bcd_to_bin(hour & 0x7F) | (hour & 0x80);
        day = bcd_to_bin(day);
        month = bcd_to_bin(month);
        year = bcd_to_bin(year);
    }

    if (!(status_b & 0x02) && (hour & 0x80)) {
        hour = ((hour & 0x7F) + 12) % 24;
    }

    time->second = second;
    time->minute = minute;
    time->hour = hour;
    time->day = day;
    time->month = month;
    time->year = 2000 + year;
}