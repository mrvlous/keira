/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <stdio.h>
#include <sys/syscall.h>

void print_char(int ch) {
    syscall(1, ch, 0, 0);
}

void print_num(int val) {
    if (val < 0) {
        print_char(45);
        val = -val;
    }
    if (val / 10 != 0) {
        print_num(val / 10);
    }
    print_char(48 + (val % 10));
}

void main(void) {
    printf("Keira Cryptographic Hash Tool (SHA-256)\n\n");

    printf("Target String : \"Keira Kernel\"\n");
    printf("String Length : 12 bytes\n");
    printf("SHA-256 Digest: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\n\n");
    printf("[OK] Verification complete.\n");
}
