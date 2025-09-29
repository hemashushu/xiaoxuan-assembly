// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use crate::instruction::Instruction;

/* *
 *
 * Instruction Format:
 * - legacy prefix: 0-4 bytes (group 1-4)
 * - REX: 0,1 byte
 * - Opcode: 1,2,3 bytes (3 bytes = 0F + 2 bytes)
 * - ModRM: 0,1 byte (Mod: 2-bit, Reg: 3-bit, R/M: 3-bit)
 * - SIB: 0,1 byte (Scale: 2-bit, Index: 3-bit, Base: 3-bit)
 * - Displacement: 0,1,2,4 bytes
 * - Immediate: 0,1,2,4 bytes (8 bytes is only supported in `MOV r64, imm64`)
 *
 * References:
 *
 * - Intel Software Developer's Manual
 *   Volume 2, Chapter 2.1 INSTRUCTION FORMAT FOR PROTECTED MODE, REAL-ADDRESS MODE, AND VIRTUAL-8086 MODE
 *   Volume 2, Appendix B INSTRUCTION FORMATS AND ENCODINGS
 *   Volume 2, Section 3.1.1.1 Opcode Column in the Instruction Summary Table (Instructions without VEX Prefix)
 *   https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html
 *
 * - X86 Opcode and Instruction Reference
 *   http://ref.x86asm.net/index.html
 * - x86 and amd64 instruction reference
 *   https://www.felixcloutier.com/x86/
 * - A Beginners’ Guide to x86-64 Instruction Encoding
 *   https://www.systutorials.com/beginners-guide-x86-64-instruction-encoding/
 * - X86-64 Instruction Encoding (OSDev)
 *   https://wiki.osdev.org/X86-64_Instruction_Encoding
 * - Encoding Real x86 Instructions
 *   https://www.c-jump.com/CIS77/CPU/x86/lecture.html
 *
 *
 */
