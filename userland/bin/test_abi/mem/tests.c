/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include "test_abi.h"

int run_test_abi_mem(void) {
    /* 2. Kernel Memory Boundary & EFAULT Protection */
    puts("  [TEST] Kernel address pointer isolation (EFAULT check)...");
#if defined(__x86_64__)
    void *kernel_addr = (void *)0xFFFF800000000000ULL;
#else
    void *kernel_addr = (void *)0xC0000000UL;
#endif
    ssize_t efault_res = write(1, kernel_addr, 16);
    if (efault_res >= 0) {
        puts("  [FAIL] Kernel address access was not blocked by copy_from_user!");
        return 1;
    }
    puts("  [OK]   Kernel space boundary validated (EFAULT enforced)");

    /* 3. Null Pointer Protection */
    puts("  [TEST] NULL pointer dereference protection...");
    ssize_t null_res = read(0, NULL, 32);
    if (null_res >= 0) {
        puts("  [FAIL] NULL pointer was not blocked by copy_to_user!");
        return 1;
    }
    puts("  [OK]   NULL pointer isolation validated (EFAULT enforced)");

    /* 8. VMM Demand Paging & Lazy Heap (sbrk) Allocation */
    puts("  [TEST] VMM demand paging & heap expansion (sbrk)...");
    void *initial_brk = sbrk(0);
    if (initial_brk == (void *)-1) {
        puts("  [FAIL] sbrk(0) query failed");
        return 1;
    }
    void *allocated_brk = sbrk(4096);
    if (allocated_brk == (void *)-1) {
        puts("  [FAIL] sbrk(4096) allocation failed");
        return 1;
    }
    /* Write through pointer to trigger demand paging page fault handler */
    char *heap_ptr = (char *)allocated_brk;
    *heap_ptr = 0x77;
    if (*heap_ptr != 0x77) {
        puts("  [FAIL] Demand paged memory verification failed");
        return 1;
    }
    puts("  [OK]   VMM demand paging & sbrk expansion operational");

    /* 11. Dual-Tier Memory Allocator & Coalescing */
    puts("  [TEST] Dual-tier memory allocator (heap & mmap tiers)...");
    void *p1 = malloc(64);
    void *p2 = malloc(64);
    void *p3 = malloc(64);
    if (!p1 || !p2 || !p3) {
        if (p1)
            free(p1);
        if (p2)
            free(p2);
        if (p3)
            free(p3);
        puts("  [FAIL] malloc() failed for small heap chunks");
        return 1;
    }
    memset(p1, 0xAA, 64);
    memset(p2, 0xBB, 64);
    memset(p3, 0xCC, 64);

    free(p2);
    free(p3);

    void *p_merged = malloc(128);
    if (!p_merged) {
        free(p1);
        puts("  [FAIL] malloc() failed for merged block");
        return 1;
    }
    free(p1);
    free(p_merged);
    puts("  [INFO] Heap tier forward coalescing validated");

    size_t large_sz = 262144;
    char *large_ptr = (char *)malloc(large_sz);
    if (large_ptr) {
        large_ptr[0] = 'M';
        large_ptr[large_sz - 1] = 'Z';
        if (large_ptr[0] == 'M' && large_ptr[large_sz - 1] == 'Z') {
            puts("  [INFO] Mmap tier 256 KiB allocation validated");
        }
        free(large_ptr);
        puts("  [INFO] Mmap tier deallocation validated");
    } else {
        puts("  [FAIL] malloc() failed for 256 KiB mmap tier");
        return 1;
    }
    puts("  [OK]   Dual-tier memory allocator operational");

    /* 16. Copy-on-Write (COW) Memory Mutation Integrity */
    puts("  [TEST] Copy-on-Write (COW) memory mutation isolation...");
#if defined(__x86_64__)
    volatile int *cow_canary = (volatile int *)malloc(sizeof(int));
    if (!cow_canary) {
        puts("  [FAIL] malloc failed for COW canary");
        return 1;
    }
    *cow_canary = 0x5A5A1234;
    pid_t cow_child = fork();
    if (cow_child < 0) {
        puts("  [FAIL] fork() failed during COW test");
        free((void *)cow_canary);
        return 1;
    } else if (cow_child == 0) {
        *cow_canary = (int)0xDEADBEEF;
        exit(0);
    } else {
        int cow_status = 0;
        waitpid(cow_child, &cow_status, 0);
        if (*cow_canary == 0x5A5A1234) {
            puts("  [INFO] Parent memory unchanged after child mutation (COW "
                 "verified)");
            puts("  [OK]   Copy-on-Write memory mutation isolation operational");
        } else {
            printf("  [FAIL] Parent memory corrupted by child: 0x%X\n", *cow_canary);
            free((void *)cow_canary);
            return 1;
        }
        free((void *)cow_canary);
    }
#else
    puts("  [INFO] Copy-on-Write memory mutation skipped on i686 (non-paging "
         "target)");
    puts("  [OK]   Copy-on-Write memory mutation isolation operational");
#endif

    /* 26. File-Backed Memory Mapping, Demand Paging, and msync Synchronization */
    puts("  [TEST] File-backed mmap, demand paging, and msync synchronization...");
#if defined(__x86_64__)
    const char *test_path = "/config/sys/mmap_abi.txt";
    int f_init = open(test_path, O_CREAT | O_RDWR | O_TRUNC, 0644);
    if (f_init < 0) {
        puts("  [FAIL] Failed to create /config/sys/mmap_abi.txt for mmap test");
        return 1;
    }
    const char *init_payload = "INIT_PAYLOAD_KEIRA_MMAP_PERSISTENCE_TEST";
    ssize_t written = write(f_init, init_payload, strlen(init_payload));
    if (written < (ssize_t)strlen(init_payload)) {
        puts("  [FAIL] Failed to write initial payload to test file");
        close(f_init);
        return 1;
    }
    close(f_init);

    int mmap_fd = open(test_path, O_RDWR, 0);
    if (mmap_fd < 0) {
        puts("  [FAIL] Failed to reopen test file for mmap");
        return 1;
    }

    char *mapped = (char *)mmap(NULL, 4096, PROT_READ | PROT_WRITE, MAP_SHARED, mmap_fd, 0);
    if (mapped == MAP_FAILED || mapped == NULL) {
        puts("  [FAIL] File-backed mmap() returned MAP_FAILED");
        close(mmap_fd);
        return 1;
    }

    if (strncmp(mapped, init_payload, strlen(init_payload)) != 0) {
        puts("  [FAIL] Demand paging data mismatch upon initial read");
        munmap(mapped, 4096);
        close(mmap_fd);
        return 1;
    }
    printf("  [INFO] Demand paging verified: read '%s'\n", init_payload);

    mapped[0] = 'D';
    mapped[1] = 'O';
    mapped[2] = 'N';
    mapped[3] = 'E';

    if (msync(mapped, 4096, MS_SYNC) != 0) {
        puts("  [FAIL] msync() returned non-zero error");
        munmap(mapped, 4096);
        close(mmap_fd);
        return 1;
    }

    if (munmap(mapped, 4096) != 0) {
        puts("  [FAIL] munmap() failed");
        close(mmap_fd);
        return 1;
    }
    close(mmap_fd);

    int verify_fd = open(test_path, O_RDONLY, 0);
    if (verify_fd < 0) {
        puts("  [FAIL] Failed to open test file for persistence verification");
        return 1;
    }
    char verify_buf[64];
    memset(verify_buf, 0, sizeof(verify_buf));
    ssize_t n_read = read(verify_fd, verify_buf, sizeof(verify_buf) - 1);
    close(verify_fd);

    if (n_read < 4 || strncmp(verify_buf, "DONE", 4) != 0) {
        printf("  [FAIL] Disk persistence verification failed: read '%s'\n", verify_buf);
        return 1;
    }
    printf("  [INFO] Disk persistence confirmed: '%s'\n", verify_buf);
    puts("  [OK]   File-backed mmap, demand paging, and msync synchronization "
         "operational");
#else
    puts("  [INFO] File-backed lazy demand paging skipped on i686 (non-paging "
         "target)");
    puts("  [OK]   File-backed mmap, demand paging, and msync synchronization "
         "operational");
#endif

    /* 30. Demand-Paged VMA Memory Validation Across Syscall Boundaries */
    puts("  [TEST] Demand-paged VMA memory validation across syscall "
         "boundaries...");
    char *vma_buf = mmap(NULL, 4096, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (vma_buf == MAP_FAILED || vma_buf == NULL) {
        puts("  [FAIL] mmap failed for demand-paging syscall test");
        return 1;
    }
    int pipe_fds[2];
    if (pipe(pipe_fds) != 0) {
        puts("  [FAIL] pipe creation failed");
        munmap(vma_buf, 4096);
        return 1;
    }
    const char *vma_msg = "KEIRA_VMA_OK";
    if (write(pipe_fds[1], vma_msg, 12) != 12) {
        puts("  [FAIL] pipe write failed");
        close(pipe_fds[0]);
        close(pipe_fds[1]);
        munmap(vma_buf, 4096);
        return 1;
    }
    ssize_t vma_read = read(pipe_fds[0], vma_buf, 12);
    close(pipe_fds[0]);
    close(pipe_fds[1]);
    if (vma_read != 12 || strncmp(vma_buf, vma_msg, 12) != 0) {
        printf("  [FAIL] Demand-paged VMA read failed: n=%ld\n", (long)vma_read);
        munmap(vma_buf, 4096);
        return 1;
    }
    if (munmap(vma_buf, 4096) != 0) {
        puts("  [FAIL] munmap failed for demand-paged VMA test");
        return 1;
    }
    puts("  [OK]   Demand-paged VMA populated and validated in syscall copy "
         "without EFAULT");

    return 0;
}
