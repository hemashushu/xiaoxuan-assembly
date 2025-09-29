// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

/* *
 * The limitations of this encoder:
 * - Only support a subset of x86-64 instructions
 * - Only support 64-bit (long) mode
 * - PIE (RIP-relative addressing) code only
 * - Only support a subset of addressing modes:
 *   - base (required) + index*scale (optional) + displacement (optional)
 *   - displacement only (RIP-relative addressing)
 *   - FS/GS segment override
 * - Effective address only accepts 64-bit registers,
 *   i.e. 32-bit compatibility addressing is not supported,
 *   e.g. "mov eax, dword [ebx]" is invalid.
 */

mod encode;
mod instruction;
mod mnemonic;
mod parser;