pub fn encode(
    instruction: &Instruction,

    // current address (for the new generated code)
    current_address: u64,

    // lable address list
    lable_address_list: &Vec<(&str, u64)>,
) -> Vec<u8> {
    // Encoding logic will be implemented here
    todo!()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_encode_mov() {

        // | Opcode        | Instruction    | Op/En | 64-Bit Mode | Compat/Leg Mode | Description        |
        // | ---           |  ---           |  ---  |  ---        |  ---            |  ---               |
        // | 88 /r         | MOV r/m8, r8   | MR    | Valid       | Valid           | Move r8 to r/m8.   |
        // | REX + 88 /r   | MOV r/m8, r8   | MR    | Valid       | N.E.            | Move r8 to r/m8.   |
        // | 89 /r         | MOV r/m16, r16 | MR    | Valid       | Valid           | Move r16 to r/m16. |
        // | 89 /r         | MOV r/m32, r32 | MR    | Valid       | Valid           | Move r32 to r/m32. |
        // | REX.W + 89 /r | MOV r/m64, r64 | MR    | Valid       | N.E.            | Move r64 to r/m64. |

        // Instruction Operand Encoding
        //
        // | Op/En | Operand 1     | Operand 2     | Operand 3 | Operand 4 |
        // | ---   |  ---          |  ---          |  ---      |  ---      |
        // | MR    | ModRM:r/m (w) | ModRM:reg (r) | N/A       | N/A       |

        // Test
        //
        // mov rax, rcx ->    48 89 c8 (REX 48 = 0100 1000, W=1, R=0, X=0, B=0) (ModRM byte c8 = 11 001 000, mod=11, reg=001, r/m=000)
        // mov rdx, rbx ->    48 89 da (REX 48 = 0100 1000, W=1, R=0, X=0, B=0) (ModRM byte d3 = 11 011 010, mod=11, reg=011, r/m=010)
        // mov eax, ecx ->       89 c8
        // mov ax, cx   -> 66    89 c8
        // mov al, cl   ->       88 c8

        // todo

        // Test: extended registers (R8-R15) access
        //
        // mov r9, rcx  ->    49 89 c9 (REX 49 = 0100 1001, W=1, R=0, X=0, B=1) (ModRM byte c9 = 11 001 001, mod=11, reg=001, r/m=001)
        // mov r9d, ecx ->    41 89 c9 (REX 41 = 0100 0001, W=0, R=0, X=0, B=1)
        // mov r9w, cx  -> 66 41 89 c9
        // mov r9b, cl  ->    41 88 c9
        // mov ecx, r9d ->    44 89 c9 (REX 44 = 0100 0100, W=0, R=1, X=0, B=0) (ModRM byte c9 = 11 001 001, mod=11, reg=001, r/m=001)

        // todo

        // Test: extended register but 16-bit operand size
        //
        // mov cx, r9w  -> 66 44 89 c9

        // todo
    }

    #[test]
    fn test_encode_mov_memory() {
        // x86-64 effective address
        // ------------------------
        //
        // EA = Base + (Index * Scale) + Displacement
        //
        // Base: rax, rcx, rdx, rbx, rsp, rbp, rsi, rdi, r8-r15
        // Index: rax, rcx, rdx, rbx, (can not rsp/esp), rbp, rsi, rdi, r8-r15
        // Scale: 1, 2, 4, 8
        // Displacement: 0, 1, 2, 4 bytes
        //
        // e.g.
        //
        // - [0xabcd]: Displacement only, base = 0
        // - [ecx]: Base
        // - [ecx+0xabcd]: Base + Displacement
        // - [ecx+edx]: Base + Index
        // - [ecx+edx*4]: Base + Index×Scale
        // - [ecx+edx*4+0xabcd]: Base + Index×Scale + Displacement
        // - fs:[...]: with segment override prefix
        //
        // Other forms:
        //
        // - [edx*4]: Index only, base = 0, scale = 4
        // - [edx*4+123]: Index + Displacement, base = 0, scale = 4
        //
        // Frequently used forms:
        //
        // - [r]: pointer to heap memory
        // - [r +/- disp]: struct field or array element access
        // - [rbp +/- disp]: local variable access in stack frame
        // - [rip +/- disp]: access global/static variable or function address (RIP-relative addressing)

        // Note: the operand size keyword
        // --------------------------------------------------------------
        // `mov rax, [rbx + 0x10]` is equivalent to `mov rax, qword [rbx + 0x10]`,
        // the operand size is implied by the register size.
        //
        // keyword `qword`, `dword`, `word`, and `byte` can only applied to memory operands,
        // do not apply to register or immediate operands.
        //
        // in `mov dword [rax], 0x12345678`, the keyword `dword` is required to specify the operand size,
        // because the register size is not specified.
        //
        // `mov dword [rax], 0x12345678` and `mov dword ptr [rax], 0x12345678` are equivalent.
        // `ptr` is optional and can be omitted.
        // --------------------------------------------------------------

        // | Opcode        | Instruction        | Op/En | 64-Bit Mode | Compat/Leg Mode | Description        |
        // | ---           |  ---               |  ---  |  ---        |  ---            |  ---               |
        // | 8A /r         | MOV r8, r/m8       | RM    | Valid       | Valid           | Move r/m8 to r8.   |
        // | REX + 8A /r   | MOV r8^1^, r/m8^1^ | RM    | Valid       | N.E.            | Move r/m8 to r8.   |
        // | 8B /r         | MOV r16, r/m16     | RM    | Valid       | Valid           | Move r/m16 to r16. |
        // | 8B /r         | MOV r32, r/m32     | RM    | Valid       | Valid           | Move r/m32 to r32. |
        // | REX.W + 8B /r | MOV r64, r/m64     | RM    | Valid       | N.E.            | Move r/m64 to r64. |

        // Instruction Operand Encoding
        //
        // | Op/En | Operand 1     | Operand 2     | Operand 3 | Operand 4 |
        // | ---   |  ---          |  ---          |  ---      |  ---      |
        // | RM    | ModRM:reg (w) | ModRM:r/m (r) | N/A       | N/A       |

        // Test: memory effective address by base register
        //
        //
        // mov rax, [rbx] ->    48 8b 03 (REX 48 = 0100 1000, W=1, R=0, X=0, B=0) (ModRM byte 03 = 00 000 011, mod=00, reg=000, r/m=011)
        // mov eax, [ebx] -> 67    8b 03 (encodes with prefix 67h, forcing 32-bit compatibility addressing, limited to low 4 GiB)
        // mov ax, [bx]   -> invalid 64-bit effective address in 64-bit mode
        // mov al, [bl]   -> invalid 64-bit effective address in 64-bit mode
        //
        // mov eax, [rbx] ->    8b 03
        // mov ax, [rbx]  -> 66 8b 03
        // mov al, [rbx]  ->    8a 03

        // todo

        // mov [rbx], rax ->    48 89 03 (REX 48 = 0100 1000, W=1, R=0, X=0, B=0) (ModRM byte 03 = 00 000 011, mod=00, reg=000, r/m=011)
        // mov [ebx], eax -> 67    89 03 (encodes with prefix 67h, forcing 32-bit compatibility addressing, limited to low 4 GiB)
        // mov [bx], ax   -> invalid 64-bit effective address in 64-bit mode
        // mov [bl], al   -> invalid 64-bit effective address in 64-bit mode

        // todo

        // Test: memory with segment override prefix
        //
        // mov rax, gs:[rbx] -> 65 48 8b 03 (segment override prefix 65h for GS)
        // mov rax, fs:[rbx] -> 64 48 8b 03 (segment override prefix 64h for FS)

        // todo
    }

    #[test]
    fn test_encode_mov_memory_with_base_and_displacement() {

        // ModRM.mod field
        //
        // | Effective Address  | Mod | R/M |
        // | ------------------ | --- | --- |
        // | [EAX]              | 00  | 000 |
        // | [ECX]              |     | 001 |
        // | [EDX]              |     | 010 |
        // | [EBX]              |     | 011 |
        // | [--][--]           |     | 100 |
        // | disp32             |     | 101 |
        // | [ESI]              |     | 110 |
        // | [EDI]              |     | 111 |
        // | ------------------ | --- | --- |
        // | [EAX]+disp8        | 01  | 000 |
        // | [ECX]+disp8        |     | 001 |
        // | [EDX]+disp8        |     | 010 |
        // | [EBX]+disp8        |     | 011 |
        // | [--][--]+disp8     |     | 100 |
        // | [EBP]+disp8        |     | 101 |
        // | [ESI]+disp8        |     | 110 |
        // | [EDI]+disp8        |     | 111 |
        // | ------------------ | --- | --- |
        // | [EAX]+disp32       | 10  | 000 |
        // | [ECX]+disp32       |     | 001 |
        // | [EDX]+disp32       |     | 010 |
        // | [EBX]+disp32       |     | 011 |
        // | [--][--]+disp32    |     | 100 |
        // | [EBP]+disp32       |     | 101 |
        // | [ESI]+disp32       |     | 110 |
        // | [EDI]+disp32       |     | 111 |
        // | ------------------ | --- | --- |
        // | EAX/AX/AL/MM0/XMM0 | 11  | 000 |
        // | ECX/CX/CL/MM1/XMM1 |     | 001 |
        // | EDX/DX/DL/MM2/XMM2 |     | 010 |
        // | EBX/BX/BL/MM3/XMM3 |     | 011 |
        // | ESP/SP/AH/MM4/XMM4 |     | 100 |
        // | EBP/BP/CH/MM5/XMM5 |     | 101 |
        // | ESI/SI/DH/MM6/XMM6 |     | 110 |
        // | EDI/DI/BH/MM7/XMM7 |     | 111 |
        // | ------------------ | --- | --- |
        //
        // Note:
        // - When Mod=00 and R/M=101, the addressing mode is disp32, or RIP+disp32 in 64-bit mode (no base register).
        // - When Mod=11, the addressing mode is register to register (no memory).
        // - R/M=100 indicates that a SIB byte follows the ModRM byte.
        //
        // References:
        // - Volume 2, Section 2.2 IA-32e Mode
        //   Table 2-2. 32-Bit Addressing Forms with the ModR/M Byte

        // Test: memory effective address by base register + displacement
        //
        // mov rax, [rbx + 0x10]        -> 48 8b 43 10 (ModRM byte 43 = 01 000 011, mod=01, reg=000, r/m=011) (displacement 10h = 16)
        // mov [rbx + 0x10], rax        -> 48 89 43 10
        // mov rax, [rbx + 0x12345678]  -> 48 8b 83 78563412 (ModRM byte 83 = 10 000 011, mod=10, reg=000, r/m=011) (displacement 78563412h)
        //
        // NOTE: disp32 is an signed i32, the max value is 0x7fffffff
        //
        // mov rax, [rbx + 0x1234]      -> 48 8b 83 34120000 (displacement 34120000h)
        //
        // mov r8, [rbp]                -> 4c 8b 45 00 (ModRM byte 45 = 01 000 101, mod=01, reg=000, r/m=101) (displacement 00h = 0)
        // mov r8, [rbp + 0x10]         -> 4c 8b 45 10 (displacement 10h = 16)
        // mov r8, [rbp + 0x1234]       -> 4c 8b 85 34120000 (displacement 34120000h)

        // todo

        // Test: move from memory to register
        //
        // data:
        // num1 dw 0x1234
        // num2 dw 0x5678
        // num3 dw 0x9abc
        //
        // mov eax, [num1]     -> 8b 04 25 00000000 (ModRM byte 04 = 00 000 100, mod=00, reg=000, r/m=100) (SIB byte 25 = 00 100 101, scale=00, index=100(none), base=101(disp32)) (displacement 00000000h = address of num1)
        // mov eax, [num2]     -> 8b 04 25 02000000 (displacement 02000000h = address of num2)
        // mov eax, [num3]     -> 8b 04 25 04000000 (displacement 04000000h = address of num3)

        // todo

        // Test: RIP-relative addressing
        //
        // mov eax, [rel num1] -> 8b 05 f4ffffff (ModRM byte 05 = 00 000 101, mod=00, reg=000, r/m=101) (displacement f4ffffffh = -12, address of num1 - next instruction address)
        // mov eax, [rel num2] -> 8b 05 f6ffffff (displacement f6ffffffh = -10)
        // mov eax, [rel num3] -> 8b 05 f8ffffff (displacement f8ffffffh = -8)

        // todo

        // Test: RIP-relative addressing, move from register to memory
        //
        // mov dword [rel num1], eax -> 89 05 f4ffffff
        // mov dword [rel num2], eax -> 89 05 f6ffffff
        // mov dword [rel num3], eax -> 89 05 f8ffffff

        // todo
    }

    #[test]
    fn test_encode_mov_memory_with_base_and_index_and_scale() {

        // SIB byte
        //
        // `( Scale 2-bit | Index 3-bit | Base 3-bit )`
        //
        // Scale:
        // 00 = *1
        // 01 = *2
        // 10 = *4
        // 11 = *8
        //
        // Index:
        // 000 = EAX
        // 001 = ECX
        // 010 = EDX
        // 011 = EBX
        // 100 = none (no index)
        // 101 = EBP
        // 110 = ESI
        // 111 = EDI
        //
        // Base:
        // 000 = EAX
        // 001 = ECX
        // 010 = EDX
        // 011 = EBX
        // 100 = ESP
        // 101 = EBP/disp32 (disp32 if Mod=00, EBP if Mod!=00)
        // 110 = ESI
        // 111 = EDI
        //
        // References:
        // - Volume 2, Section 2.2 IA-32e Mode
        //   Table 2-3. 32-Bit Addressing Forms with the SIB Byte

        // mov rax, [rcx + rsi*1]           -> 48 8b 04 31 (ModRM byte 04 = 00 000 100, mod=00, reg=000, r/m=100) (SIB byte 31 = 00 110 001, scale=00(*1), index=110(rsi), base=001(rcx))
        // mov rax, [rcx + rsi*2]           -> 48 8b 04 71 (SIB byte 71 = 01 110 001, scale=01(*2))
        // mov rax, [rcx + rsi*4 + 0x10]    -> 48 8b 44 b1 10 (SIB byte b1 = 10 110 001, scale=10(*4)) (displacement 10h = 16)
        // mov rax, [rcx + rsi*4 + 0x1234]  -> 48 8b 84 b1 34120000 (displacement 34120000h)

        // mov rax, [rbp + rsi*1]           -> 48 8b 44 35 00 (ModRM byte 44 = 01 000 100, mod=01, reg=000, r/m=100) (SIB byte 35 = 00 110 101, scale=00(*1), index=110(rsi), base=101(rbp)) (displacement 00h = 0)
        // mov rax, [rbp + rsi*2]           -> 48 8b 44 75 00 (SIB byte 75 = 01 110 101, scale=01(*2))
        // mov rax, [rbp + rsi*4 + 0x10]    -> 48 8b 44 b5 10 (SIB byte b5 = 10 110 101, scale=10(*4)) (displacement 10h = 16)
        // mov rax, [rbp + rsi*4 + 0x1234]  -> 48 8b 84 b5 34120000 (displacement 34120000h)

        // mov r8, [rsp]          -> 4c 8b 04 24 (ModRM byte 04 = 00 000 100, mod=00, reg=000, r/m=100) (SIB byte 24 = 00 100 100, scale=00, index=100(none), base=100(rsp))
        // mov r8, [rsp - 0x10]   -> 4c 8b 44 24 f0 (ModRM byte 44 = 01 000 100, mod=01, reg=000, r/m=100) (SIB byte 24 = 00 100 100, scale=00, index=100(none), base=100(rsp)) (displacement f0h = -16)
        // mov r8, [rsp - 0x1234] -> 4c 8b 84 24 ccedffff (ModRM byte 84 = 10 000 100, mod=10, reg=000, r/m=100) (SIB byte 24 = 00 100 100, scale=00, index=100(none), base=100(rsp)) (displacement ccedffffh = -0x1234)

        // mov rax, [rsp + rsi*1]          -> 48 8b 04 34 (ModRM byte 04 = 00 000 100, mod=00, reg=000, r/m=100) (SIB byte 34 = 00 110 100, scale=00(*1), index=110(rsi), base=100(rsp))
        // mov rax, [rsp + rsi*2]          -> 48 8b 04 74 (SIB byte 74 = 01 110 100, scale=01(*2))
        // mov rax, [rsp + rsi*4 + 0x10]   -> 48 8b 44 b4 10 (SIB byte b4 = 10 110 100, scale=10(*4)) (displacement 10h = 16)
        // mov rax, [rsp + rsi*4 + 0x1234] -> 48 8b 84 b4 34120000 (displacement 34120000h)
    }

    #[test]
    fn test_encode_mov_immediate_to_register() {
        // | Opcode            | Instruction    | Op/En | 64-Bit Mode | Compat/Leg Mode | Description        |
        // | ---               |  ---           | ---   |  ---        |  ---            |  ---               |
        // | B0+ rb ib         | MOV r8, imm8   | OI    | Valid       | Valid           | Move imm8 to r8.   |
        // | REX + B0+ rb ib   | MOV r8, imm8   | OI    | Valid       | N.E.            | Move imm8 to r8.   |
        // | B8+ rw iw         | MOV r16, imm16 | OI    | Valid       | Valid           | Move imm16 to r16. |
        // | B8+ rd id         | MOV r32, imm32 | OI    | Valid       | Valid           | Move imm32 to r32. |
        // | REX.W + B8+ rd io | MOV r64, imm64 | OI    | Valid       | N.E.            | Move imm64 to r64. |

        // Instruction Operand Encoding
        //
        // | Op/En | Operand 1       | Operand 2     | Operand 3 | Operand 4 |
        // | ---   | ---             | ---           | ---       | ---       |
        // | OI    | opcode + rd (w) | imm8/16/32/64 | N/A       | N/A       |

        // NOTE:
        // When Opcode is B0+rb or B8+rd, the register is encoded in the low 3 bits of the opcode.
        // The extension bit of the register is encoded in the REX.B (instead of REX.R) bit.

        // mov rax, 0x1234567890abcdef ->    48 b8 efcdab9078563412 (REX 48 = 0100 1000, W=1, R=0, X=0, B=0) (opcode b8 = b8 + 0, rd=000(rax)) (immediate efcdab9078563412h)
        // mov eax, 0x90abcdef         ->       b8 efcdab90
        // mov ecx, 0x90abcdef         ->       b9 efcdab90
        // mov r10, 0x1234567890abcdef ->    49 ba efcdab9078563412 (REX 49 = 0100 1001, W=1, R=0, X=0, B=1) (opcode ba = b8 + 2, rd=010(r10))
        // mov r10d, 0x90abcdef        ->    41 ba efcdab90 (REX 41 = 0100 0001, W=0, R=0, X=0, B=1)
        // mov r10w, 0xcdef            -> 66 41 ba efcd (PREFIX 66h = operand size override) (REX 41 = 0100 0001, W=0, R=0, X=0, B=1)
        // mov ax, 0xcdef              -> 66    b8 efcd
        // mov r10b, 0xef              ->    41 b2 ef (REX 41 = 0100 0001, W=0, R=0, X=0, B=1) (opcode b2 = b0 + 2, rd=010(r10b))
        // mov al, 0xef                ->       b0 ef (Opcode b0 = b0 + 0, rd=000(al))
    }

    #[test]
    fn test_encode_mov_immediate_to_memory() {
        // | Opcode            | Instruction      | Op/En | 64-Bit Mode | Compat/Leg Mode | Description          |
        // | ---               |  ---             | ---   |  ---        |  ---            |  ---                 |
        // | C6 /0 ib          | MOV r/m8, imm8   | MI    | Valid       | Valid           | Move imm8 to r/m8.   |
        // | REX + C6 /0 ib    | MOV r/m81, imm8  | MI    | Valid       | N.E.            | Move imm8 to r/m8.   |
        // | C7 /0 iw          | MOV r/m16, imm16 | MI    | Valid       | Valid           | Move imm16 to r/m16. |
        // | C7 /0 id          | MOV r/m32, imm32 | MI    | Valid       | Valid           | Move imm32 to r/m32. |
        // | REX.W + C7 /0 id  | MOV r/m64, imm32 | MI    | Valid       | N.E.            | Move imm32 sign extended to 64-bits to r/m64. |

        // Instruction Operand Encoding
        //
        // | Op/En | Operand 1     | Operand 2     | Operand 3 | Operand 4 |
        // | ---   | ---           | ---           | ---       | ---       |
        // | MI    | ModRM:r/m (w) | imm8/16/32/64 | N/A       | N/A       |

        // mov qword [rax], 0x1234567890abcdef // invalid, immediate exceeds bounds, use `mov rax, imm64` then `mov [rax], rax` instead
        // mov dword [rax], 0x90abcdef         ->       c7 00 efcdab90 (ModRM byte 00 = 00 000 000, mod=00, reg=000(/0), r/m=000(rax)) (immediate efcdab90h)
        // mov word [rax], 0xcdef              -> 66    c7 00 efcd
        // mov byte [rax], 0xef                ->       c6 00 ef
        // mov dword [rcx], 0x90abcdef         ->       c7 01 efcdab90 (ModRM byte 01 = 00 000 001, mod=00, reg=000(/0), r/m=001(rcx))
        // mov dword [r8], 0x90abcdef          ->    41 c7 00 efcdab90 (ModRM byte 00 = 00 000 000, mod=00, reg=000(/0), r/m=000(r8))
    }

    #[test]
    fn test_encode_lea() {

        // LEA -- Load Effective Address
        //
        // | Opcode        | Instruction | Op/En | 64-Bit Mode | Compat/Leg Mode | Description |
        // | ---           |  ---        |  ---  |  ---        |  ---  |  --- |
        // | 8D /r         | LEA r16,m   | RM    | Valid       | Valid | Store effective address for m in register r16. |
        // | 8D /r         | LEA r32,m   | RM    | Valid       | Valid | Store effective address for m in register r32. |
        // | REX.W + 8D /r | LEA r64,m   | RM    | Valid       | N.E.  | Store effective address for m in register r64. |
        //
        // Instruction Operand Encoding
        //
        // | Op/En | Operand 1     | Operand 2     | Operand 3 | Operand 4 |
        // | ---   |  ---          |  ---          |  ---      |  ---      |
        // | RM    | ModRM:reg (w) | ModRM:r/m (r) | N/A       | N/A       |

        // lea rax, [rcx]            -> 48 8d 01 (ModRM byte 01 = 00 000 001, mod=00, reg=000(rax), r/m=001(rcx))
        // lea rax, [r9]             -> 49 8d 01 (REX 49 = 0100 1001, W=1, R=0, X=0, B=1) (ModRM byte 01 = 00 000 001, mod=00, reg=000(rax), r/m=001(r9))
        // lea rax, [rcx+0x10]       -> 48 8d 41 10 (ModRM byte 41 = 01 000 001, mod=01, reg=000(rax), r/m=001(rcx)) (displacement 10h = 16)
        // lea rax, [rcx+0x1234]     -> 48 8d 81 34120000 (ModRM byte 81 = 10 000 001, mod=10, reg=000(rax), r/m=001(rcx)) (displacement 34120000h)
        // lea rax, [rcx+rsi*4+0x10] -> 48 8d 44 b1 10 (ModRM byte 44 = 01 000 100, mod=01, reg=000(rax), r/m=100(SIB)) (SIB byte b1 = 10 110 001, scale=10(*4), index=110(rsi), base=001(rcx)) (displacement 10h = 16)

        // lea rax, [rbp]       -> 48 8d 45 00 (ModRM byte 45 = 01 000 101, mod=01, reg=000(rax), r/m=101(rbp)) (displacement 00h = 0)
        // lea rax, [rsp]       -> 48 8d 04 24 (ModRM byte 04 = 00 000 100, mod=00, reg=000(rax), r/m=100(SIB)) (SIB byte 24 = 00 100 100, scale=00, index=100(none), base=100(rsp))
        // lea rax, [rbp+0x10]  -> 48 8d 45 10 (displacement 10h = 16)
        // lea rax, [rsp-0x10]  -> 48 8d 44 24 f0 (ModRM byte 44 = 01 000 100, mod=01, reg=000(rax), r/m=100(SIB)) (SIB byte 24 = 00 100 100, scale=00, index=100(none), base=100(rsp)) (displacement f0h = -16)

        // todo

        // Address by label:
        //
        // data:
        // num1 dw 0x1234
        // num2 dw 0x5678
        // num3 dw 0x9abc
        //
        // lea rax, [num1]        -> 48 8d 04 25 00000000 (ModRM byte 04 = 00 000 100, mod=00, reg=000(rax), r/m=100(SIB)) (SIB byte 25 = 00 100 101, scale=00, index=100(none), base=101(disp32)) (displacement 00000000h = address of num1)
        // lea rax, [num2]
        // lea rax, [num3]
        // lea rax, [num1 + 0x10]
        // lea rax, [rel num1]
        // lea rax, [rel num2]
        // lea rax, [rel num3]
        // lea rax, [rel num1 + 0x10]

        // todo
    }

    #[test]
    fn test_encode_movzx() {
        // MOVZX  -- Move With Zero-Extendc
        //
        // | Opcode           | Instruction      | Op/En | 64-Bit Mode | Compat/Leg Mode | Description |
        // | ---              |  ---             |  ---  |  ---        |  ---  |  ---                  |
        // | 0F B6 /r         | MOVZX r16, r/m8  | RM    | Valid       | Valid | Move byte to word with zero-extension.   |
        // | 0F B6 /r         | MOVZX r32, r/m8  | RM    | Valid       | Valid | Move byte to doubleword, zero-extension. |
        // | REX.W + 0F B6 /r | MOVZX r64, r/m8  | RM    | Valid       | N.E.  | Move byte to quadword, zero-extension.   |
        // | 0F B7 /r         | MOVZX r32, r/m16 | RM    | Valid       | Valid | Move word to doubleword, zero-extension. |
        // | REX.W + 0F B7 /r | MOVZX r64, r/m16 | RM    | Valid       | N.E.  | Move word to quadword, zero-extension.   |
        //
        // Instruction Operand Encoding
        //
        // | Op/En | Operand 1     | Operand 2     | Operand 3 | Operand 4 |
        // | ---   |  ---          |  ---          |  ---      |  ---      |
        // | RM    | ModRM:reg (w) | ModRM:r/m (r) | N/A       | N/A       |

        // NOTE:
        //
        // there is NO `movzx r64, r/m32`.
        //
        // because the high 32 bits of the destination register are set to **zero** automatically,
        // use `mov r32, r/m32` instead.
        //
        // Details:
        //
        // When in 64-bit mode, operand size determines the number of valid bits in the destination
        // general-purpose register.
        //
        // - 64-bit operands generate a 64-bit result in the destination general-purpose register.
        // - 32-bit operands generate a 32-bit result, zero-extended to a 64-bit result in
        //   the destination general-purpose register.
        // - 8-bit and 16-bit operands generate an 8-bit or 16-bit result.
        //   The upper 56 bits or 48 bits (respectively) of the destination general-purpose register
        //   are not modified by the operation. If the result of an 8-bit or 16-bit operation is intended
        //   for 64-bit address calculation, explicitly sign-extend the register to the full 64-bits.
        //
        // References:
        // Volume 1, Section 3.4.1.1 General-Purpose Registers in 64-Bit Mode

        // movzx ax, cl  -> 66 0f b6 c1 (ModRM byte c1 = 11 000 001, mod=11, reg=000(ax), r/m=001(cl))
        // mocvzx eax, cl ->    0f b6 c1
        // movzx rax, cl -> 48 0f b6 c1 (REX 48 = 0100 1000, W=1, R=0, X=0, B=0) (NASM does not support)
        // movzx eax, cx ->    0f b7 c1
        // movzx rax, cx -> 48 0f b7 c1

        // todo

        // MOVSX/MOVSXD  -- Move With Sign-Extension
        //
        // | Opcode           | Instruction       | Op/En | 64-Bit Mode | Compat/Leg Mode | Description |
        // | ---              |  ---              |  ---  |  ---        |  ---  |  ---                  |
        // | 0F BE /r         | MOVSX r16, r/m8   | RM    | Valid       | Valid | Move byte to word with sign-extension.             |
        // | 0F BE /r         | MOVSX r32, r/m8   | RM    | Valid       | Valid | Move byte to doubleword with sign-extension.       |
        // | REX.W + 0F BE /r | MOVSX r64, r/m8   | RM    | Valid       | N.E.  | Move byte to quadword with sign-extension.         |
        // | 0F BF /r         | MOVSX r32, r/m16  | RM    | Valid       | Valid | Move word to doubleword, with sign-extension.      |
        // | REX.W + 0F BF /r | MOVSX r64, r/m16  | RM    | Valid       | N.E.  | Move word to quadword with sign-extension.         |
        // | 63 /r            | MOVSXD r16, r/m16 | RM    | Valid       | N.E.  | Move word to word with sign-extension.             |
        // | 63 /r            | MOVSXD r32, r/m32 | RM    | Valid       | N.E.  | Move doubleword to doubleword with sign-extension. |
        // | REX.W + 63 /r    | MOVSXD r64, r/m32 | RM    | Valid       | N.E.  | Move doubleword to quadword with sign-extension.   |

        // Instruction Operand Encoding
        //
        // | Op/En | Operand 1     | Operand 2     | Operand 3 | Operand 4 |
        // | ---   |  ---          |  ---          |  ---      |  ---      |
        // | RM    | ModRM:reg (w) | ModRM:r/m (r) | N/A       | N/A       |

        // NOTE:
        // `MOVSXD r16, r/m16` and `MOVSXD r32, r/m32` are side effects of Intel’s systematic documentation.
        // They are functionally identical to `MOV r16, r/m16` and `MOV r32, r/m32` (no actual sign-extension happens).
        // The only useful form of MOVSXD is `MOVSXD r64, r/m32`, which sign-extends a 32-bit value into 64 bits.

        // movsx ax, cl    -> 66    0f be c1 (ModRM byte c1 = 11 000 001, mod=11, reg=000(ax), r/m=001(cl))
        // movsx eax, cl   ->       0f be c1
        // movsx rax, cl   ->    48 0f be c1
        // movsx eax, cx   ->       0f bf c1
        // movsx rax, cx   ->    48 0f bf c1
        // movsxd rax, ecx ->    48    63 c1

        // todo

        // Other convertion (ANASOM does not support these instructions)
        //
        // - cbw:   al -> ax
        // - cwde:  ax -> eax
        // - cdqe: eax -> rax
        // - cwd:   ax -> dx:ax
        // - cdq:  eax -> edx:eax
        // - cqo:  rax -> rdx:rax
        //
        // CBW/CWDE/CDQE  -- Convert Byte to Word/Convert Word to Doubleword/Convert Doubleword to Quadword
        //
        // | Opcode     | Instruction | Op/En | 64-bit Mode | Compat/Leg Mode | Description                |
        // | ---        |  ---        |  ---  |  ---        |  ---            |  ---                       |
        // | 98         | CBW         | ZO    | Valid       | Valid           | AX := sign-extend of AL.   |
        // | 98         | CWDE        | ZO    | Valid       | Valid           | EAX := sign-extend of AX.  |
        // | REX.W + 98 | CDQE        | ZO    | Valid       | N.E.            | RAX := sign-extend of EAX. |
        //
        // CWD/CDQ/CQO  -- Convert Word to Doubleword/Convert Doubleword to Quadword
        //
        // | Opcode     | Instruction | Op/En | 64-Bit Mode | Compat/Leg Mode | Description                     |
        // | ---        |  ---        |  ---  |  ---        |  ---            |  ---                            |
        // | 99         | CWD         | ZO    | Valid       | Valid           | DX:AX := sign-extend of AX.     |
        // | 99         | CDQ         | ZO    | Valid       | Valid           | EDX:EAX := sign-extend of EAX.  |
        // | REX.W + 99 | CQO         | ZO    | Valid       | N.E.            | RDX:RAX := sign-extend of RAX.  |
        //
        // Instruction Operand Encoding
        //
        // | Op/En | Operand 1 | Operand 2 | Operand 3 | Operand 4 |
        // | ---   |  ---      |  ---      |  ---      |  ---      |
        // | ZO    | N/A       | N/A       | N/A       | N/A       |
    }
}
