/**
 * Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
 *
 * This Source Code Form is subject to the terms of
 * the Mozilla Public License version 2.0 and additional exceptions,
 * more details in file LICENSE and CONTRIBUTING.
 */

#include <stdio.h>
#include <stdlib.h>
#include <pthread.h>
#include <unistd.h>
#include <time.h>

extern void inc_tls(int);
extern int get_tls_var();
extern __thread int tls_var;

void sleep_100ms()
{
    struct timespec ts;
    ts.tv_sec = 0;
    ts.tv_nsec = 100 * 1000;
    nanosleep(&ts, NULL);
}

void *child_thread_start(void *arg)
{
    pthread_t tid = pthread_self();

    printf("%ld >> init: %d\n", tid, tls_var);
    printf("%ld >> init (from lib): %d\n", tid, get_tls_var());
    sleep_100ms();

    inc_tls(11);
    printf("%ld >> after inc: %d\n", tid, tls_var);
    printf("%ld >> after inc (from lib): %d\n", tid, get_tls_var());
    sleep_100ms();

    tls_var = 13;
    printf("%ld >> after set: %d\n", tid, tls_var);
    printf("%ld >> after set (from lib): %d\n", tid, get_tls_var());
    sleep_100ms();

    pthread_exit(NULL);
}

void test_threads(void)
{
    int num_threads = 5;
    pthread_t tid[num_threads];

    for (int i = 0; i < num_threads; i++)
    {
        pthread_create(&tid[i], NULL, &child_thread_start, NULL);
    }

    for (int i = 0; i < num_threads; i++)
    {
        pthread_join(tid[i], NULL);
    }
}

void test_single_thread(void)
{
    printf("init: %d\n", tls_var);
    printf("init (from lib): %d\n", get_tls_var());

    inc_tls(11);
    printf("after inc: %d\n", tls_var);
    printf("after inc (from lib): %d\n", get_tls_var());

    tls_var = 13;
    printf("after set: %d\n", tls_var);
    printf("after set (from lib): %d\n", get_tls_var());
}

int main(int argc, char *argv[])
{
    test_single_thread();
    // test_threads();
    exit(EXIT_SUCCESS);
}