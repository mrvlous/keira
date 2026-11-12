// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

#include "common.h"
#include "lexer.h"
#include "parser.h"

#include <syscall.h>

void match(int expected) {
    if (tok == expected) {
        tok = next_token();
    } else {
        print_str("Error on line ");
        print_num(line_num);
        print_str(": Expected ");
        print_str(token_name(expected));
        print_str(", got ");
        print_str(token_name(tok));
        print_str(" ('");
        print_str(token_string);
        print_str("')\n");
        sys_exit(1);
    }
}
