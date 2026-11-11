// SPDX-License-Identifier: GPL-2.0-only

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
