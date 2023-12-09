// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

// note:
//
// the data types that VM supports:
//
// - i64
// - f64
// - i32
// - f32
//
// note that 'i32' above means a 32-bit integer, which is equivalent to
// the 'uint32_t' in C or 'u32' in Rust. do not confuse it with 'i32' in Rust.
// the same applies to the i8, i16 and i64.
//
// data layout:
//
// the default implement of XiaoXuan VM is stack-base, which its operands are
// 8-byte raw data, the data presentation is as the following:
//
//    MSB                             LSB
// 64 |---------------------------------| 0
//    |   16     16      16     8    8  | bits
//    |-------|-------|-------|----|----|
//    |               i64               | <-- native data type
//    |---------------------------------|
//    |???????????????|        i32      |
//    |---------------------------------|
//    |???????????????|-sign--|   i16   |
//    |---------------------------------|
//    |???????????????|-sign-extend| i8 |
//    |---------------------------------|
//    |               f64               | <-- native data type
//    |---------------------------------|
//    |???????????????|        f32      |
//    |---------------------------------|
//
// the value of the high end part of operand is undefined.

// note:
//
// the floating-point number:
//
// like most processors and VM, f32/f64 is stored with
// IEEE 754-2008 format, in addition to the normal floating-point numbers,
// there are some special values (variants):
//
// - NaN
// - +Infinity, -Infinity
// - +0, -0
//
// these variants make the programming language become complex and
// sometimes make problems unpredictable, for example:
//
// - NaN != NaN
// - (NaN < 0) == false, (NaN > 0) == false
// - (a != b) cannot assert that !(a == b)
// - 1/-0 = -Infinity
//
// ref:
//
// - https://en.wikipedia.org/wiki/Floating-point_arithmetic
// - https://en.wikipedia.org/wiki/IEEE_754
// - https://en.wikipedia.org/wiki/IEEE_754-2008_revision
// - https://en.wikipedia.org/wiki/Signed_zero
// - https://en.wikipedia.org/wiki/Subnormal_number
// - https://en.wikipedia.org/wiki/Single-precision_floating-point_format
// - https://en.wikipedia.org/wiki/Half-precision_floating-point_format
// - https://doc.rust-lang.org/std/primitive.f32.html
//
// to simplify the upper-level programming language, the f32/f64 in XiaoXuan VM
// only support the normal (includes subnormal) floating-point number and +0,
// and the VM will simply throw an exception when other variants are encountered.
//
// e.g. for f32:
//
//                   MSB                                  LSB
//                   sign    exponent (8 bits)   fraction (23 bits)                                 implicit leading number 1
//                   ----    -----------------   ------------------                                 |
//                   |       |                   |                                                  v
//          format   0       00000000            0000000000 0000000000 000     value = (-1)^sign * (1 fraction) * 2^(exponent-offset), offset = 127 for f32, 1023 for f64
//          example  1       10000001            0100000000 0000000000 000     value = (-1)^1 * (1 + 0*2^(-1) + 1*2^(-2)) * 2^(129-127) = -1 * 1.25 * 4 = -5.0
//          example  0       01111100            0100000000 0000000000 000     value = (-1)^0 * 1.25 * 2^(-3) = 0.15625
// support?
//  Y                -       00000001--\
//                           11111110--/         ---------- ---------- ---     normal number
//  Y                0       00000000            0000000000 0000000000 000     value = +0
//  N                1       00000000            0000000000 0000000000 000     value = -0
//  Y                -       00000000            ---------- ---------- ---     subnormal number (i.e., numbers between 0 and MIN)
//  N                -       11111111            0000000000 0000000000 000     value = +/- Infinity
//  N                -       11111111            ---------- ---------- ---     NaN
//
// when load data from memory as floating-point number, there are some checking as the following:
// 1. exponent between (00000001) and (11111110): pass
// 2. exponent is zero, if the sign bit is zero: pass
// 3. failed.
//
// in other words, the +/-Infinity, -0, NaN, will cause the VM to throw exceptions.

// note:
//
// the boolean type:
//
// the boolean value is represented by a i32 number:
// - TRUE, the number is `1:i32`,
// - FALSE, the number is `0:i32`.
//
// on the other hand, 0:i32 and 0:i64 are both treated as FALSE,
// and all other i32/i64 non-zero are treated as TRUE.

// note:
//
// the instruction schemes:
//
// XiaoXuan VM instructions are not fixed-length code. there are
// 16 bits, 32 bits, 64 bits and 96 bits instructions, it is
// necessary to insert the `nop` instruction between the 16 (32) bits instruction and the 64 (96) bits
// instruction to achieve 32 bits (4-byte) alignment.
//
// - 16 bits:
//   instructions without parameters, such as `i32_eq`, `i32_add`.
// - 32 bits:
//   instructions with 1 parameter, such as `heap_load`, `heap_store`.
//   16 bits opcode + 16 bits parameter
// - 64 bits:
//   instructions with 1 parameter, such as `i32_imm`, `f32_imm`, `call`.
//   16 bits opcode + 16 bits padding + 32 bits parameter (ALIGN 4-byte alignment require)
// - 64 bits:
//   instructions with 2 parameters, such as `data_load`, `break`, `recur`.
//   16 bits opcode + 16 bits parameter 0 + 32 bits parameter 1 (ALIGN 4-byte alignment require)
// - 64 bits:
//   instructions with 3 parameter, such as `local_load`, `local_store`,
//   16 bits opcode + 16 bits parameter 1 + 16 bits parameter 2 + 16 bits parameter 3
// - 96 bits
//   instructions with 2 parameters, such as `i64_imm`, `f64_imm`. `block`, 'block_nez'
//   (only these instructions currently are 96 bits)
//   16 bits opcode + 16 bits padding + 32 bits parameter 0 + 32 bits parameter 1 (ALIGN 4-byte alignment require)
// - 128 bits
//   instructions with 3 parameters, such as `block_alt`
//   (only these instructions currently are 128 bits)
//   16 bits opcode + 16 bits padding + 32 bits parameter 0 + 32 bits parameter 1 + 32 bits parameter 2 (ALIGN 4-byte alignment require)

