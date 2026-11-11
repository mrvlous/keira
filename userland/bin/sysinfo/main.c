// SPDX-License-Identifier: GPL-2.0-only

#include "sysinfo.h"

#include <stdio.h>

int main(int argc, char **argv) {
    (void)argc;
    (void)argv;

    puts("Keira Ring 3 Diagnostic Utility (sysinfo)");

    if (display_system_info() != 0) {
        puts("[FAIL] Unable to query system metrics.");
        return 1;
    }

    puts("[OK] System info query completed successfully.");
    return 0;
}
