/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef KEIRA_DRIVERS_RTC_REGS_H
#define KEIRA_DRIVERS_RTC_REGS_H

/**
 * CMOS RTC Hardware Ports and Registers
 */

#define CMOS_ADDRESS_PORT 0x70
#define CMOS_DATA_PORT 0x71

#define RTC_REG_SECONDS 0x00
#define RTC_REG_MINUTES 0x02
#define RTC_REG_HOURS 0x04
#define RTC_REG_DAY 0x07
#define RTC_REG_MONTH 0x08
#define RTC_REG_YEAR 0x09
#define RTC_REG_STATUS_A 0x0A
#define RTC_REG_STATUS_B 0x0B

#endif /* KEIRA_DRIVERS_RTC_REGS_H */
