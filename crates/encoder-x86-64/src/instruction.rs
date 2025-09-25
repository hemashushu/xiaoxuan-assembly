// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

// use bitflags::bitflags;

use crate::mnemonic::Mnemonic;

#[derive(Debug, PartialEq, Clone)]
pub struct Instruction {
    pub mnemonic: Mnemonic,
    pub operands: [Option<Operand>; 4],
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operand {
    Register(u8),           // Register operand, e.g., RAX, RBX
    Immediate8(u8),         // Immediate value operand
    Immediate16(u16),       // Immediate value operand
    Immediate32(u32),       // Immediate value operand
    Immediate64(u64),       // Immediate value operand
    Memory(u64 /* size */), // Memory address operand
}

/* *
 * Not supported:
 *
 * - Segment registers (CS, DS, ES, SS, 16-bit), CPU managed automatically
 * - X87 FPU (ST0–ST7, 80-bit)
 * - Control registers (CR0–CR4, CR8, 32-bit)
 * - System table pointer registers (GDTR, LDTR, IDTR, 16-bit, task register TR)
 * - MMX (MM0–MM7, 64-bit, the low part of ST0–ST7)
 * - Test registers (TR3–TR7)
 * - Memory Protection Extensions, MPX (BND0–BND3, 128-bit), deprecated
 * - AVX-512 (XMM0-XMM31, YMM0-YMM31, ZMM0-ZMM31, k0-k7, 512-bit)
 *
 * Supported:
 *
 * - Debug Registers (DR0–DR7, 32-bit)
 * - RDX:RAX register pair representing a 128-bit operand.
 * - SSE, SSE2, SSE3 (XMM0-XMM15, 128-bit)
 * - AVX, AVX2 (XMM0-XMM15, YMM0-YMM15, 256-bit), recommended
 *
 * See:
 * - Volume 1, Section 3.4.1.1 General-Purpose Registers in 64-Bit Mode
 * - Volume 1, Section 3.4.2.1 Segment Registers in 64-Bit Mode
 * - Volume 1, Section 3.4.3 EFLAGS Register
 * - Volume 1, Section 3.6.1 Operand Size and Address Size in 64-Bit Mode
 *
 * From:
 *
 * - Intel® 64 and IA-32 Architectures Software Developer Manuals
 *   https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html
 *
 * - x86_64 Assembly Resources
 *   https://gist.github.com/aagontuk/177947659828c1fe6611d4971bfddd27
 *
 * - SIMD Instruction Sets
 *   https://www.syncfusion.com/succinctly-free-ebooks/assemblylanguage/simd-instruction-sets
 *
 */
#[derive(Debug, PartialEq, Clone)]
pub enum Register {
    /* *
     * AH, BH, CH and DH are conflicted with REX prefix in long mode, they are
     * not allowed since ANASM is intented for developing modern x86-64 applications.
     *
     * Background
     * ==========
     *
     * ## 1. What is the REX prefix?
     *
     * * In x86 machine code, instructions can be preceded by *prefix bytes* that modify
     *   their behavior (e.g. operand size, segment overrides).
     * * For **x86-64**, AMD introduced a new kind of prefix called **REX**,
     *   which is a **1-byte prefix in the range `0x40–0x4F`**.
     * * It extends the instruction encoding to allow:
     *
     *   1. **Access to new registers** (`R8–R15`)
     *   2. **Access to the high 64-bit register halves** (64-bit operand size)
     *   3. **Access to 64-bit addressing** with new registers as base/index.
     *
     * ## 2. REX prefix format
     *
     * It’s one byte:
     *
     * ```
     * 0100WRXB
     * ```
     *
     * * The upper 4 bits are fixed `0100`.
     * * The lower 4 bits are flags:
     *
     * | Bit | Name                                                        | Effect |
     * | --- | ----------------------------------------------------------- | ------ |
     * | W   | 1 = use 64-bit operand size (instead of default 32-bit)     |        |
     * | R   | Extends the **reg field** of ModR/M (adds 8 more registers) |        |
     * | X   | Extends the **index field** of SIB (adds 8 more registers)  |        |
     * | B   | Extends the **r/m field** of ModR/M or base field of SIB    |        |
     *
     * So with REX, you can address registers `R8–R15` and also set 64-bit operand sizes.
     *
     * ## 3. Example
     *
     * Without REX:
     *
     * ```asm
     * mov eax, ebx   ; 32-bit move
     * ```
     *
     * Machine code: `89 D8`
     *
     * With REX.W=1 (0x48):
     *
     * ```asm
     * mov rax, rbx   ; 64-bit move
     * ```
     *
     * Machine code: `48 89 D8`
     *
     * With REX.B=1:
     *
     * ```asm
     * mov rax, r8    ; uses extended register
     * ```
     *
     * Machine code: `4C 89 C0`
     *
     * ## 4. Why does it conflict with AH/BH/CH/DH?
     *
     * * Historically, x86 had 8-bit registers `AL, CL, DL, BL, AH, CH, DH, BH`.
     * * When x86-64 added new low-byte registers (`SPL, BPL, SIL, DIL, R8B–R15B`), they reused the same opcode space.
     * * To distinguish them, you must use a REX prefix.
     * * But if a REX prefix is present, the AH/BH/CH/DH encodings are no longer valid.
     *   This is why assemblers disallow mixing REX with AH/BH/CH/DH.
     *
     * Summary:
     *
     * - The REX prefix (0x40–0x4F) is the special instruction prefix that unlocks 64-bit operand sizes
     *   and access to the new registers `R8–R15`.
     * - But once REX is used, the “old” high-byte registers (`AH, BH, CH, DH`) are off-limits.
     * - REX = 0100_0000 (0x40) is usually equivalent to the same instruction without it,
     *   except when you’re working with 8-bit registers, because it changes the
     *   available set (removes AH/BH/CH/DH, introduces SPL/BPL/SIL/DIL).
     *
     * Other prefixes
     * ==============
     *
     * | Prefix type       | Bytes                  | Still valid in long mode? | Notes                                |
     * | ----------------- | ---------------------- | ------------------------- | ------------------------------------ |
     * | Legacy lock/rep   | F0, F2, F3             | Yes                       | atomic memory ops, String ops        |
     * | Segment overrides | 2E, 36, 3E, 26, 64, 65 | Yes (but only FS/GS, 64/65 matter) |                             |
     * | Operand size      | 66                     | Yes                       | For 16-bit ops in 64-bit mode        |
     * | Address size      | 67                     | Yes                       | For 32-bit addressing in 64-bit mode |
     * | REX               | 40 – 4F                | Yes                       | Unlocks 64-bit regs, ops             |
     * | VEX               | C4/C5                  | Yes                       | AVX, AVX2 SIMD                       |
     * | EVEX              | 62                     | Yes                       | AVX-512                              |
     * | XOP (AMD)         | 8F                     | Partially                 | Rare, non-Intel                      |
     *
     *
     * - Operand-size override
     *   66: switches between 16-bit and 32/64-bit operand size.
     *   In long mode, default operand size = 32 bits; adding REX.W makes it 64.
     *   So 66 is still used for 16-bit ops.
     *
     * - Address-size override
     *   67: switches between 64-bit addressing and 32-bit (or 16-bit) addressing.
     *   In long mode, default addressing = 64 bits;
     *   with 67 you can do 32-bit addressing.
     *
     * Volume 1 Table 3-4. Effective Operand- and Address-Size Attributes in 64-Bit Mode
     *
     * REX.W Prefix            |  0  0  0  0  1  1  1  1
     * Operand-Size Prefix 66H |  N  N  Y  Y  N  N  Y  Y
     * Address-Size Prefix 67H |  N  Y  N  Y  N  Y  N  Y
     * Effective Operand Size  | 32 32 16 16 64 64 64 64
     * Effective Address Size  | 64 32 64 32 64 32 64 32
     *
     * See: Volume 1, Section 3.6.1 Operand Size and Address Size in 64-Bit Mode
     *
     * */
    RAX, EAX, AX, AL, /* alias R0, (REX.R/B, ModRM.reg/rm) = 0,0 */
    RCX, ECX, CX, CL, /* alias R1, (REX.R/B, ModRM.reg/rm) = 0,1 */
    RDX, EDX, DX, DL, /* alias R2, (REX.R/B, ModRM.reg/rm) = 0,2 */
    RBX, EBX, BX, BL, /* alias R3, (REX.R/B, ModRM.reg/rm) = 0,3 */
    RSP, ESP, SP, SPL, /* alias R4, (REX.R/B, ModRM.reg/rm) = 0,4 */
    RBP, EBP, BP, BPL, /* alias R5, (REX.R/B, ModRM.reg/rm) = 0,5 */
    RSI, ESI, SI, SIL, /* alias R6, (REX.R/B, ModRM.reg/rm) = 0,6 */
    RDI, EDI, DI, DIL, /* alias R7, (REX.R/B, ModRM.reg/rm) = 0,7 */
    R8, R8D, R8W, R8B, /* (REX.R/B, ModRM.reg/rm) = 1.0 */
    R9, R9D, R9W, R9B, /* (REX.R/B, ModRM.reg/rm) = 1.1 */
    R10, R10D, R10W, R10B, /* (REX.R/B, ModRM.reg/rm) = 1.2 */
    R11, R11D, R11W, R11B, /* (REX.R/B, ModRM.reg/rm) = 1.3 */
    R12, R12D, R12W, R12B, /* (REX.R/B, ModRM.reg/rm) = 1.4 */
    R13, R13D, R13W, R13B, /* (REX.R/B, ModRM.reg/rm) = 1.5 */
    R14, R14D, R14W, R14B, /* (REX.R/B, ModRM.reg/rm) = 1.6 */
    R15, R15D, R15W, R15B, /* (REX.R/B, ModRM.reg/rm) = 1.7 */
    XMM0, YMM0, ZMM0, /* (REX.R/B, ModRM.reg/rm) = 0,0 */
    XMM1, YMM1, ZMM1, /* (REX.R/B, ModRM.reg/rm) = 0,1 */
    XMM2, YMM2, ZMM2, /* (REX.R/B, ModRM.reg/rm) = 0,2 */
    XMM3, YMM3, ZMM3, /* (REX.R/B, ModRM.reg/rm) = 0,3 */
    XMM4, YMM4, ZMM4, /* (REX.R/B, ModRM.reg/rm) = 0,4 */
    XMM5, YMM5, ZMM5, /* (REX.R/B, ModRM.reg/rm) = 0,5 */
    XMM6, YMM6, ZMM6, /* (REX.R/B, ModRM.reg/rm) = 0,6 */
    XMM7, YMM7, ZMM7, /* (REX.R/B, ModRM.reg/rm) = 0,7 */
    XMM8, YMM8, ZMM8, /* (REX.R/B, ModRM.reg/rm) = 1.0 */
    XMM9, YMM9, ZMM9, /* (REX.R/B, ModRM.reg/rm) = 1.1 */
    XMM10, YMM10, ZMM10, /* (REX.R/B, ModRM.reg/rm) = 1.2 */
    XMM11, YMM11, ZMM11, /* (REX.R/B, ModRM.reg/rm) = 1.3 */
    XMM12, YMM12, ZMM12, /* (REX.R/B, ModRM.reg/rm) = 1.4 */
    XMM13, YMM13, ZMM13, /* (REX.R/B, ModRM.reg/rm) = 1.5 */
    XMM14, YMM14, ZMM14, /* (REX.R/B, ModRM.reg/rm) = 1.6 */
    XMM15, YMM15, ZMM15, /* (REX.R/B, ModRM.reg/rm) = 1.7 */
    RIP, EIP, IP,
    RFLAGS, EFLAGS, FLAGS,