// the simplified schemes:
//
// - [opcode i16]                                                                               ;; 16-bit
// - [opcode i16] - [param i16    ]                                                             ;; 32-bit
// - [opcode i16] - [pading 16-bit] + [param i32              ]                                 ;; 64-bit
// - [opcode i16] - [param i16    ] + [param i32              ]                                 ;; 64-bit
// - [opcode i16] - [param i16    ] + [param i16] + [param i16]                                 ;; 64-bit
// - [opcode i16] - [pading 16-bit] + [param i32              ] + [param i32]                   ;; 96-bit
// - [opcode i16] - [pading 16-bit] + [param i32              ] + [param i32] + [param i32]     ;; 128-bit
//
// the opcode scheme:
//
// MSB           LSB
// 00000000 00000000
// -------- --------
// |        |
// |        \ items
// |
// \ catalogs
//
#[repr(u16)]
#[derive(Debug, PartialEq, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum Opcode {
    //
    // fundamental
    //
    nop = 0x100,                // instruction to do nothing,
                                // it's usually used for padding instructions to archieve 32/64 bits (4/8-byte) alignment.
    zero,                       // push 0 (i64) onto stack                          () -> i64
    drop,                       // drop one operand (the top most operand)          (operand op:any) -> ()
    duplicate,                  // duplicate one operand (the top most operand)     (operand op:any) -> any
    swap,                       // swap the top two operands                        (operand left:any right:any) -> (any, any)
    select_nez,                 // (operand when_true:any when_false:any test:i32) -> any
                                //
                                // | test    | a
                                // | false   | b
                                // | true    | c
                                // | ...     |
                                // \---------/
                                //
                                // pop operands a, b and c, then push c if a!=0, otherwise push b.

                                // b and c should be the same data type.
    i32_imm = 0x180,            // (param immediate_number:i32) -> i32
    i64_imm,                    // (param immediate_number_low:i32, immediate_number_high:i32) -> i64
    f32_imm,                    // (param immediate_number:i32) -> f32
    f64_imm,                    // (param immediate_number_low:i32, immediate_number_high:i32) -> f64

    // some ISA (VM or real machine) place the immediate numbers in a list of constants
    // in the program image, and then load the constants by address to archieve the
    // purpose of loading the immediate numbers, the ARM ISA has a similar scheme, it
    // pace large immediate numbers in the instruction area outside of the current function
    // (or inside the function and using instruction 'jump' to skip these area so that they
    // are not parsed as instructions).
    // however, the XixoaXuan VM ISA are designed as variable-length and don't necessarily
    // require the program to have a data section or heap,so the immediate numbers are
    // placed directly after the 'imm' instruction.

    // i32/i64/f32/f64 load/store instructions require the address and offset alignment:
    //
    // | data type | align (bytes) |
    // |-----------|---------------|
    // | i8        | 1             |
    // | i16       | 2             |
    // | i32       | 4             |
    // | i64       | 8             |
    // | f32       | 4             |
    // | f64       | 8             |

    // local variables loading and storing:
    //
    // load the specified local variable and push onto to the stack, or
    // pop one operand off the stack and set the specified local variable.
    //
    // note that arguments of function or block are also local variables, the index of arguments are
    // follow the local variables, e.g. consider there are 4 local variables in a function which has
    // 2 parameters, the indices of them are as the following:
    //
    //     local variable      arguments
    //     [i32 i32 i64 i64]  [i32 i32]
    // idx  0   1   2   3      4   5

    // note about the INDEX of local variable (data, function):
    //
    // using the 'index', rather than the 'address/pointer' to access local variables (including
    // data in the data section and functions talked about in the following sections) is the
    // security strategy of the XiaoXuan ISA and VM.
    // because the 'index' includes the type, data length and location (the safe access range) of the 'object',
    // when accessing the object, the VM can check whether the type of the object, and the range is legal
    // or not, so it can prevent a lot of unsafe accessing.
    // for example, the traditional method of using pointers to access a array is very easy
    // to read/write data outside the range.

    // note:
    // in some stack base VM, the arguments of a function are placed on the top
    // of the stack, so it is also possible to read the arguments directly in the function
    // using instructions with the POP function (e.g. the comparison instructions, the arithmetic
    // instructions).
    // this feature can be used as a trick to improve performance, but the XiaoXuan ISA doesn't
    // provide this feature. please note that to access arguments you should always using the index.
    local_load64_i64 = 0x200,   // load local variable      (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16) -> i64
    local_load64_f64,           // Load f64 with floating-point validity check.     (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16) -> f64
    local_load32_i32,           //                          (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16) -> i32
    local_load32_i16_s,         //                          (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16) -> i32
    local_load32_i16_u,         //                          (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16) -> i32
    local_load32_i8_s,          //                          (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16) -> i32
    local_load32_i8_u,          //                          (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16) -> i32
    local_load32_f32,           // Load f32 with floating-point validity check.     (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16) -> f32
    local_store64,              // store local variable     (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16)    (operand number:i64) -> ()
    local_store32,              //                          (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16)    (operand number:i32) -> ()
    local_store16,              //                          (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16)    (operand number:i32) -> ()
    local_store8,               //                          (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16)    (operand number:i32) -> ()

    local_long_load64_i64 = 0x280,  //                      (param reversed_index:i16 local_variable_index:i32)     (operand offset_bytes:i32) -> i64
    local_long_load64_f64,      //                          (param reversed_index:i16 local_variable_index:i32)     (operand offset_bytes:i32) -> f64
    local_long_load32_i32,      //                          (param reversed_index:i16 local_variable_index:i32)     (operand offset_bytes:i32) -> i32
    local_long_load32_i16_s,    //                          (param reversed_index:i16 local_variable_index:i32)     (operand offset_bytes:i32) -> i32
    local_long_load32_i16_u,    //                          (param reversed_index:i16 local_variable_index:i32)     (operand offset_bytes:i32) -> i32
    local_long_load32_i8_s,     //                          (param reversed_index:i16 local_variable_index:i32)     (operand offset_bytes:i32) -> i32
    local_long_load32_i8_u,     //                          (param reversed_index:i16 local_variable_index:i32)     (operand offset_bytes:i32) -> i32
    local_long_load32_f32,      //                          (param reversed_index:i16 local_variable_index:i32)     (operand offset_bytes:i32) -> f32
    local_long_store64,         //                          (param reversed_index:i16 local_variable_index:i32)     (operand offset_bytes:i32 number:i64) -> ()
    local_long_store32,         //                          (param reversed_index:i16 local_variable_index:i32)     (operand offset_bytes:i32 number:i32) -> ()
    local_long_store16,         //                          (param reversed_index:i16 local_variable_index:i32)     (operand offset_bytes:i32 number:i32) -> ()
    local_long_store8,          //                          (param reversed_index:i16 local_variable_index:i32)     (operand offset_bytes:i32 number:i32) -> ()

    // note:
    // - there are 2 sets of local load/store instructions, one set is the
    //   local_load.../local_store.., they are designed to access primitive type data
    //   and struct data, the other set is the local_long_load.../local_long_store..., they
    //   are designed to access long byte-type data.
    // - the local_(long_)load32* instructions will leave the value of the high part of
    //   operand (on the stack) undefined/unpredictable.

    //
    // data (thread-local variables) loading and storing
    //
    // load the specified data and push onto to the stack, or
    // pop one operand off the stack and set the specified data
    //
    data_load64_i64 = 0x300,    // load data                (param offset_bytes:i16 data_public_index:i32) -> i64
    data_load64_f64,            // Load f64 with floating-point validity check.     (param offset_bytes:i16 data_public_index:i32) -> f64
    data_load32_i32,            //                          (param offset_bytes:i16 data_public_index:i32) -> i32
    data_load32_i16_s,          //                          (param offset_bytes:i16 data_public_index:i32) -> i32
    data_load32_i16_u,          //                          (param offset_bytes:i16 data_public_index:i32) -> i32
    data_load32_i8_s,           //                          (param offset_bytes:i16 data_public_index:i32) -> i32
    data_load32_i8_u,           //                          (param offset_bytes:i16 data_public_index:i32) -> i32
    data_load32_f32,            // Load f32 with floating-point validity check.     (param offset_bytes:i16 data_public_index:i32) -> f32
    data_store64,               // store data               (param offset_bytes:i16 data_public_index:i32)      (operand number:i64) -> ()
    data_store32,               //                          (param offset_bytes:i16 data_public_index:i32)      (operand number:i32) -> ()
    data_store16,               //                          (param offset_bytes:i16 data_public_index:i32)      (operand number:i32) -> ()
    data_store8,                //                          (param offset_bytes:i16 data_public_index:i32)      (operand number:i32) -> ()

    // there are also 2 sets of data load/store instructions, one set is the
    // data_load.../data_store.., they are designed to access primitive type data
    // and struct data, the other set is the data_long_load.../data_long_store..., they
    // are designed to access long byte-type data.
    data_long_load64_i64 = 0x380,   //                              (param data_public_index:i32)   (operand offset_bytes:i32) -> i64
    data_long_load64_f64,       //                                  (param data_public_index:i32)   (operand offset_bytes:i32) -> f64
    data_long_load32_i32,       //                                  (param data_public_index:i32)   (operand offset_bytes:i32) -> i32
    data_long_load32_i16_s,     //                                  (param data_public_index:i32)   (operand offset_bytes:i32) -> i32
    data_long_load32_i16_u,     //                                  (param data_public_index:i32)   (operand offset_bytes:i32) -> i32
    data_long_load32_i8_s,      //                                  (param data_public_index:i32)   (operand offset_bytes:i32) -> i32
    data_long_load32_i8_u,      //                                  (param data_public_index:i32)   (operand offset_bytes:i32) -> i32
    data_long_load32_f32,       //                                  (param data_public_index:i32)   (operand offset_bytes:i32) -> f32
    data_long_store64,          //                                  (param data_public_index:i32)   (operand offset_bytes:i32 number:i64) -> ()
    data_long_store32,          //                                  (param data_public_index:i32)   (operand offset_bytes:i32 number:i32) -> ()
    data_long_store16,          //                                  (param data_public_index:i32)   (operand offset_bytes:i32 number:i32) -> ()
    data_long_store8,           //                                  (param data_public_index:i32)   (operand offset_bytes:i32 number:i32) -> ()

    // note
    // both local variables and data have NO data type, they both are
    // bytes in the memory. so you can call 'local_store8' and 'local_load32_i8_u'
    // even if the local variable is defined as i64.

    //
    // heap (thread-local memory) loading and storing
    //

    // note that the address of heap is a 64-bit integer number, which means that you
    // must write the target address (to stack) using the
    // instructions 'i64_imm', 'local_(long_)load' or 'data_(long_)load'.
    // do NOT use the 'i32_imm', 'local_(long_)load32' or 'data_(long_)load32', because
    // the latter instructions leave the value of the high part of
    // operand (on the stack) undefined/unpredictable.
    heap_load64_i64 = 0x400,    // load heap                        (param offset_bytes:i16)    (operand heap_addr:i64) -> i64
    heap_load64_f64,            // Load f64 with floating-point validity check.     (param offset_bytes:i16)    (operand heap_addr:i64) -> f64
    heap_load32_i32,            //                                  (param offset_bytes:i16)    (operand heap_addr:i64) -> i32
    heap_load32_i16_s,          //                                  (param offset_bytes:i16)    (operand heap_addr:i64) -> i32
    heap_load32_i16_u,          //                                  (param offset_bytes:i16)    (operand heap_addr:i64) -> i32
    heap_load32_i8_s,           //                                  (param offset_bytes:i16)    (operand heap_addr:i64) -> i32
    heap_load32_i8_u,           //                                  (param offset_bytes:i16)    (operand heap_addr:i64) -> i32
    heap_load32_f32,            // Load f32 with floating-point validity check.     (param offset_bytes:i16)    (operand heap_addr:i64) -> f32
    heap_store64,               // store heap                       (param offset_bytes:i16)    (operand heap_addr:i64 number:i64) -> ()
    heap_store32,               //                                  (param offset_bytes:i16)    (operand heap_addr:i64 number:i32) -> ()
    heap_store16,               //                                  (param offset_bytes:i16)    (operand heap_addr:i64 number:i32) -> ()
    heap_store8,                //                                  (param offset_bytes:i16)    (operand heap_addr:i64 number:i32) -> ()

    // heap memory
    heap_fill = 0x480,          // fill the specified memory region with the specified (i8) value
                                // (operand offset:i64 value:i8 count:i64) -> ()
    heap_copy,                  // copy the specified memory region to the specified location
                                // (operand dst_offset:i64 src_offset:i64 length_in_bytes:i64) -> ()
    heap_capacity,              // return the amount of pages of the thread-local
                                // memory (i.e. heap), each page is MEMORY_PAGE_SIZE_IN_BYTES (64 KiB) by default
                                // () -> pages:i64
    heap_resize,                // increase or decrease the heap size return the new capacity (in pages)
                                // (operand pages:i64) -> new_pages:i64

    //
    // conversion
    //

    // truncate i64 to i32
    // discard the high 32 bits of an i64 number directly
    i32_truncate_i64 = 0x500,   // (operand number:i64) -> i32

    // extend i32 to i64
    i64_extend_i32_s,           // (operand number:i32) -> i64
    i64_extend_i32_u,           // (operand number:i32) -> i64

    // demote f64 to f32
    f32_demote_f64,             // (operand number:f64) -> f32

    // promote f32 to f64
    f64_promote_f32,            // (operand number:i32) -> f64

    // convert float to int
    // truncate fractional part
    i32_convert_f32_s,          //                                  (operand number:f32) -> i32
    i32_convert_f32_u,          // note -x.xx(float) -> 0(int)      (operand number:f32) -> i32
    i32_convert_f64_s,          //                                  (operand number:f64) -> i32
    i32_convert_f64_u,          // note -x.xx(float) -> 0(int)      (operand number:f64) -> i32
    i64_convert_f32_s,          //                                  (operand number:f32) -> i64
    i64_convert_f32_u,          // note -x.xx(float) -> 0(int)      (operand number:f32) -> i64
    i64_convert_f64_s,          //                                  (operand number:f64) -> i64
    i64_convert_f64_u,          // note -x.xx(float) -> 0(int)      (operand number:f64) -> i64

    // convert int to float
    f32_convert_i32_s,          // (operand number:i32) -> f32
    f32_convert_i32_u,          // (operand number:i32) -> f32
    f32_convert_i64_s,          // (operand number:i64) -> f32
    f32_convert_i64_u,          // (operand number:i64) -> f32
    f64_convert_i32_s,          // (operand number:i32) -> f64
    f64_convert_i32_u,          // (operand number:i32) -> f64
    f64_convert_i64_s,          // (operand number:i64) -> f64
    f64_convert_i64_u,          // (operand number:i64) -> f64

    //
    // comparsion
    //

    // for the binary operations, the first one pops up from the
    // stack is the right-hand-side-value, e.g.
    //
    // |                 | --> stack end
    // | right hand side | --> 1st pop: RHS
    // | left hand side  | --> 2nd pop: LHS
    // \-----------------/ --> stack start
    //
    // it is the same order as the function parameter, e.g.
    // function `add (a, b)`
    // the parameters in the stack is:
    //
    //  |   |
    //  | b |
    //  | a |
    //  \---/
    //
    // note that two operands MUST be the same data type.

    // the result of the comparison is a logical TRUE or FALSE (i.e., the data type 'boolean'),
    // when the result is TRUE, the number `1:i32` is pushed onto the stack,
    // and vice versa the number is `0:i32`.

    // instruction `i32_lt_u` example:
    //
    // ```
    // ;; load 2 numbers on to the stack
    // (i32.imm 11)
    // (i32.imm 22)
    //
    // ;; now the stack layout is:
    // ;;
    // ;; |    |
    // ;; | 22 |
    // ;; | 11 |
    // ;; \----/
    //
    // ;; check if '11' is less then '22', i.e. `11 < 22 ?`
    // ;; `1` will be pushed on to the stack
    // i32.lt_u
    //
    // ;; now the stack layout is:
    // ;;
    // ;; |    |
    // ;; | 1  |
    // ;; \----/
    // ```
    i32_eqz = 0x600,            // (operand number:i32) -> i32
    i32_nez,                    // (operand number:i32) -> i32
    i32_eq,                     // (operand left:i32 right:i32) -> i32
    i32_ne,                     // (operand left:i32 right:i32) -> i32
    i32_lt_s,                   // (operand left:i32 right:i32) -> i32
    i32_lt_u,                   // (operand left:i32 right:i32) -> i32
    i32_gt_s,                   // (operand left:i32 right:i32) -> i32
    i32_gt_u,                   // (operand left:i32 right:i32) -> i32
    i32_le_s,                   // redundant    (operand left:i32 right:i32) -> i32
    i32_le_u,                   // redundant    (operand left:i32 right:i32) -> i32
    i32_ge_s,                   // redundant    (operand left:i32 right:i32) -> i32
    i32_ge_u,                   // redundant    (operand left:i32 right:i32) -> i32

    i64_eqz,                    // (operand number:i64) -> i32
    i64_nez,                    // (operand number:i64) -> i32
    i64_eq,                     // (operand left:i64 right:i64) -> i32
    i64_ne,                     // (operand left:i64 right:i64) -> i32
    i64_lt_s,                   // (operand left:i64 right:i64) -> i32
    i64_lt_u,                   // (operand left:i64 right:i64) -> i32
    i64_gt_s,                   // (operand left:i64 right:i64) -> i32
    i64_gt_u,                   // (operand left:i64 right:i64) -> i32
    i64_le_s,                   // redundant    (operand left:i64 right:i64) -> i32
    i64_le_u,                   // redundant    (operand left:i64 right:i64) -> i32
    i64_ge_s,                   // redundant    (operand left:i64 right:i64) -> i32
    i64_ge_u,                   // redundant    (operand left:i64 right:i64) -> i32

    f32_eq,                     // (operand left:f32 right:f32) -> i32
    f32_ne,                     // (operand left:f32 right:f32) -> i32
    f32_lt,                     // (operand left:f32 right:f32) -> i32
    f32_gt,                     // (operand left:f32 right:f32) -> i32
    f32_le,                     // (operand left:f32 right:f32) -> i32
    f32_ge,                     // (operand left:f32 right:f32) -> i32

    f64_eq,                     // (operand left:f64 right:f64) -> i32
    f64_ne,                     // (operand left:f64 right:f64) -> i32
    f64_lt,                     // (operand left:f64 right:f64) -> i32
    f64_gt,                     // (operand left:f64 right:f64) -> i32
    f64_le,                     // (operand left:f64 right:f64) -> i32
    f64_ge,                     // (operand left:f64 right:f64) -> i32

    //
    // arithmetic
    //
    i32_add = 0x700,            // (operand left:i32 right:i32) -> i32
                                // wrapping add, e.g. 0xffff_ffff + 2 = 1 (-1 + 2 = 1)
    i32_sub,                    // (operand left:i32 right:i32) -> i32
                                // wrapping sub, e.g. 11 - 211 = -200
    i32_mul,                    // (operand left:i32 right:i32) -> i32
                                // wrapping mul, e.g. 0xf0e0d0c0 * 2 = 0xf0e0d0c0 << 1
    i32_div_s,                  // (operand left:i32 right:i32) -> i32
    i32_div_u,                  // (operand left:i32 right:i32) -> i32
    i32_rem_s,                  // calculate the remainder      (operand left:i32 right:i32) -> i32
    i32_rem_u,                  // (operand left:i32 right:i32) -> i32
    i32_inc,                    // (param amount:i16) (operand number:i32) -> i32
                                // wrapping inc, e.g. 0xffff_ffff inc 2 = 1
    i32_dec,                    // (param amount:i16) (operand number:i32) -> i32
                                // wrapping dec, e.g. 0x1 dec 2 = 0xffff_ffff

    // remainder vs modulus
    // --------------------
    // The remainder (%) operator returns the remainder left over when one operand is
    // divided by a second operand. It always takes the sign of the dividend.
    // For the operation n % d, n is called the dividend and d is called the divisor.
    //
    // (13 % 5) = 3
    //  ^    ^
    //  |    |divisor
    //  |dividend <--------- the result always takes the sign of the dividend.
    //
    // (-13 % 5) = -3
    // (4 % 2) = 0
    // (-4 % 2) = -0
    //
    // ref: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Remainder
    //
    // Example of Modulus:
    //
    // 5 % 3 = 2 [here divisible is 5 which is positively signed so the remainder will also be
    // positively signed and the divisor is also positively signed. As both remainder and divisor
    // are of same sign the result will be same as remainder]
    //
    // -5 % 3 = 1 [here divisible is -5 which is negatively signed so the remainder will also be
    // negatively signed and the divisor is positively signed. As both remainder and divisor are
    // of opposite sign the result will be sum of remainder and divisor -2 + 3 = 1]
    //
    // 5 % -3 = -1 [here divisible is 5 which is positively signed so the remainder will also
    // be positively signed and the divisor is negatively signed. As both remainder and divisor
    // are of opposite sign the result will be sum of remainder and divisor 2 + -3 = -1]
    //
    // -5 % -3 = -2 [here divisible is -5 which is negatively signed so the remainder will also
    // be negatively signed and the divisor is also negatively signed. As both remainder and
    // divisor are of same sign the result will be same as remainder]
    //
    // ref: https://stackoverflow.com/questions/13683563/whats-the-difference-between-mod-and-remainder
    // ref: https://en.wikipedia.org/wiki/Euclidean_division
    // ref: https://en.wikipedia.org/wiki/Modulo

    // instruction `i32_add` example:
    //
    // ```
    // ;; load two numbers onto the stack
    // (i32.imm 10)
    // (i32.imm 3)
    // ;; subtract one number from the other
    // ;; the top item on the stack will be 7 (10 - 3 = 7)
    // i32.sub
    // ```

    // instruction `i32.rem_u` example:
    //
    // ```
    // ;; load two numbers onto the stack
    // (i32.imm 10)
    // (i32.imm 3)
    // ;; calculate the remainder of dividing one number by the other
    // ;; the top item on the stack will be 1 (10 % 3 = 1)
    // i32.rem_u
    // ```
    i64_add,                    // wrapping add     (operand left:i64 right:i64) -> i64
    i64_sub,                    // wrapping sub     (operand left:i64 right:i64) -> i64
    i64_mul,                    // wrapping mul     (operand left:i64 right:i64) -> i64
    i64_div_s,                  // (operand left:i64 right:i64) -> i64
    i64_div_u,                  // (operand left:i64 right:i64) -> i64
    i64_rem_s,                  // (operand left:i64 right:i64) -> i64
    i64_rem_u,                  // (operand left:i64 right:i64) -> i64
    i64_inc,                    // wrapping inc     (param amount:i16) (operand number:i64) -> i64
    i64_dec,                    // wrapping dec     (param amount:i16) (operand number:i64) -> i64

    f32_add,                    // (operand left:f32 right:f32) -> f32
    f32_sub,                    // (operand left:f32 right:f32) -> f32
    f32_mul,                    // (operand left:f32 right:f32) -> f32
    f32_div,                    // (operand left:f32 right:f32) -> f32

    f64_add,                    // (operand left:f64 right:f64) -> f64
    f64_sub,                    // (operand left:f64 right:f64) -> f64
    f64_mul,                    // (operand left:f64 right:f64) -> f64
    f64_div,                    // (operand left:f64 right:f64) -> f64

    //
    // bitwise
    //

    // see also:
    // https://en.wikipedia.org/wiki/Bitwise_operation
    i32_and = 0x800,            // bitwise AND                  (operand left:i32 right:i32) -> i32
    i32_or,                     // bitwise OR                   (operand left:i32 right:i32) -> i32
    i32_xor,                    // bitwise XOR                  (operand left:i32 right:i32) -> i32
    i32_shift_left,             // left shift                   (operand number:i32 move_bits:i32) -> i32       // move_bits = [0,32)
    i32_shift_right_s,          // arithmetic right shift       (operand number:i32 move_bits:i32) -> i32       // move_bits = [0,32)
    i32_shift_right_u,          // logical right shift          (operand number:i32 move_bits:i32) -> i32       // move_bits = [0,32)
    i32_rotate_left,            // left rotate                  (operand number:i32 move_bits:i32) -> i32       // move_bits = [0,32)
    i32_rotate_right,           // right rotate                 (operand number:i32 move_bits:i32) -> i32       // move_bits = [0,32)
    i32_not,                    // bitwise NOT                  (operand number:i32) -> i32
    i32_leading_zeros,          // count leading zeros          (operand number:i32) -> i32
    i32_trailing_zeros,         // count trailing zeros         (operand number:i32) -> i32
    i32_count_ones,             // count the number of ones in the binary representation     (operand number:i32) -> i32

    // instruction `i32.shl` example:
    //
    // ```
    // ;; load two numbers onto the stack
    // (i32.imm 7)              ;; 00000111
    // ;; perform a bitwise left-shift
    // ;; left shift one spot
    // ;; the top item on the stack will be 14 (00001110)
    // (i32.shl 1)
    // ```

    // instructions `i32.clz`, `i32.ctz` and `i32.popcnt` examples:
    //
    // ```
    // ;; load a number onto the stack
    // (i32.imm 8_388_608)      ;; 00000000_10000000_00000000_00000000
    // ;; count leading zeros
    // ;; the top item on the stack will be 8
    // i32.clz
    //
    // ;; load a number onto the stack
    // (i32.imm 8_388_608)      ;; 00000000_10000000_00000000_00000000
    // ;; count trailing zeros
    // ;; the top item on the stack will be 23
    // i32.ctz
    //
    // ;; load a number onto the stack
    // (i32.imm 130)            ;; 10000010
    // ;; count the 1s
    // ;; the top item on the stack will be 2
    // i32.popcnt
    // ```
    i64_and,                    // (operand left:i64 right:i64) -> i64
    i64_or,                     // (operand left:i64 right:i64) -> i64
    i64_xor,                    // (operand left:i64 right:i64) -> i64
    i64_shift_left,             // left shift                   (operand number:i64 move_bits:i32) -> i64       // move_bits = [0,64)
    i64_shift_right_s,          // arithmetic right shift       (operand number:i64 move_bits:i32) -> i64       // move_bits = [0,64)
    i64_shift_right_u,          // logical right shift          (operand number:i64 move_bits:i32) -> i64       // move_bits = [0,64)
    i64_rotate_left,            // left rotate                  (operand number:i64 move_bits:i32) -> i64       // move_bits = [0,64)
    i64_rotate_right,           // right rotate                 (operand number:i64 move_bits:i32) -> i64       // move_bits = [0,64)
    i64_not,                    // (operand number:i64) -> i64
    i64_leading_zeros,          // (operand number:i64) -> i32
    i64_trailing_zeros,         // (operand number:i64) -> i32
    i64_count_ones,             // (operand number:i64) -> i32

    //
    // math
    //
    f32_abs = 0x900,                // (operand number:f32) -> f32
    f32_neg,                        // (operand number:f32) -> f32
    f32_ceil,                       // (operand number:f32) -> f32
    f32_floor,                      // (operand number:f32) -> f32
    f32_round_half_away_from_zero,  // (operand number:f32) -> f32
    // f32_round_half_to_even,      // (operand number:f32) -> f32
    f32_trunc,                  // the integer part of x            (operand number:f32) -> f32
    f32_fract,                  // the fractional part of  x        (operand number:f32) -> f32
    f32_sqrt,                   // sqrt(x)                          (operand number:f32) -> f32
    f32_cbrt,                   // cbrt(x), the cube root of x      (operand number:f32) -> f32
    f32_exp,                    // e^x                              (operand number:f32) -> f32
    f32_exp2,                   // 2^x                              (operand number:f32) -> f32
    f32_ln,                     // log_e(x)                         (operand number:f32) -> f32
    f32_log2,                   // log_2(x)                         (operand number:f32) -> f32
    f32_log10,                  // log_10(x)                        (operand number:f32) -> f32
    f32_sin,                    // (operand number:f32) -> f32
    f32_cos,                    // (operand number:f32) -> f32
    f32_tan,                    // (operand number:f32) -> f32
    f32_asin,                   // (operand number:f32) -> f32
    f32_acos,                   // (operand number:f32) -> f32
    f32_atan,                   // (operand number:f32) -> f32
    f32_pow,                    // left^right                       (operand left:f32 right:f32) -> f32
    f32_log,                    // log_right(left)                  (operand left:f32 right:f32) -> f32

    // examples of 'round_half_away_from_zero':
    // round(2.4) = 2.0
    // round(2.6) = 3.0
    // round(2.5) = 3.0
    // round(-2.5) = -3.0
    //
    // ref:
    // https://en.wikipedia.org/wiki/Rounding#Rounding_half_away_from_zero
    f64_abs,                        // (operand number:f64) -> f64
    f64_neg,                        // (operand number:f64) -> f64
    f64_ceil,                       // (operand number:f64) -> f64
    f64_floor,                      // (operand number:f64) -> f64
    f64_round_half_away_from_zero,  // (operand number:f64) -> f64
    // f64_round_half_to_even,      // (operand number:f64) -> f64
    f64_trunc,                  // (operand number:f64) -> f64
    f64_fract,                  // (operand number:f64) -> f64
    f64_sqrt,                   // (operand number:f64) -> f64
    f64_cbrt,                   // (operand number:f64) -> f64
    f64_exp,                    // (operand number:f64) -> f64
    f64_exp2,                   // (operand number:f64) -> f64
    f64_ln,                     // (operand number:f64) -> f64
    f64_log2,                   // (operand number:f64) -> f64
    f64_log10,                  // (operand number:f64) -> f64
    f64_sin,                    // (operand number:f64) -> f64
    f64_cos,                    // (operand number:f64) -> f64
    f64_tan,                    // (operand number:f64) -> f64
    f64_asin,                   // (operand number:f64) -> f64
    f64_acos,                   // (operand number:f64) -> f64
    f64_atan,                   // (operand number:f64) -> f64
    f64_pow,                    // (operand left:f32 right:f32) -> f64
    f64_log,                    // (operand left:f32 right:f32) -> f64

    //
    // control flow
    //

    // when the instruction 'end' is executed, a stack frame will be removed and
    // the results of the current block or function will be placed on the top of stack.
    //
    end = 0xa00, // finish a block or a function.

    // create a block scope. a block is similar to a function, it also has
    // parameters and results, it shares the type with function, so the 'block'
    // instruction has a parameter called 'type_index'.
    // this instruction leads VM to create a stack frame which is called 'block frame',
    // block frame is similar to 'function frame' except it has no local variables.
    //
    block, // (param type_index:i32, local_list_index:i32)

    // the instruction 'break' is similar to the instruction 'end', it is
    // used for finishing a block or a function.
    // - for a block:
    //   a block stack frame will be removed and jump to the next instruction
    //   that AFTER the instruction 'end'.
    //   the value of the parameter 'next_inst_offset' should be (`addr of next inst after 'end'` - `addr of break`)
    // - for a function:
    //   a function stack frame will be removed and return to the the
    //   instruction next to the instruction 'call'.
    //   the value of the parameter 'next_inst_offset' is ignored.
    //
    // note that this instruction implies the function of instruction 'end'.
    //
    // e.g.
    //
    // ```bytecode
    // 0d0000 block 0           ;; the size of instruction 'block' is 8 bytes
    // 0d0008   nop             ;;
    // 0d0010   break 0 14      ;; the size of instruction 'break' is 8 bytes, (14 = 24 - 10) ---\
    // 0d0018   nop             ;;                                                               |
    // 0d0020   nop             ;;                                                               |
    // 0d0022 end               ;;                                                               |
    // 0d0024 nop               ;; <-- jump to here (the instruction that next to the 'end')-----/
    // ```
    //
    // instruction 'break' not only just finish a block or a function, but also
    // brings the operands out of the block or function, e.g.
    //
    // 0d0000 block 0           ;; assumes the block type is '()->(i32,i32)'
    // 0d0008   i32.imm 11      ;;
    // 0d0016   i32.imm 13      ;;                 | 17                 | -----\ operands '17' and '13' were
    // 0d0024   i32.imm 17      ;; --------------> | 13                 | -----\ taken out of the block frame
    // 0d0032   break 0 14      ;; ---\            | 11                 |      |
    // 0d0040   nop             ;;    |            | [block frame info] |      v
    // 0d0042   nop             ;;    | jump       | ..                 |    | 17                 |
    // 0d0044   nop             ;;    |            | [func frame info]  |    | 13                 |
    // 0d0046   nop             ;;    |            \____________________/    | ..                 |
    // 0d0048 end               ;;    |               the stack layout       | [func frame info]  |
    // 0d0050 nop               ;; <--/ -----------------------------------> \____________________/
    //                                                                        the stack layout
    //
    // the instruction 'break' can cross over multiple block nested.
    // when the parameter 'reversed_index' is 0, it simply finish the current block.
    // when the value is greater than 0, multiple block stack frame will be removed,
    // as well as the operands will be taken out of the block.
    // (the amount of the operands is determined by the 'target block type'.
    //
    // ```bytecode
    // 0d0000 block 0           ;; assumes the block type is '()->(i32,i32,i32)'
    // 0d0008   block 0         ;; assumes the block type is '()->(i32,i32)'
    // 0d0016     block 0       ;; assumes the block type is '()->(i32)'
    // 0d0024       nop
    // 0d0026       break 1 14  ;; (18 = 44 - 26) --------\
    // 0d0034       nop         ;;                        |
    // 0d0036     end           ;;                        |
    // 0d0038     nop           ;;                        |
    // 0d0040   end             ;;                        | carries operands (i32, i32) and
    // 0d0042   nop             ;; <----------------------/ jumps to here
    // 0d0044 end
    // ```

    // note of the block range
    //
    // the value of parameter 'next_inst_offset' should not exceed the block range.
    // the MAX value is the address of the instruction 'end' of the target frame.

    // background:
    //
    // there is a similar instruction in WASM named 'br/break', it is used
    // to make the PC jump to the address of the instruction 'end'.
    // it is more elegant than XiaoXuan instruction 'break', but less efficient
    // because it need to carry the operand twice.
    // after balancing between performance and elegance, XiaoXuan instruction
    // 'break' implies 'end' as well as jumps directly to the next instruction
    // after the instruction 'end'.
    break_, // (param reversed_index:i16, next_inst_offset:i32)

    // the instruction 'recur' lets VM to jump to the instruction next to the instruction 'block', 'block_alt'
    // or the first instruction of the current function,
    // as well as all the operands in the current stack frame will be removed except
    // the operands for the 'target block/function params' are reserved and placed on the top of stack.
    // it is commonly used to construct the 'while/for' structures in general programming languages,
    // it is also used to implement the TCO (tail call optimization, see below section).
    //
    // when the target frame is the function frame itself, the param 'start_inst_offset' is ignore and
    // all local variables will be reset to 0 (except the arguments).
    //
    // note that the value of 'start_inst_offset' is a positive number.
    //
    // 0d0000 block 0           ;; assumes the block type is '()->(i32,i32)'
    // 0d0008   i32.imm 11      ;; <------\
    // 0d0016   i32.imm 13      ;;        |         | 17                 | -----\ operands '17' and '13' were
    // 0d0024   i32.imm 17      ;; ---------------> | 13                 | -----\ taken out of the block frame
    // 0d0032   nop             ;;        |         | 11                 |      |
    // 0d0034   nop             ;;        |         | [block frame info] |      v
    // 0d0036   nop             ;;        |         | ..                 |    | 17                 |
    // 0d0038   nop             ;;  jump  |         | [func frame info]  |    | 13                 |
    // 0d0040   recur 0 14      ;; -------/         \____________________/    | [block frame info] |
    // 0d0048 end               ;;        |            the stack layout       | ..                 |
    // 0d0050 nop               ;;        \---------------------------------> | [func frame info]  |
    //                                                                        \____________________/
    //                                                                           the stack layout

    // the instruction 'recur' can cross over multiple block nested also.
    //
    // ```bytecode
    // 0d0000 block 0           ;; assumes the block type is '()->(i32,i32)'
    // 0d0008   nop             ;; <--------------------------------------------\ carries operands (i32,i32) and
    // 0d0010   block 0         ;; assumes the block type is '()->(i32)'        | jumps to here
    // 0d0018     nop           ;;                                              |
    // 0d0020     recur 1 12    ;; (12 = 20 - 8) -------------------------------/
    // 0d0028     nop
    // 0d0030   end
    // 0d0032 end
    // ```
    //
    // 'start_inst_offset' is the address of the next instruction follows 'block'
    recur, // (param reversed_index:i16, start_inst_offset:i32)

    // create a block scope only if the operand on the top of stack is
    // NOT equals to ZERO (logic TRUE).
    //
    // the value of 'next_inst_offset' should be the address of the next instructions
    // AFTER the instruction 'end'.
    //
    // instruction 'block_nez' is commonly used to construct the 'if' structures
    // in general programming languages.
    //
    // e.g
    //
    // ```c
    // if (i != 0) {
    //     ...
    // }
    // ```
    //
    // ```bytecode
    // 0d0000 block_nez 0 100   ;; -----\
    // ....                     ;;      |
    // 0d0100 end               ;;      |
    // 0d0102 nop               ;; <----/ jump to here when FALSE
    // ```
    //
    // 'block_nez' has NO PARAMS and NO RESULTS, but it can owns local variables.
    //
    block_nez, // (param local_list_index:i32, next_inst_offset:i32)

    // the instruction 'block_alt' is similar to the 'block', it also creates a new block scope
    // as well as a block stack frame.
    // but it jumps to the 'alternative instruction' if the operand on the top of stack is
    // equals to ZERO (logic FALSE).
    //
    // note:
    // 0:i32 and 0:i64 are both treated as logic FALSE and
    // all other i32/i64 non-zero are treated as logic TRUE.
    //
    // so the instruction 'block_alt' means only executes the instructions that follow the instruction 'block_alt'
    // when the logic is TRUE, otherwise, jump to the 'alternative instruction'.
    //
    // e.g.
    //
    // ```c
    // if (i != 0) {
    //     /* the 'then' part */
    // } else {
    //     /* the 'else' part */
    // }
    // ```
    //
    // ```bytecode
    //                          ;; TRUE             | FALSE
    //                          ;; |                | |
    //                          ;; |                | \-->--\
    // 0d0000 block_alt 0 158   ;; V                |       | jump to 0d0158 when FALSE
    // 0d0008 ...               ;; |+               |       |-
    // ;; the 'then' part       ;; |+               |       |-
    // 0d0150 break 0 200       ;; \-->--\+         |       |-
    // 0d0158 ...               ;;       |-         | /--<--/+
    // ;; the 'else' part       ;;       |-         | |+
    // 0d0350 end               ;;       |-         | |+
    // 0d0352 nop               ;; <-----/          | |
    // ```
    //
    // (+ => execute, - => pass)
    //
    // 'block_alt' has NO PARAMS, but it can return values and own local variables.
    //
    block_alt, // (param type_index:i32, local_list_index:i32, alt_inst_offset:i32)

    // a complete 'for' structure is actually combined with instructions 'block', 'block_alt', 'recur', 'break'
    // and 'break_nez', e.g.
    //
    // ```rust
    // let i = loop {
    //   ...
    //   if ... break 100;
    //   ...
    // }
    // ```
    //
    // the equivalent bytecodes are:
    //
    // ```bytecode
    // 0d0000 block 0
    // 0d0008   ...             ;; <-------------\
    //          ...             ;;               |
    // 0d0100   block_alt 0 28  ;; ----\         |
    // 0d0112     i32.imm 100   ;;     |         |
    // 0d0120     break 1 88    ;; ----|----\    |
    // 0d0128   end             ;; <---/    |    |
    //          ...             ;;          |    |
    // 0d0200   recur 0 192     ;; ---------|----/
    // 0d0208 end               ;;          |
    // 0d0210 ...               ;; <--------/
    // ```
    //
    // the code above can be optimized by instruction 'break_nez', e.g.
    //
    // ```bytecode
    // 0d0000 block 0
    // 0d0008   ...             ;; <-------------\
    //          ...             ;;               |
    // 0d0100   i32.imm 100     ;;               |
    //          ...             ;;               |
    // 0d0120   break_nez 0 88  ;; ---------\    |
    // 0d0128   drop            ;;          |    |
    //          ...             ;;          |    |
    // 0d0200   recur 0 192     ;; ---------|----/
    // 0d0208 end               ;;          |
    // 0d0210 ...               ;; <--------/
    // ```
    break_nez, // (param reversed_index:i16, next_inst_offset:i32)

    // when the target frame is the function frame itself, the param 'start_inst_offset' is ignore and
    // all local variables will be reset to 0.
    //
    // instruction 'recur_nez' is used to implement the TCO (tail call optimization).
    //
    // consider the following function:
    //
    // ```rust
    // /* calculate '3+2+1', the result should be '6' */
    //
    // let s = accumulate(0, 3);   //
    //
    // fn accumulate(sum: i32, number: i32) -> i32 {    // /-----\
    //     let new_sum = sum + number;                  // | +   |
    //     if number == 0 {                             // | --> | branch then
    //         new_sum                                  // | <-- | return 0
    //     } else {                                     // | --> | branch else
    //         accumulate(new_sum, number - 1)          // | <-- | return 1
    //     }                                            // |     |
    // }                                                // \-----/
    // ```
    //
    // when invoke function call 'accumulate(0, 3)', the calling path is as follows:
    //
    // (0,3)--\
    //        |    /-----\     (3,2)  /-----\     (5,1)  /-----\     (6,0)  /-----\
    //        \--> | +   |   /------> | +   |   /------> | +   |   /------> | +   |
    //             |=====|   |        |=====|   |        |=====|   |        |=====|
    //             | --> |   |        | --> |   |        | --> |   |        | --> | -----\
    //             | <-- |   |        | <-- |   |        | <-- |   |  /---- | <-- | <----/
    //             |=====|   |        |=====|   |        |=====|   |  |     |=====|
    //             | --> | --/        | --> | --/        | --> | --/  |     | --> |
    //        /--- | <-- | <--------- | <-- | <--------- | <-- | <----/     | <-- |
    //        |    \-----/            \-----/            \-----/            \-----/
    //   6 <--/
    //
    //
    // the function 'accumulate' is invoked 4 times and 4 stack frames are created.
    // since there is no other operation after statement 'accumulate(new_sum, number - 1)', and
    // only return a value afterwards, so the calling path can be simplified as follows:
    //
    // (0,3)--\
    //        |    /-----\     (3,2)  /-----\     (5,1)  /-----\     (6,0)  /-----\
    //        \--> | +   |   /------> | +   |   /------> | +   |   /------> | +   |
    //             |=====|   |        |=====|   |        |=====|   |        |=====|
    //             | --> |   |        | --> |   |        | --> |   |        | --> | ---\
    //             | <-- |   |        | <-- |   |        | <-- |   |    /-- | <-- | <--/
    //             |=====|   |        |=====|   |        |=====|   |    |   |=====|
    //             | --> | --/        | --> | --/        | --> | --/    |   | --> |
    //             | <-- |            | <-- |            | <-- |        |   | <-- |
    //             \-----/            \-----/            \-----/        |   \-----/
    //   6 <------------------------------------------------------------/
    //
    // now we can introduce the instruction 'recur/recur_nez', which lets the VM execute
    // (rather than 'call') the function again from beginning with new arguments,
    // so that we can optimize the calling path:
    //
    // (0,3)--\
    //        |    /-----\
    //        \--> | +   | <----\  <----\  <----\
    //             |=====|      |       |       |
    //             | --> | --\  |       |       |
    //        /--- | <-- | <-/  |       |       |
    //        |    |=====|      |       |       |
    //        |    | --> | -----/  -----/  -----/
    //        |    |     | (3,2)  (5,1)  (6,0)
    //        |    \-----/
    //   6 <--/
    //
    // shown above is TCO (tail call optimization), this optimization saves us from
    // creating and destroying calling stack frames multiple times, which saves resources
    // to improve program efficiency.
    //
    // an important prerequisite for TCO is that the 'recur call' statement
    // (in genernal programming language) must be the last operation of the function,
    // or the last operation in the function branch, otherwise the logical error will occur. e.g.
    //
    // ```rust
    // fn factorial(number: i32) -> i32 {
    //     if number == 0 {
    //         1
    //     } else {
    //         number * factorial(number - 1)
    //     }
    // }
    // ```
    //
    // the statement 'number * factorial(number - 1)' exands as follows:
    //
    // ```rust
    // let i = number;
    // let j = factorial(number - 1);
    // i * j
    // ```
    //
    // obviously the function call statement 'factorial(number - 1)' is neither the last operation
    // of the function nor the last operation of the function branch. the last operation
    // is 'i * j', so this code cann't apply TCO.
    //
    // of course we can modify this function so that it can apply TCO, e.g.
    //
    // ```rust
    // /* calculate '5*4*3*2*1', the result should be '120' */
    //
    // let s = factorial_tco(1, 5);
    //
    // fn factorial_tco(sum: i32, number: i32) -> i32 {
    //     if number == 0 {
    //         sum
    //     } else {
    //         let new_sum = sum * number;
    //         factorial_tco(new_sum, number - 1)
    //     }
    // }
    // ```
    recur_nez, // (param reversed_index:i16, start_inst_offset:i32)

    // table of control flow structures and control flow instructions:
    //
    // ## branch
    //
    // | structure         | assembly          | instruction(s)     |
    // |-------------------|-------------------|--------------------|
    // |                   |                   | ..a..              |
    // | if ..a.. {        | (when (a)         | block_nez -\       |
    // |    ..b..          |       (b)         |   ..b..    |       |
    // | }                 | )                 | end        |       |
    // |                   |                   | ...    <---/       |
    // |-------------------|-------------------|--------------------|
    // |                   |                   | ..a..              |
    // | if ..a.. {        | (if (a)           | block_alt ---\     |
    // |    ..b..          |     (b)           |   ..b..      |     |
    // | } else {          |     (c)           |   break 0  --|-\   |
    // |    ..c..          | )                 |   ..c..  <---/ |   |
    // | }                 |                   | end            |   |
    // |                   |                   | ...      <-----/   |
    // |-------------------|-------------------|--------------------|
    // |                   |                   | ..a..              |
    // | if ..a.. {        | (if (a)           | block_alt ---\     |
    // |    ..b..          |     (b)           |   ..b..      |     |
    // | } else if ..c.. { |     (if (c)       |   break 0 ---|---\ |
    // |    ..d..          |         (d)       |   ..c..  <---/   | |
    // | } else {          |         (e)       |   block_alt --\  | |
    // |    ..e..          |     )             |     ..d..     |  | |
    // | }                 | )                 |     break 0 --|-\| |
    // |                   |                   |     ..e..  <--/ || |
    // |                   |                   |   end           || |
    // |                   |                   | end        <----/| |
    // |                   |                   | ...        <-----/ |
    // |                   |                   |                    |
    // |                   | ----------------- | ------------------ |
    // |                   |                   |                    |
    // |                   | (branch           | block              | // note:
    // |                   |   (case (a) (b))  |   ..a..            | //
    // |                   |   (case (c) (d))  |   block_nez -\     | // 'block_nez + x_imm/x_loadx + break N + end'
    // |                   |   (default (e))   |     ..b..    |     | // can be optimized as:
    // |                   | )                 |     break 1 -|--\  | // 'x_imm/x_loadx + swap + break_nez N-1 + drop'
    // |                   |                   |   end        |  |  |
    // |                   |                   |   ..c..  <---/  |  |
    // |                   |                   |   block_nez -\  |  |
    // |                   |                   |     ..d..    |  |  |
    // |                   |                   |     break 1 -|--|  |
    // |                   |                   |   end        |  |  |
    // |                   |                   |   ..e..  <---/  |  |
    // |                   |                   | end             |  |
    // |                   |                   | ...        <----/  |
    // |-------------------|-------------------|--------------------|
    //
    // ## loop
    //
    // | structure         | assembly          | instructions(s)    |
    // |-------------------|-------------------|--------------------|
    // | loop {            | (for (code        | block              |
    // |    ...            |   ...             |   ...   <--\       |
    // | }                 |   (recur ...)     |   recur 0 -/       |
    // |                   | ))                | end                |
    // |-------------------|-------------------|--------------------|
    // | while ..a.. {     |                   | block              |
    // |    ...            |                   |   ..a..   <----\   |
    // | }                 |                   |   break_nez 0 -|-\ |
    // |                   |                   |   ...          | | |
    // | (or for...)       |                   |   recur 0 -----/ | |
    // |                   |                   | end              | |
    // |                   |                   | ...        <-----/ |
    // |                   |                   |                    |
    // |                   | ----------------- | ------------------ |
    // |                   |                   |                    |
    // |                   | (for (code        | block              |
    // |                   |   (when (a)       |   ..a..    <---\   |
    // |                   |     (code ...     |   block_nez    |   |
    // |                   |       (recur ...) |     ...        |   |
    // |                   |     )             |     recur 1 ---/   |
    // |                   |   )               |   end              |
    // |                   | ))                | end                |
    // |                   |                   |                    |
    // |                   |                   |                    |
    // |-------------------|-------------------|--------------------|
    // | do {              |                   | block              |
    // |    ...            |                   |   ...      <---\   |
    // | }while(..a..)     |                   |   ..a..        |   |
    // |                   |                   |   recur_nez 0 -/   |
    // |                   |                   | end                |
    // |                   |                   |                    |
    // |                   | ----------------- | ------------------ |
    // |                   |                   |                    |
    // |                   | (for (code        | block              |
    // |                   |   ...             |   ...      <---\   |
    // |                   |   (when (a)       |   ..a..        |   |
    // |                   |     (recur ...)   |   block_nez    |   |
    // |                   |   )               |     recur 1 ---/   |
    // |                   | ))                |   end              |
    // |                   |                   | end                |
    // |                   |                   |                    |
    // |                   |                   |                    |
    // |-------------------|-------------------|--------------------|
    //
    //
    // ## TCO
    //
    // | structure         | assembly          | instructions(s)    |
    // |-------------------|-------------------|--------------------|
    // | func foo {        | (function         | -- func begin --   |
    // |    ...            |   (code ...       |   ...   <-------\  |
    // |    if ..a.. {     |     (when (a)     |   ..a..         |  |
    // |      foo()        |       (tcall ...) |   block_nez --\ |  |
    // |    }              |     )             |     recur 1 --|-/  |
    // | }                 |   )               |   end         |    |
    // |                   | )                 | end      <----/    |
    // |                   |                   |                    |
    // |                   | ----------------- | ------------------ |
    // |                   |                   |                    | // note:
    // |                   |                   | -- func begin --   | //
    // |                   |                   |   ...   <-------\  | // 'block_nez + recur 1/break 1 + end'
    // |                   |                   |   ..a..         |  | // can be optimized as:
    // |                   |                   |   recur_nez 0 --/  | // 'recur_nez 0' and 'break_nez 0'
    // |                   |                   | end                |
    // |                   |                   |                    |
    // |-------------------|-------------------|--------------------|
    // | func foo {        | (function (code   | -- func begin --   |
    // |    if ..a.. {     |   (if (a)         |   ..a.. <------\   |
    // |       ..b..       |     (b)           |   block_alt -\ |   |
    // |    } else {       |     (code         |     ..b..    | |   |
    // |       ..c..       |       ..c..       |     break 0 -|-|-\ |
    // |       foo()       |       (tcall ...) |     ..c.. <--/ | | |
    // |    }              |     )             |     recur 1 ---/ | |
    // | }                 |   )               |   end            | |
    // |                   | ))                | end         <----/ |
    // |                   |                   |                    |
    // |-------------------|-------------------|--------------------|
    //
    //
    // ## break
    //
    // | structure         | assembly          | instructions(s)    |
    // |-------------------|-------------------|--------------------|
    // |                   |                   |                    |
    // | loop {            | (for (code        | block              |
    // |    ...            |    ...            |   ...        <---\ |
    // |    if ..a.. {     |    (when (a)      |   ..a..          | |
    // |      break;       |                   |   block_nez      | |
    // |    }              |       (break ...) |     break 1 ---\ | |
    // |    ...            |    )              |   end          | | |
    // | }                 |    ...            |   ...          | | |
    // |                   |    (recur ...)    |   recur 0  ----|-/ |
    // |                   | ))                | end            |   |
    // |                   |                   | ...      <-----/   |
    // |                   |                   |                    |
    // |-------------------|-------------------|--------------------|
    // | func foo {        | (function (code   | -- func begin --   |
    // |    ...            |   ...             |   ...              |
    // |    if ..a.. {     |   (when (a)       |   ..a..            |
    // |      return ...;  |                   |   block_nez        |
    // |    }              |       (return ..) |     break 1  ---\  |
    // |    ...            |   )               |   end           |  |
    // | }                 |   ...             |   ...           |  |
    // |                   | ))                | end         <---/  |
    // |                   |                   |                    |
    // |                   |                   |                    |
    // |-------------------|-------------------|--------------------|

    //
    // function call
    //
    call = 0xb00,               // general function call            (param function_public_index:i32) -> (...)
    dyncall,                    // dynamic function call            (operand function_public_index:i32) -> (...)

    // call a function which is specified at runtime.
    //
    // when a general or anonymous function is passed to another function as a parameter,
    // it passs a pointer of a struct 'closure_function_item' actually:
    //
    // closure_function_item {
    //     [ref_count],
    //     function_public_index:i32,
    //     captured_data:i64_ref
    // }
    //
    // "captured_data" is a structure that contains the data that function captured,
    // for example, an anonymous function captured an i32 number and a string, then
    // the 'captured data struct' is:
    //
    // captured_data_001: {
    //     f0:i32,
    //     f1:i64_ref
    // }

    // the target of dynamic function includes 'anonymous function' and 'regular function',
    // when the target is anonymous function, an additional parameter is appended to
    // the anonymous function automatically when it is compiled to assembly, e.g.
    //
    // `let a = fn (i32 a, i32 b) {...}`
    //
    // will be compiled to:
    //
    // `let a = fn (i32 a, i32 b, addr captured_data) {...}`
    //
    // ```text
    //                              /--> function_public_index --> fn (a, b, captured_data) {...}
    //                         /--->|
    //                         |    \--> captured_data
    //                         |
    // let a = filter(list, predicate)
    // ```
    //
    // when the target is a regular function, the compiler generates an anonymous function
    // for wrapping the general function, e.g.
    //
    // ```text
    //                              /--> function_public_index --> fn wrapper (a, b, captured_data) --> fn regular (a, b)
    //                         /--->|
    //                         |    \--> captured_data = 0
    //                         |
    // let a = filter(list, predicate)
    // ```

    // note:
    // 'function public index' includes the imported functions, it equals to
    // 'amount of imported functions' + 'function internal index'

    envcall,                    // environment call                 (param env_func_num:i32) -> (...)

    // `(operand args..., syscall_num:i32, params_count: i32)` -> (return_value:i64, error_no:i32)
    //
    // the syscall arguments should be pushed on the stack first, e.g.
    //
    // | params_count   |
    // | syscall_num    |
    // | arg6           |
    // | arg5           |
    // | arg4           |
    // | arg3           |
    // | arg2           |                  | error no       |
    // | arg1           |     return -->   | return value   |
    // | ...            |                  | ...            |
    // \----------------/ <-- stack start  \----------------/
    //
    // when a syscall complete, the return value is store into the 'rax' register,
    // if the operation fails, the value is a negative value (rax < 0).
    // there is no 'errno' if invoke syscall by assembly directly.
    syscall,

    // external function call
    // `(param external_function_index:i32) -> void/i32/i64/f32/f64`
    //
    // note that both 'syscall' and 'extcall' are optional, they may be
    // unavailable in some environment.
    // the supported feature list can be obtained through the instruction 'envcall' with code 'features'.
    extcall,

    //
    // host
    //
    panic = 0xc00,              // terminate VM
    unreachable,                // unreachable code     (param code:u32) -> ()
    debug,                      // for VM debug         (param code:u32) -> ()

    // get the host address of memory
    //
    // it is not safe to access data using (host's) memory addresses,
    // but this is necessary to talk to external libraries (e.g. C library)
    //
    // about memory access safe
    //
    // |                    |  by indice   | by mem allocator | by host address |
    // |--------------------|--------------|------------------|-----------------|
    // | local vars         | safe         | -                | unsafe          |
    // |--------------------|--------------|------------------|-----------------|
    // | read-only data     |              |                  |                 |
    // | read-write data    | safe         | -                | unsafe          |
    // | uninitilized data  |              |                  |                 |
    // |--------------------|--------------|------------------|-----------------|
    // | heap               | unsafe       | controllable     | unsafe          |
    // |--------------------|--------------|------------------|-----------------|
    host_addr_local,            // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16) -> i64/i32
                                //
                                // note that the host address only valid in the current function and
                                // in its sub-functions. when a function exited, the function stack frame
                                // will be destroied (or modified), as well as the local variables.
                                //
    host_addr_local_long,       // (param reversed_index:i16 local_variable_index:i32)      (operand offset_bytes:i32) -> i64/i32
    host_addr_data,             // (param offset_bytes:i16 data_public_index:i32) -> i64/i32
    host_addr_data_long,        // (param data_public_index:i32)                            (operand offset_bytes:i32) -> i64/i32
    host_addr_heap,             // (param offset_bytes:i16)                                 (operand heap_addr:i64) -> i64/i32

    host_addr_function,         // create a new host function and map it to a VM function.
                                // this host function named 'bridge funcion'
                                //
                                // `(param function_public_index:i32) -> i64/i32`
                                //
                                // return the existing bridge function if the bridge function corresponding
                                // to the specified VM function has already been created.

    // it's commonly used for creating a callback function pointer for external C function.
    //
    // note:
    // - a bridge function (host function) will be created when `create_host_function` is executed,
    //   as well as the specified VM function will be appended to the "host function bridge table" to
    //   prevent duplicate creation.
    // - a bridge function is refered to a (module idx, function idx) tuple.
    // - the bridge function is created via JIT codegen.
    // - when the external C function calls the bridge function, a new thread is created.
    //
    // when the XiaoXUan VM is embed into a C or Rust application as a library, the C or Rust application
    // can call the VM function through the bridge function as if it calls a native function.
    //
    // call bridge functon from Rust application example:
    //
    // ref:
    // https://doc.rust-lang.org/nomicon/ffi.html
    // https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html
    // https://doc.rust-lang.org/stable/reference/items/functions.html
    //
    // ```rust
    // fn main() {
    //     let func_ptr = ... (pointer of the bridge function)
    //     let func_addr = ... (virtual memory address of the bridge function)
    //
    //     /** mock pointer and address
    //     let func_ptr = cb_func as *const extern "C" fn(usize, usize);
    //     let func_ptr = cb_func as *const u8;
    //     let func_addr = func_ptr as usize;
    //     */
    //
    //     println!("{:p}", func_ptr);
    //     println!("0x{:x}", func_addr);
    //
    //     let func_from_ptr: fn(usize, usize) = unsafe { std::mem::transmute(func_ptr) };
    //     (func_from_ptr)(11, 13);
    //
    //     let ptr_from_addr = func_addr as *const ();
    //     let func_from_addr: fn(usize, usize) = unsafe { std::mem::transmute(ptr_from_addr) };
    //     (func_from_addr)(17, 19);
    // }
    //
    // #[no_mangle]
    // pub extern "C" fn cb_func(a1: usize, a2: usize) {
    //     println!("numbers: {},{}", a1, a2);
    // }
    // ```
    //
    // call bridge functon from C application example:
    //
    // ```c
    // int main(void)
    // {
    //     void *func_ptr = ...
    //     int (*func_from_ptr)(int, int) = (int (*)(int, int))func_ptr;
    //     printf("1+2=%d\n", (*func_from_ptr)(1, 2));
    //     exit(EXIT_SUCCESS);
    // }
    // ```

    host_copy_from_heap,        // copy data from VM heap to host memory
                                // (operand dst_pointer:i64 src_offset:i64 length_in_bytes:i64) -> ()
                                //
    host_copy_to_heap,          // copy data from host memory to VM heap
                                // (operand dst_offset:i64 src_pointer:i64 length_in_bytes:i64) -> ()

    //
    // SIMD/Vectorization
    //

    // TODO

    // ref:
    // - https://github.com/rust-lang/portable-simd
    // - https://doc.rust-lang.org/std/simd/index.html
    // - https://github.com/rust-lang/packed_simd
    // - https://github.com/rust-lang/rfcs/blob/master/text/2325-stable-simd.md
}

pub const MAX_OPCODE_NUMBER: usize = 0xd00;
