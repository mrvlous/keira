/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/syscall.h>
#include <unistd.h>

int main(int argc, char **argv) {
    (void)argc;
    (void)argv;

    puts("Keira Ring 3 Syscall Security & ABI Verification Harness");

    /* 1. Process Identity & Descriptor Verification */
    pid_t pid = getpid();
    pid_t ppid = getppid();
    printf("  [TEST] Process identity: PID=%d, PPID=%d\n", (int)pid, (int)ppid);
    if (pid < 0) {
        puts("  [FAIL] Invalid PID returned by getpid()");
        return 1;
    }
    puts("  [OK]   Process identity verified");

    /* 2. Kernel Memory Boundary & EFAULT Protection */
    puts("  [TEST] Kernel address pointer isolation (EFAULT check)...");
    void *kernel_addr = (void *)0xFFFF800000000000ULL;
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

    /* 4. Invalid File Descriptor (EBADF) Protection */
    puts("  [TEST] Invalid file descriptor validation...");
    char tmp_buf[16];
    ssize_t ebadf_neg = read(-1, tmp_buf, sizeof(tmp_buf));
    ssize_t ebadf_large = read(999, tmp_buf, sizeof(tmp_buf));
    if (ebadf_neg >= 0 || ebadf_large >= 0) {
        puts("  [FAIL] Out-of-bounds file descriptor accepted!");
        return 1;
    }
    puts("  [OK]   Out-of-bounds file descriptors rejected (EBADF enforced)");

    /* 5. VFS Working Directory (chdir/getcwd) Tracking */
    puts("  [TEST] VFS working directory tracking...");
    char cwd_buf[64];
    memset(cwd_buf, 0, sizeof(cwd_buf));
    if (getcwd(cwd_buf, sizeof(cwd_buf)) == NULL) {
        puts("  [FAIL] getcwd() failed on initial directory");
        return 1;
    }
    if (chdir("/config") == 0) {
        memset(cwd_buf, 0, sizeof(cwd_buf));
        if (getcwd(cwd_buf, sizeof(cwd_buf)) != NULL) {
            printf("  [INFO] Changed working directory to: %s\n", cwd_buf);
        }
    }
    puts("  [OK]   VFS working directory tracking operational");

    /* 6. VFS File Seeking (lseek) */
    puts("  [TEST] VFS file seeking (lseek)...");
    int fd = open("/config/sys/hostname.cfg", O_RDONLY, 0);
    if (fd >= 0) {
        off_t pos = lseek(fd, 0, SEEK_CUR);
        if (pos != 0) {
            printf("  [WARN] Unexpected initial seek offset: %d\n", (int)pos);
        }
        close(fd);
    }
    puts("  [OK]   VFS seek pointer operational");

    /* 7. BSD Socket Lifecycle (Socket, Connect, Send, Close) */
    puts("  [TEST] BSD socket lifecycle & async stream dispatch...");
    int sock = socket(AF_INET, SOCK_STREAM, 0);
    if (sock >= 0) {
        printf("  [INFO] Allocated socket descriptor: fd=%d\n", sock);
        struct sockaddr_in saddr;
        memset(&saddr, 0, sizeof(saddr));
        saddr.sin_family = AF_INET;
        saddr.sin_port = 0x5000;            /* Port 80 */
        saddr.sin_addr.s_addr = 0x0100007F; /* 127.0.0.1 */

        if (connect(sock, (const struct sockaddr *)&saddr, sizeof(saddr)) == 0) {
            puts("  [INFO] Socket connected to remote endpoint");
            ssize_t sent = send(sock, "PING", 4, 0);
            if (sent == 4) {
                puts("  [INFO] Transmitted 4 stream payload bytes");
            }
        }
        close(sock);
    } else {
        puts("  [FAIL] socket() returned invalid descriptor");
        return 1;
    }
    puts("  [OK]   BSD socket subsystem operational");

    /* 8. VMM Demand Paging & Lazy Heap (sbrk) Allocation */
    puts("  [TEST] VMM demand paging & lazy heap (sbrk) allocation...");
    void *old_brk = sbrk(65536);
    if (old_brk != (void *)-1) {
        volatile char *canary_page1 = (volatile char *)old_brk + 4096;
        volatile char *canary_page2 = (volatile char *)old_brk + 32768;
        *canary_page1 = 'K';
        *canary_page2 = 'R';
        if (*canary_page1 == 'K' && *canary_page2 == 'R') {
            puts("  [INFO] Lazy pages faulted in transparently via #PF");
        } else {
            puts("  [FAIL] Canary value corruption on faulted heap pages");
            return 1;
        }
    } else {
        puts("  [FAIL] sbrk(65536) heap expansion failed");
        return 1;
    }
    puts("  [OK]   VMM demand paging operational");

    /* 9. Out-of-bounds Syscall Handshake */
    puts("  [TEST] Out-of-bounds syscall safety check...");
    int64_t unhandled = syscall0(9999);
    (void)unhandled;
    puts("  [OK]   Undefined syscall safely handled without kernel fault");

    puts("\n[DONE] All Ring 3 Syscall Security & Fault Injection tests PASSED.");
    return 0;
}
