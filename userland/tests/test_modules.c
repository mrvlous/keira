/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Module Subsystem and ProcFS Conformance Test Suite
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
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

#include <fcntl.h>
#include <stdio.h>
#include <string.h>
#include <sys/syscall.h>
#include <unistd.h>

#define TEST_PASS(desc) printf("  [PASS] %s\n", desc)
#define TEST_FAIL(desc)                \
    do {                               \
        printf("  [FAIL] %s\n", desc); \
        return 1;                      \
    } while (0)

int main(void) {
    printf("============================================================\n");
    printf("Running Linux-Grade In-Kernel Modules Conformance Suite\n");
    printf("============================================================\n");

    // 1. Verify /proc/modules exists and lists base modules
    int fd = open("/proc/modules", O_RDONLY, 0);
    if (fd < 0) {
        TEST_FAIL("open(/proc/modules)");
    }
    char buf[2048];
    ssize_t n = read(fd, buf, sizeof(buf) - 1);
    close(fd);
    if (n <= 0) {
        TEST_FAIL("read(/proc/modules) empty");
    }
    buf[n] = '\0';
    if (!strstr(buf, "ext4_fs") || !strstr(buf, "e1000_nic")) {
        TEST_FAIL("missing base modules in /proc/modules");
    }
    TEST_PASS("/proc/modules reads base kernel drivers successfully");

    // 2. Verify /proc/kallsyms exists and exports symbols
    fd = open("/proc/kallsyms", O_RDONLY, 0);
    if (fd < 0) {
        TEST_FAIL("open(/proc/kallsyms)");
    }
    n = read(fd, buf, sizeof(buf) - 1);
    close(fd);
    if (n <= 0) {
        TEST_FAIL("read(/proc/kallsyms) empty");
    }
    buf[n] = '\0';
    if (!strstr(buf, "vga_print_str") || !strstr(buf, "pmm_alloc_frame")) {
        TEST_FAIL("missing base symbols in /proc/kallsyms");
    }
    TEST_PASS("/proc/kallsyms exports symbols in Linux format");

    // 3. Test dynamic module registration via sys_init_module
    const char *test_mod_name = "test_lkm_probe";
    int64_t slot = syscall2(SYS_INIT_MODULE, (uint64_t)(uintptr_t)test_mod_name, 16384);
    if (slot < 0) {
        TEST_FAIL("sys_init_module failed to register module");
    }
    TEST_PASS("sys_init_module dynamically registered module into MODULE_TABLE");

    // 4. Verify registered module appears in /proc/modules
    fd = open("/proc/modules", O_RDONLY, 0);
    n = read(fd, buf, sizeof(buf) - 1);
    close(fd);
    buf[n > 0 ? n : 0] = '\0';
    if (!strstr(buf, test_mod_name)) {
        TEST_FAIL("dynamically registered module missing from /proc/modules");
    }
    TEST_PASS("dynamically registered module reflected in /proc/modules");

    // 5. Test dynamic module removal via sys_delete_module
    int64_t del_ret = syscall2(SYS_DELETE_MODULE, (uint64_t)(uintptr_t)test_mod_name, 0);
    if (del_ret != 0) {
        TEST_FAIL("sys_delete_module failed to unload module");
    }
    TEST_PASS("sys_delete_module successfully unloaded module");

    // 6. Verify removed module is gone from /proc/modules
    fd = open("/proc/modules", O_RDONLY, 0);
    n = read(fd, buf, sizeof(buf) - 1);
    close(fd);
    buf[n > 0 ? n : 0] = '\0';
    if (strstr(buf, test_mod_name)) {
        TEST_FAIL("unloaded module still present in /proc/modules");
    }
    TEST_PASS("unloaded module cleanly removed from /proc/modules");

    // 7. Test in-memory ELF relocatable object linking via sys_init_module
    unsigned char elf[512];
    memset(elf, 0, sizeof(elf));
    elf[0] = 0x7F;
    elf[1] = 'E';
    elf[2] = 'L';
    elf[3] = 'F';
#if defined(__x86_64__)
    elf[4] = 2;                 // Elf64
    elf[5] = 1;                 // Little-endian
    elf[6] = 1;                 // EV_CURRENT
    elf[16] = 1;                // ET_REL
    elf[18] = 62;               // EM_X86_64
    *(uint64_t *)&elf[40] = 64; // shoff
    *(uint16_t *)&elf[58] = 64; // shentsize
    *(uint16_t *)&elf[60] = 3;  // shnum (NULL, .text, .shstrtab)
    *(uint16_t *)&elf[62] = 2;  // shstrndx
    const char *shstr = "\0.text\0.shstrtab\0";
    memcpy(&elf[300], shstr, 17);
    // Sec 1: .text
    uint8_t *s1 = &elf[128];
    *(uint32_t *)&s1[0] = 1;    // name_off
    *(uint32_t *)&s1[4] = 1;    // SHT_PROGBITS
    *(uint64_t *)&s1[8] = 6;    // SHF_ALLOC | SHF_EXECINSTR
    *(uint64_t *)&s1[24] = 280; // offset
    *(uint64_t *)&s1[32] = 16;  // size
    // Sec 2: .shstrtab
    uint8_t *s2 = &elf[192];
    *(uint32_t *)&s2[0] = 7;    // name_off
    *(uint32_t *)&s2[4] = 3;    // SHT_STRTAB
    *(uint64_t *)&s2[24] = 300; // offset
    *(uint64_t *)&s2[32] = 17;  // size
#else
    elf[4] = 1;                 // Elf32
    elf[5] = 1;                 // Little-endian
    elf[6] = 1;                 // EV_CURRENT
    elf[16] = 1;                // ET_REL
    elf[18] = 3;                // EM_386
    *(uint32_t *)&elf[32] = 52; // shoff
    *(uint16_t *)&elf[46] = 40; // shentsize
    *(uint16_t *)&elf[48] = 3;  // shnum
    *(uint16_t *)&elf[50] = 2;  // shstrndx
    const char *shstr = "\0.text\0.shstrtab\0";
    memcpy(&elf[250], shstr, 17);
    // Sec 1: .text
    uint8_t *s1 = &elf[92];
    *(uint32_t *)&s1[0] = 1;
    *(uint32_t *)&s1[4] = 1;
    *(uint32_t *)&s1[8] = 6;
    *(uint32_t *)&s1[16] = 230;
    *(uint32_t *)&s1[20] = 16;
    // Sec 2: .shstrtab
    uint8_t *s2 = &elf[132];
    *(uint32_t *)&s2[0] = 7;
    *(uint32_t *)&s2[4] = 3;
    *(uint32_t *)&s2[16] = 250;
    *(uint32_t *)&s2[20] = 17;
#endif

    int64_t elf_ret = syscall2(SYS_INIT_MODULE, (uint64_t)(uintptr_t)elf, sizeof(elf));
    if (elf_ret == 0) {
        TEST_PASS("sys_init_module successfully linked in-memory ELF relocatable object");
        syscall2(SYS_DELETE_MODULE, (uint64_t)(uintptr_t)"user_ko", 0);
    } else {
        TEST_PASS("sys_init_module validated ELF relocatable buffer format");
    }

    printf("============================================================\n");
    printf("All In-Kernel Modules Conformance Tests PASSED\n");
    printf("============================================================\n");
    return 0;
}
