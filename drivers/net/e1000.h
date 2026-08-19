/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef KEIRA_DRIVERS_NET_E1000_H
#define KEIRA_DRIVERS_NET_E1000_H

#include <stdint.h>

/**
 * Keira Drivers: Intel e1000 Gigabit Ethernet Network Card Driver
 */

#define E1000_VENDOR_ID 0x8086
#define E1000_DEVICE_ID 0x100E

void e1000_init_c(void);
int e1000_is_present(void);
void e1000_get_mac_c(uint8_t *mac_buf);
int e1000_send_packet_c(const uint8_t *data, uint32_t len);

#endif /* KEIRA_DRIVERS_NET_E1000_H */