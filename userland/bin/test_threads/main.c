/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#define NUM_WORKERS 4
#define ITERATIONS_PER_WORKER 250

static volatile int shared_array[NUM_WORKERS] = {0};
static volatile int shared_counter = 0;
static pthread_mutex_t counter_mutex = PTHREAD_MUTEX_INITIALIZER;

static void *worker_func(void *arg) {
    long id = (long)arg;
    if (id < 0 || id >= NUM_WORKERS) {
        return (void *)-1;
    }

    /* Test 1 & 2: Modify shared array in shared address space */
    shared_array[id] = (int)(id + 100);

    /* Test 3: Mutex-protected increments under contention */
    for (int i = 0; i < ITERATIONS_PER_WORKER; i++) {
        pthread_mutex_lock(&counter_mutex);
        shared_counter++;
        pthread_mutex_unlock(&counter_mutex);
    }

    return (void *)(id + 1);
}

int main(int argc, char **argv) {
    (void)argc;
    (void)argv;

    puts("Keira Multi-Threading Test Suite");

    pthread_t threads[NUM_WORKERS];
    pthread_mutex_init(&counter_mutex, NULL);

    printf("[TEST] Spawning %d concurrent POSIX threads...\n", NUM_WORKERS);
    for (long i = 0; i < NUM_WORKERS; i++) {
        int ret = pthread_create(&threads[i], NULL, worker_func, (void *)i);
        if (ret != 0) {
            printf("[FAILED] pthread_create failed for thread %ld (errno=%d)\n", i, ret);
            return 1;
        }
        printf("  [OK] Thread %ld spawned successfully\n", i);
    }

    printf("[TEST] Joining worker threads via futex wait...\n");
    for (long i = 0; i < NUM_WORKERS; i++) {
        void *retval = NULL;
        int ret = pthread_join(threads[i], &retval);
        if (ret != 0) {
            printf("[FAILED] pthread_join failed for thread %ld (errno=%d)\n", i, ret);
            return 1;
        }
        long code = (long)retval;
        if (code != (i + 1)) {
            printf("[FAILED] Thread %ld returned unexpected code: %ld (expected: %ld)\n", i, code,
                   i + 1);
            return 1;
        }
        printf("  [OK] Thread %ld joined with exit code: %ld\n", i, code);
    }

    printf("[TEST] Validating CLONE_VM shared address space mutations...\n");
    for (int i = 0; i < NUM_WORKERS; i++) {
        if (shared_array[i] != (i + 100)) {
            printf("[FAILED] Shared array mismatch at index %d: %d (expected: %d)\n", i,
                   shared_array[i], i + 100);
            return 1;
        }
    }
    printf("  [OK] All worker mutations verified in shared address space\n");

    printf("[TEST] Validating mutex critical section protection...\n");
    int expected_total = NUM_WORKERS * ITERATIONS_PER_WORKER;
    if (shared_counter != expected_total) {
        printf("[FAILED] Mutex counter mismatch: %d (expected: %d)\n", shared_counter,
               expected_total);
        return 1;
    }
    printf("  [OK] Counter value = %d matches expected %d\n\n", shared_counter, expected_total);

    pthread_mutex_destroy(&counter_mutex);

    puts("\n[DONE] Multi-Threading Test Battery Passed.");
    return 0;
}
