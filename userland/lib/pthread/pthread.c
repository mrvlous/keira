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
#include <malloc.h>
#include <pthread.h>
#include <sched.h>
#include <stdint.h>
#include <stdlib.h>
#include <sys/syscall.h>
#include <unistd.h>

struct pthread_internal {
    pthread_t id;
    int tid;
    int clear_tid;
    void *(*routine)(void *);
    void *arg;
    void *retval;
    int exited;
    void *stack_base;
    size_t stack_size;
};

static int pthread_trampoline(void *arg_struct) {
    struct pthread_internal *t = (struct pthread_internal *)arg_struct;
    if (t && t->routine) {
        t->retval = t->routine(t->arg);
    }
    if (t) {
        t->exited = 1;
    }
    return 0;
}

int pthread_create(pthread_t *thread, const pthread_attr_t *attr, void *(*start_routine)(void *),
                   void *arg) {
    if (!thread || !start_routine) {
        return EINVAL;
    }

    size_t stack_size = 65536;
    if (attr && attr->stack_size >= 4096) {
        stack_size = attr->stack_size;
    }

    struct pthread_internal *t = (struct pthread_internal *)malloc(sizeof(struct pthread_internal));
    if (!t) {
        return EAGAIN;
    }

    void *stack_base = malloc(stack_size);
    if (!stack_base) {
        free(t);
        return EAGAIN;
    }

    t->routine = start_routine;
    t->arg = arg;
    t->retval = NULL;
    t->exited = 0;
    t->stack_base = stack_base;
    t->stack_size = stack_size;
    t->clear_tid = 1;

    void *child_stack = (void *)((uintptr_t)stack_base + stack_size);
    int flags = CLONE_VM | CLONE_FS | CLONE_FILES | CLONE_SIGHAND | CLONE_THREAD |
                CLONE_PARENT_SETTID | CLONE_CHILD_CLEARTID;

    int child_tid = 0;
    int ret = clone(pthread_trampoline, child_stack, flags, t, &child_tid, NULL, &t->clear_tid);
    if (ret < 0) {
        int err = errno ? errno : EAGAIN;
        free(stack_base);
        free(t);
        return err;
    }

    t->tid = child_tid;
    t->id = (pthread_t)(uintptr_t)t;
    *thread = t->id;
    return 0;
}

int pthread_join(pthread_t thread, void **retval) {
    struct pthread_internal *t = (struct pthread_internal *)(uintptr_t)thread;
    if (!t) {
        return EINVAL;
    }

    while (!t->exited && t->clear_tid != 0) {
        syscall4(SYS_FUTEX, (uint64_t)(uintptr_t)&t->clear_tid, 0, (uint64_t)t->clear_tid, 0);
    }

    if (retval) {
        *retval = t->retval;
    }

    if (t->stack_base) {
        free(t->stack_base);
    }
    free(t);
    return 0;
}

void pthread_exit(void *retval) {
    (void)retval;
    sys_exit(0);
}

pthread_t pthread_self(void) {
    return (pthread_t)sys_getpid();
}

int pthread_mutex_init(pthread_mutex_t *mutex, const pthread_mutexattr_t *attr) {
    if (!mutex) {
        return EINVAL;
    }
    mutex->lock = 0;
    mutex->owner = 0;
    mutex->type = attr ? attr->type : PTHREAD_MUTEX_NORMAL;
    return 0;
}

int pthread_mutex_destroy(pthread_mutex_t *mutex) {
    if (!mutex) {
        return EINVAL;
    }
    mutex->lock = 0;
    mutex->owner = 0;
    return 0;
}

int pthread_mutex_trylock(pthread_mutex_t *mutex) {
    if (!mutex) {
        return EINVAL;
    }
    int expected = 0;
    if (__atomic_compare_exchange_n(&mutex->lock, &expected, 1, 0, __ATOMIC_ACQUIRE,
                                    __ATOMIC_RELAXED)) {
        mutex->owner = sys_getpid();
        return 0;
    }
    return EBUSY;
}

int pthread_mutex_lock(pthread_mutex_t *mutex) {
    if (!mutex) {
        return EINVAL;
    }
    int pid = sys_getpid();
    if (mutex->type == PTHREAD_MUTEX_RECURSIVE && mutex->owner == pid) {
        mutex->lock++;
        return 0;
    }

    while (1) {
        int expected = 0;
        if (__atomic_compare_exchange_n(&mutex->lock, &expected, 1, 0, __ATOMIC_ACQUIRE,
                                        __ATOMIC_RELAXED)) {
            mutex->owner = pid;
            return 0;
        }
        syscall4(SYS_FUTEX, (uint64_t)(uintptr_t)&mutex->lock, 0, 1, 0);
    }
}

int pthread_mutex_unlock(pthread_mutex_t *mutex) {
    if (!mutex) {
        return EINVAL;
    }
    if (mutex->type == PTHREAD_MUTEX_RECURSIVE && mutex->owner == sys_getpid()) {
        mutex->lock--;
        if (mutex->lock > 0) {
            return 0;
        }
    }
    mutex->owner = 0;
    __atomic_store_n(&mutex->lock, 0, __ATOMIC_RELEASE);
    syscall4(SYS_FUTEX, (uint64_t)(uintptr_t)&mutex->lock, 1, 1, 0);
    return 0;
}

int pthread_attr_init(pthread_attr_t *attr) {
    if (!attr) {
        return EINVAL;
    }
    attr->stack_size = 65536;
    attr->stack_addr = NULL;
    attr->detachstate = PTHREAD_CREATE_JOINABLE;
    return 0;
}

int pthread_attr_destroy(pthread_attr_t *attr) {
    if (!attr) {
        return EINVAL;
    }
    return 0;
}

int pthread_attr_setstacksize(pthread_attr_t *attr, size_t stacksize) {
    if (!attr || stacksize < 4096) {
        return EINVAL;
    }
    attr->stack_size = stacksize;
    return 0;
}

int pthread_attr_getstacksize(const pthread_attr_t *attr, size_t *stacksize) {
    if (!attr || !stacksize) {
        return EINVAL;
    }
    *stacksize = attr->stack_size;
    return 0;
}
