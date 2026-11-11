// SPDX-License-Identifier: GPL-2.0-only

#include "parser.h"

/* Assignment Expressions (=, +=, -=, *=, /=, %=, &=, |=, ^=, <<=, >>=) */
void assignment_expr(void) {
    log_or_expr();
}

void expression(void) {
    assignment_expr();
}