    // TLS segment in Windows x86-64,
    // e.g. `mov rax, qword [gs:0x28]`.
    // writing GS has no effect,
    // GS.base is set up by the OS for thread-local storage (TLS)
    // and can be changed only with syscall `arch_prctl`
    // the final address is `GS.base + offset`.
    GS,

    // FS is similar to GS, but used in Linux x86-64 (and Windows 32-bit) user space.
    // for example, _glibc_ set up FS.base with `arch_prctl(ARCH_SET_FS, addr)`
    // once a thread is created.
    // GS is more used in kernel space to access per-CPU data.
    FS,
}

/// Definition of an instruction, including its mnemonic and encoding details.
///
/// Instruction list:
/// - http://ref.x86asm.net/index.html
/// - http://ref.x86asm.net/coder64.html
/// - https://www.felixcloutier.com/x86/
///
#[derive(Debug, PartialEq, Clone)]
pub struct InstructionDefinition {
    pub mnemonic: Mnemonic,
    pub two_bytes: bool, // true if the instruction uses the 0F prefix
    pub primary_opcode: u8,
    pub secondary_opcode: Option<u8>,
    pub operands: [Option<OperandDefinition>; 4],
}

#[derive(Debug, PartialEq, Clone)]
pub struct OperandDefinition {
    pub encoding: OperandEncoding,
    pub access: OperandAccess,
    pub size: OperandSize,
    pub operand_type: OperandType
}

#[derive(Debug, PartialEq, Clone)]
pub enum OperandEncoding {
    ModRmReg,  // ModRM:reg(...)
    ModRmRm,   // ModRM:r/m(...)
    SIB,       // SIB byte
    Immediate, // 8/16/32/64-bit Immediate
}

#[derive(Debug, PartialEq, Clone)]
pub enum OperandAccess {
    Read,
    Write,
    ReadWrite
}

#[derive(Debug, PartialEq, Clone)]
pub enum OperandType {
    Register(RegisterType),
    Mem(Option<OperandSize>),
    Immediate,
}

#[derive(Debug, PartialEq, Clone)]
pub enum RegisterType {
    General,
    AVX,
}

#[derive(Debug, PartialEq, Clone)]
pub enum OperandSize {
    Unsized,
    Byte,       // 8-bit
    Word,       // 16-bit
    Dword,      // 32-bit
    // Far16,      // 16:16
    // Fword,      // 48-bit
    // Far32,      // 16:32
    Qword,      // 64-bit
    // Tbyte,      // 80-bit
    // Far64,      // 16:64
    XMMWord,    // 128-bit
    YMMWord,    // 256-bit
    ZMMWord,    // 512-bit
}

// bitflags! {
#[repr(u8)]
#[derive(Debug, PartialEq, Clone)]
pub enum REW {
    // Operand size extension
    // 1 = 64-bit operand size (REX.W)
    // 0 = default operand size (32-bit, or 16-bit with 0x66 prefix)
    W = 0b0000_1000,

    // ModRM.reg Register extension
    // 1 = extends ModRM.reg field (REX.R)
    //     Extends the reg field of the ModR/M byte with an extra high bit.
    //     Allows access to registers r8–r15.
    // 0 = no extension
    R = 0b0000_0100,

    // SIB index extension
    // 1 = extends SIB.index field (REX.X)
    //     Extends the index field of the SIB byte.
    //     Allows index registers r8–r15.
    // 0 = no extension
    X = 0b0000_0010,

    // ModRM.r/m Register extension or SIB Base extension
    // 1 = extends ModRM.r/m field or SIB base field (REX.B)
    //     Extends the r/m field of the ModR/M byte or the base field of the SIB byte.
    //     Allows access to registers r8–r15.
    // 0 = no extension
    B = 0b0000_0001,
}
// }

#[derive(Debug, PartialEq, Clone)]
pub struct ModRM {
    pub Mode: u8, // 2 bits
    pub Register: u8, // 3 bits
    pub RegOrMemory: u8,  // 3 bits
}

#[derive(Debug, PartialEq, Clone)]
pub struct SIB {
    pub Scale: u8, // 2 bits
    pub Index: u8, // 3 bits
    pub Base: u8,  // 3 bits
}