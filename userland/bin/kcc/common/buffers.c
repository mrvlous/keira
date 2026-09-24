/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include "common.h"

char src_buf[MAX_SOURCE_SIZE];
char prep_buf[MAX_SOURCE_SIZE];
unsigned char code_buf[MAX_CODE_SIZE];
unsigned char data_buf[MAX_DATA_SIZE];
int code_idx = 0;
int data_idx = 0;
