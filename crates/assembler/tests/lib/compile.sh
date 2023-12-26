# Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
#
# This Source Code Form is subject to the terms of
# the Mozilla Public License version 2.0 and additional exceptions,
# more details in file LICENSE and CONTRIBUTING.

#!/bin/bash
gcc -Wall -g -fpic -shared -Wl,-soname,libtest0.so.1 -o libtest0.so.1.0.0 libtest0.c

if [ ! -f libtest0.so.1 ]
then
    ln -s libtest0.so.1.0.0 libtest0.so.1
fi

if [ ! -f libtest0.so ]
then
    ln -s libtest0.so.1.0.0 libtest0.so
fi

# compile the app:
# `gcc -Wall -g -o test_threads.elf test_threads.c -L $(pwd) -ltest0`
#
# run the app:
# `LD_LIBRARY_PATH=. ./test_threads.elf`

# set the shared library search path to the relative path '.' so that the
# applications can be run directly without setting 'LD_LIBRARY_PATH' or running 'ldconfig'.
gcc -Wall -g -Wl,-rpath,'$ORIGIN' -o test_threads.elf test_threads.c -L $(pwd) -ltest0
gcc -Wall -g -Wl,-rpath,'$ORIGIN' -o test_import_function.elf test_import_function.c -L $(pwd) -ltest0
gcc -Wall -g -Wl,-rpath,'$ORIGIN' -o test_indirect_function_call.elf test_indirect_function_call.c -L $(pwd) -ltest0
gcc -Wall -g -Wl,-rpath,'$ORIGIN' -o test_import_data.elf test_import_data.c -L $(pwd) -ltest0
gcc -Wall -g -Wl,-rpath,'$ORIGIN' -o test_import_tls_data.elf test_import_tls_data.c -L $(pwd) -ltest0