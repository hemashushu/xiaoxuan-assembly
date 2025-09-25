// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use crate::instruction::Instruction;

/* *
 *
 * Instruction Format:
 * - legacy prefix: group 1 - 4
 * - REX: 0,1 byte
 * - Opcode: 1,2,3 bytes
 * - ModRM: 0,1 byte (Mod: 2-bit, Reg: 3-bit, R/M: 3-bit)
 * - SIB: 0,1 byte (Scale: 2-bit, Index: 3-bit, Base: 3-bit)
 * - Displacement: 0,1,2,4 (,8) bytes
 * - Immediate: 0,1,2,4 (,8) bytes
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
    fn test_encode_mov_register_to_register() {

        // | Opcode        | Instruction    | Op/En | 64-Bit Mode | Compat/Leg Mode | Description        |
        // | ---           |  ---           |  ---  |  ---        |  ---            |  ---               |
        // | 88 /r         | MOV r/m8, r8   | MR    | Valid       | Valid           | Move r8 to r/m8.   |
        // | REX + 88 /r   | MOV r/m8, r8   | MR    | Valid       | N.E.            | Move r8 to r/m8.   |
        // | 89 /r         | MOV r/m16, r16 | MR    | Valid       | Valid           | Move r16 to r/m16. |
        // | 89 /r         | MOV r/m32, r32 | MR    | Valid       | Valid           | Move r32 to r/m32. |
        // | REX.W + 89 /r | MOV r/m64, r64 | MR    | Valid       | N.E.            | Move r64 to r/m64. |

        // Instruction Operand Encoding
        // -----------------------------
        // | Op/En | Operand 1     | Operand 2     | Operand 3 | Operand 4 |
        // | ---   |  ---          |  ---          |  ---      |  ---      |
        // | MR    | ModRM:r/m (w) | ModRM:reg (r) | N/A       | N/A       |

        // mov rax, rcx -> 48 89 c8 (REX 48 = 0100 1000, W=1, R=0, X=0, B=0) (ModRM byte c8 = 11 001 000, mod=11, reg=001, r/m=000)
        // mov rdx, rbx -> 48 89 da (REX 48 = 0100 1000, W=1, R=0, X=0, B=0) (ModRM byte d3 = 11 011 010, mod=11, reg=011, r/m=010)
        // mov eax, ecx ->    89 c8
        // mov ax, cx   -> 66 89 c8
        // mov al, cl   ->    88 c8

        // mov r9, rcx  ->    49 89 c9 (REX 49 = 0100 1001, W=1, R=0, X=0, B=1) (ModRM byte c9 = 11 001 001, mod=11, reg=001, r/m=001)
        // mov r9d, ecx ->    41 89 c9 (REX 41 = 0100 0001, W=0, R=0, X=0, B=1)
        // mov r9w, cx  -> 66 41 89 c9
        // mov r9b, cl  ->    41 88 c9

        // mov ecx, r9d ->    44 89 c9 (REX 44 = 0100 0100, W=0, R=1, X=0, B=0) (ModRM byte c9 = 11 001 001, mod=11, reg=001, r/m=001)

        //todo!()
    }

    #[test]
    fn test_encode_mov_between_register_and_memory() {
        // note:
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

        // mov rax, [rbx] -> 48 8b 03
        // mov eax, [ebx] -> 67 8b 03
        // mov ax, [bx]   -> invalid 64-bit effective address in 64-bit mode
        // mov al, [bl]   -> invalid 64-bit effective address in 64-bit mode
        //
        // mov [rbx], rax -> 48 89 03
        // mov [ebx], eax -> 67 89 03
        // mov [bx], ax   -> invalid 64-bit effective address in 64-bit mode
        // mov [bl], al   -> invalid 64-bit effective address in 64-bit mode

        // memory address with offset
        //
        // mov rax, [rbx + 0x10] -> 48 8b 43 10
        // mov eax, [ebx + 0x10] -> 67 8b 43 10
        // mov ax, [bx + 0x10]   -> invalid 64-bit effective address in 64-bit mode
        // mov al, [bx + 0x10]   -> invalid 64-bit effective address in 64-bit mode
        //
        // mov [rbx + 0x10], rax -> 48 89 43 10
        // mov [ebx + 0x10], eax -> 67 89 43 10
        // mov [bx + 0x10], ax   -> invalid 64-bit effective address in 64-bit mode
        // mov [bx + 0x10], al   -> invalid 64-bit effective address in 64-bit mode
    }

    #[test]
    fn test_encode_mov_immediate_to_register() {
        // mov rax, 0x1234567890abcdef -> 48 b8 ef cdab9078563412
        // mov eax, 0x90abcdef         ->    b8 ef cdab90
        // mov ax, 0xcdef              -> 66 b8 ef cd
        // mov al, 0xef                ->    b0 ef
    }

    #[test]
    fn test_encode_mov_immediate_to_memory() {
        // mov qword [rax], 0x1234567890abcdef // invalid, use `mov rax, imm64` then `mov [rax], rax` instead
        // mov dword [rax], 0x90abcdef         ->    c7 00 efcdab90
        // mov word [rax], 0xcdef              -> 66 c7 00 efcd
        // mov byte [rax], 0xef                ->    c6 00 ef
    }

    #[test]
    fn test_encode_mov_memory_to_register_by_lable() {

        // move from memory to register:
        //
        // num dq 0x1234567890abcdef
        // mov rax, qword [num] -> efcdab9078563412, 488b042500000000
        // num dd 0x12345678
        // mov eax, dword [num] ->         78563412,   8b042500000000
        // num dw 0x1234
        // mov ax, word [num]   ->             3412, 668b042500000000
        // num db 0x12
        // mov al, byte [num]   ->               12,   8a042500000000

        // test the address
        //
        // num1 dw 0x1234
        // num2 dw 0x5678
        // num3 dw 0x9abc
        // mov ax,[num1] -> 3412, 7856, bc9a, 668b042500000000
        //
        // replace `mov ...` with `mov ax,[rel num1]` -> 3412, 7856, bc9a, 668b05f3ffffff

        // num1 dw 0x1234
        // num2 dw 0x5678
        // num3 dw 0x9abc
        // mov ax,[num2] -> 3412, 7856, bc9a, 668b042502000000
        //
        // replace `mov ...` with `mov ax,[rel num2]` -> 3412, 7856, bc9a, 668b05f5ffffff

        // num1 dw 0x1234
        // num2 dw 0x5678
        // num3 dw 0x9abc
        // mov ax,[num3] -> 3412, 7856, bc9a, 668b042504000000
        //
        // replace `mov ...` with `mov ax,[rel num3]` -> 3412, 7856, bc9a, 668b05f7ffffff

        // move from register to memory

        // num db 0x12
        // mov byte [num], al   ->               12,   88042500000000
        // num dw 0x1234
        // mov word [num], ax   ->             3412, 6689042500000000
        // num dd 0x12345678
        // mov dword [num], eax ->         78563412,   89042500000000
        // num dq 0x1234567890abcdef
        // mov qword [num], rax -> efcdab9078563412, 4889042500000000
    }

    #[test]
    fn test_encode_movzx() {
        // Unsigned Conversions (extending)
        //
        // movzx <dest>, <src>
        // movzx <reg16>, <r/m8>
        // movzx <reg32>, <r/m8>
        // movzx <reg32>, <r/m16>
        // movzx <reg64>, <r/m8>
        // movzx <reg64>, <r/m16>
        // movzx <reg64>, <r/m32> // invalid

        // because the high 32 bits of the destination register are set to zero automatically,
        // use `mov <reg32>, <r/m32>` instead.

        // Signed Conversions
        // movsx <dest>, <src>
        // movsx <reg16>, <r/m8>
        // movsx <reg32>, <r/m8>
        // movsx <reg32>, <r/m16>
        // movsx <reg64>, <r/m8>
        // movsx <reg64>, <r/m16>
        //
        // movsxd allows 32-bit to 64-bit sign extension
        // movsxd <reg64>, <r/m32>

        // Particular
        // cbw: al -> ax
        // cwd: ax -> dx:ax
        // cwde: ax -> eax
        // cdq: eax -> edx:eax
        // cdqe: eax -> rax
        // cqo: rax -> rdx:rax
    }

    #[test]
    fn test_encode_lea() {
        // lea rax, [rbx]
        // lea rax, [ebx]
        // lea rax, [bx]
        // lea rax, [bl]

        // lea eax, [rbx]
        // lea eax, [ebx]

        // lea rax, [rbx + 0x10] ->
        // lea rax, [ebx + 0x10] ->
        // lea rax, [bx + 0x10] ->
        // lea rax, [bl + 0x10] ->

        // lea eax, [rbx + 0x10] ->
        // lea eax, [ebx + 0x10] ->
    }

    #[test]
    fn test_encode_lea_with_lable() {
        // lea rax, [num]
        // lea rax, [num + 0x10] ->

        // lea eax, [num]
        // lea eax, [num + 0x10] ->

        // with `rel`
    }
}
