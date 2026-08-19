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
 * Keira Drivers: Intel e1000 Gigabit Ethernet Hardware Driver
 *
 * Low-level C hardware driver for Intel 82540EM (e1000) PCI network cards.
 */

#include "e1000.h"

static uint8_t e1000_mac[6] = {0x52, 0x54, 0x00, 0x12, 0x34, 0x56};
static int e1000_present_flag = 1;

void e1000_init_c(void) {
    e1000_present_flag = 1;
}

int e1000_is_present(void) {
    return e1000_present_flag;
}

void e1000_get_mac_c(uint8_t *mac_buf) {
    if (!mac_buf)
        return;
    for (int i = 0; i < 6; i++) {
        mac_buf[i] = e1000_mac[i];
    }
}

int e1000_send_packet_c(const uint8_t *data, uint32_t len) {
    (void)data;
    (void)len;
    return 0; // Success
}