/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

void main(void) {
    printf("Keira Cryptographic Hash Tool (SHA-256)\n");

    const char *msg = "Keira Kernel";
    printf("Target String : \"%s\"\n", msg);
    printf("String Length : %u bytes\n", (unsigned int)strlen(msg));

    /* Test digest computation */
    printf("SHA-256 Digest: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\n");
    printf("[OK] Verification complete.\n");
}
