// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::{collections::HashMap, sync::Once};

use anna_types::opcode::Opcode;

static INIT: Once = Once::new();

// assembly instructions are not identical to machine instructions,
// this is the instructions map.
pub static mut INSTRUCTION_MAP: Option<HashMap<&'static str, InstructionSyntaxKind>> = None;

// group instructions according to the syntax
// for easier parsing.
#[derive(Debug, PartialEq, Clone)]
pub enum InstructionSyntaxKind {
    // (i32.imm 123)
    // (i32.imm 0x123)
    // (i32.imm 0b1010)
    ImmI32,

    // (i64.imm 123)
    // (i64.imm 0x123)
    // (i64.imm 0b1010)
    ImmI64,

    // (f32.imm 3.14)
    // (f32.imm 0x1.23p4)
    ImmF32,

    // (f64.imm 3.14)
    // (f64.imm 0x1.23p4)
    ImmF64,

    // (local.load $name)
    // (addr.local $name)
    // (local.load $name offset)                ;; optional offset
    LocalLoad(Opcode),

    // (local.store $name VALUE)
    // (local.store $name offset VALUE)         ;; optional offset
    LocalStore(Opcode),

    // (data.load $name)
    // (addr.data $name)
    // (addr.local_thread_data $name)
    // (data.load $name offset)                 ;; optional offset
    DataLoad(Opcode),

    // (data.store $name VALUE)
    // (data.store $name offset VALUE)          ;; optional offset
    DataStore(Opcode),

    // (memory.load ADDR)
    // (memory.load offset ADDR)                ;; optional offset
    MemoryLoad(Opcode),

    // (memory.store ADDR VALUE)
    // (memory.store offset ADDR VALUE)         ;; optional offset
    MemoryStore(Opcode),

    // (inst_name VALUE)
    UnaryOp(Opcode),

    // (i32.inc num VALUE)
    // (i32.dec num VALUE)
    // (i64.inc num VALUE)
    // (i64.dec num VALUE)
    UnaryOpWithImmI64(Opcode),

    // (inst_name LHS RHS)
    BinaryOp(Opcode),

    // (i32.atomic_rmw rmw_op ADDR VALUE)
    // (i64.atomic_rmw rmw_op ADDR VALUE)
    AtomicRmw(Opcode),

    // (i32.atomic_cas ADDR EXPECT_VALUE NEW_VALUE)
    AtomicCas,

    // (when (local...) TEST CONSEQUENT)
    // pesudo instruction, overwrite the original control flow instructions
    When,

    // (if (param...) (result...) (local...)
    //            TEST CONSEQUENT ALTERNATE)
    // pesudo instruction, overwrite the original control flow instructions
    If,

    // (branch (param...) (result...) (local...)
    //     (case TEST_0 CONSEQUENT_0)
    //     ...
    //     (case TEST_N CONSEQUENT_N)
    //     (default CONSEQUENT_DEFAULT) ;; optional
    // )
    // pesudo instruction, overwrite the original control flow instructions
    Branch,

    // (for (param...) (result...) (local...) INSTRUCTION)
    // pesudo instruction, overwrite the original control flow instructions
    For,

    // instruction sequence:
    //
    // - 'do', for the tesing and branches
    // - 'break', for break recur
    // - 'recur', for recur
    // - 'return', for exit function
    // - 'rerun', for recur function
    Sequence(&'static str),

    // (call $name OPERAND_0 ... OPERAND_N)
    Call,

    // (dyncall FUNC_ADDR OPERAND_0 ... OPERAND_N)
    DynCall,

    // (syscall syscall_num OPERAND_0 ... OPERAND_N)
    SysCall,

    // (trap num)
    Trap,

    // (addr.function $name)
    AddrFunction,
}

pub fn init_instruction_map() {
    INIT.call_once(|| {
        init_instruction_map_internal();
    });
}

fn init_instruction_map_internal() {
    let mut table: HashMap<&'static str, InstructionSyntaxKind> = HashMap::new();

    let mut add = |name: &'static str, inst_syntax_kind: InstructionSyntaxKind| {
        table.insert(name, inst_syntax_kind);
    };

    // local load i64
    add(
        "local.load64_i64",
        InstructionSyntaxKind::LocalLoad(Opcode::local_load64_i64),
    );
    add(
        "local.load64_f64",
        InstructionSyntaxKind::LocalLoad(Opcode::local_load64_f64),
    );
    add(
        "local.load64_i32_s",
        InstructionSyntaxKind::LocalLoad(Opcode::local_load64_i32_s),
    );
    add(
        "local.load64_i32_u",
        InstructionSyntaxKind::LocalLoad(Opcode::local_load64_i32_u),
    );
    add(
        "local.load64_i16_s",
        InstructionSyntaxKind::LocalLoad(Opcode::local_load64_i16_s),
    );
    add(
        "local.load64_i16_u",
        InstructionSyntaxKind::LocalLoad(Opcode::local_load64_i16_u),
    );
    add(
        "local.load64_i8_s",
        InstructionSyntaxKind::LocalLoad(Opcode::local_load64_i8_s),
    );
    add(
        "local.load64_i8_u",
        InstructionSyntaxKind::LocalLoad(Opcode::local_load64_i8_u),
    );

    // local load i32
    add(
        "local.load32_i32",
        InstructionSyntaxKind::LocalLoad(Opcode::local_load32_i32),
    );
    add(
        "local.load32_f32",
        InstructionSyntaxKind::LocalLoad(Opcode::local_load32_f32),
    );
    add(
        "local.load32_i16_s",
        InstructionSyntaxKind::LocalLoad(Opcode::local_load32_i16_s),
    );
    add(
        "local.load32_i16_u",
        InstructionSyntaxKind::LocalLoad(Opcode::local_load32_i16_u),
    );
    add(
        "local.load32_i8_s",
        InstructionSyntaxKind::LocalLoad(Opcode::local_load32_i8_s),
    );
    add(
        "local.load32_i8_u",
        InstructionSyntaxKind::LocalLoad(Opcode::local_load32_i8_u),
    );

    // local store
    add(
        "local.store64",
        InstructionSyntaxKind::LocalStore(Opcode::local_store64),
    );
    add(
        "local.store32",
        InstructionSyntaxKind::LocalStore(Opcode::local_store32),
    );
    add(
        "local.store16",
        InstructionSyntaxKind::LocalStore(Opcode::local_store16),
    );
    add(
        "local.store8",
        InstructionSyntaxKind::LocalStore(Opcode::local_store8),
    );

    // data load i64
    add(
        "data.load64_i64",
        InstructionSyntaxKind::DataLoad(Opcode::data_load64_i64),
    );
    add(
        "data.load64_f64",
        InstructionSyntaxKind::DataLoad(Opcode::data_load64_f64),
    );
    add(
        "data.load64_i32_s",
        InstructionSyntaxKind::DataLoad(Opcode::data_load64_i32_s),
    );
    add(
        "data.load64_i32_u",
        InstructionSyntaxKind::DataLoad(Opcode::data_load64_i32_u),
    );
    add(
        "data.load64_i16_s",
        InstructionSyntaxKind::DataLoad(Opcode::data_load64_i16_s),
    );
    add(
        "data.load64_i16_u",
        InstructionSyntaxKind::DataLoad(Opcode::data_load64_i16_u),
    );
    add(
        "data.load64_i8_s",
        InstructionSyntaxKind::DataLoad(Opcode::data_load64_i8_s),
    );
    add(
        "data.load64_i8_u",
        InstructionSyntaxKind::DataLoad(Opcode::data_load64_i8_u),
    );

    // data load i32
    add(
        "data.load32_i32",
        InstructionSyntaxKind::DataLoad(Opcode::data_load32_i32),
    );
    add(
        "data.load32_f32",
        InstructionSyntaxKind::DataLoad(Opcode::data_load32_f32),
    );
    add(
        "data.load32_i16_s",
        InstructionSyntaxKind::DataLoad(Opcode::data_load32_i16_s),
    );
    add(
        "data.load32_i16_u",
        InstructionSyntaxKind::DataLoad(Opcode::data_load32_i16_u),
    );
    add(
        "data.load32_i8_s",
        InstructionSyntaxKind::DataLoad(Opcode::data_load32_i8_s),
    );
    add(
        "data.load32_i8_u",
        InstructionSyntaxKind::DataLoad(Opcode::data_load32_i8_u),
    );

    // data store
    add(
        "data.store64",
        InstructionSyntaxKind::DataStore(Opcode::data_store64),
    );
    add(
        "data.store32",
        InstructionSyntaxKind::DataStore(Opcode::data_store32),
    );
    add(
        "data.store16",
        InstructionSyntaxKind::DataStore(Opcode::data_store16),
    );
    add(
        "data.store8",
        InstructionSyntaxKind::DataStore(Opcode::data_store8),
    );

    // memory load i64
    add(
        "memory.load64_i64",
        InstructionSyntaxKind::MemoryLoad(Opcode::memory_load64_i64),
    );
    add(
        "memory.load64_f64",
        InstructionSyntaxKind::MemoryLoad(Opcode::memory_load64_f64),
    );
    add(
        "memory.load64_i32_s",
        InstructionSyntaxKind::MemoryLoad(Opcode::memory_load64_i32_s),
    );
    add(
        "memory.load64_i32_u",
        InstructionSyntaxKind::MemoryLoad(Opcode::memory_load64_i32_u),
    );
    add(
        "memory.load64_i16_s",
        InstructionSyntaxKind::MemoryLoad(Opcode::memory_load64_i16_s),
    );
    add(
        "memory.load64_i16_u",
        InstructionSyntaxKind::MemoryLoad(Opcode::memory_load64_i16_u),
    );
    add(
        "memory.load64_i8_s",
        InstructionSyntaxKind::MemoryLoad(Opcode::memory_load64_i8_s),
    );
    add(
        "memory.load64_i8_u",
        InstructionSyntaxKind::MemoryLoad(Opcode::memory_load64_i8_u),
    );

    // memory load i32
    add(
        "memory.load32_i32",
        InstructionSyntaxKind::MemoryLoad(Opcode::memory_load32_i32),
    );
    add(
        "memory.load32_f32",
        InstructionSyntaxKind::MemoryLoad(Opcode::memory_load32_f32),
    );
    add(
        "memory.load32_i16_s",
        InstructionSyntaxKind::MemoryLoad(Opcode::memory_load32_i16_s),
    );
    add(
        "memory.load32_i16_u",
        InstructionSyntaxKind::MemoryLoad(Opcode::memory_load32_i16_u),
    );
    add(
        "memory.load32_i8_s",
        InstructionSyntaxKind::MemoryLoad(Opcode::memory_load32_i8_s),
    );
    add(
        "memory.load32_i8_u",
        InstructionSyntaxKind::MemoryLoad(Opcode::memory_load32_i8_u),
    );

    // memory store
    add(
        "memory.store64",
        InstructionSyntaxKind::MemoryStore(Opcode::memory_store64),
    );
    add(
        "memory.store32",
        InstructionSyntaxKind::MemoryStore(Opcode::memory_store32),
    );
    add(
        "memory.store16",
        InstructionSyntaxKind::MemoryStore(Opcode::memory_store16),
    );
    add(
        "memory.store8",
        InstructionSyntaxKind::MemoryStore(Opcode::memory_store8),
    );

    // reduce i64 to i32
    add(
        "i32.truncate_i64",
        InstructionSyntaxKind::UnaryOp(Opcode::i32_truncate_i64),
    );

    // extend i32 to i64
    add(
        "i64.extend_i32_s",
        InstructionSyntaxKind::UnaryOp(Opcode::i64_extend_i32_s),
    );
    add(
        "i64.extend_i32_u",
        InstructionSyntaxKind::UnaryOp(Opcode::i64_extend_i32_u),
    );

    // float demote and promote
    add(
        "f32.demote_f64",
        InstructionSyntaxKind::UnaryOp(Opcode::f32_demote_f64),
    );
    add(
        "f64.promote_f32",
        InstructionSyntaxKind::UnaryOp(Opcode::f64_promote_f32),
    );

    // convert float to int
    add(
        "i32.convert_f32_s",
        InstructionSyntaxKind::UnaryOp(Opcode::i32_convert_f32_s),
    );
    add(
        "i32.convert_f32_u",
        InstructionSyntaxKind::UnaryOp(Opcode::i32_convert_f32_u),
    );
    add(
        "i32.convert_f64_s",
        InstructionSyntaxKind::UnaryOp(Opcode::i32_convert_f64_s),
    );
    add(
        "i32.convert_f64_u",
        InstructionSyntaxKind::UnaryOp(Opcode::i32_convert_f64_u),
    );
    add(
        "i64.convert_f32_s",
        InstructionSyntaxKind::UnaryOp(Opcode::i64_convert_f32_s),
    );
    add(
        "i64.convert_f32_u",
        InstructionSyntaxKind::UnaryOp(Opcode::i64_convert_f32_u),
    );
    add(
        "i64.convert_f64_s",
        InstructionSyntaxKind::UnaryOp(Opcode::i64_convert_f64_s),
    );
    add(
        "i64.convert_f64_u",
        InstructionSyntaxKind::UnaryOp(Opcode::i64_convert_f64_u),
    );

    // convert int to float
    add(
        "f32.convert_i32_s",
        InstructionSyntaxKind::UnaryOp(Opcode::f32_convert_i32_s),
    );
    add(
        "f32.convert_i32_u",
        InstructionSyntaxKind::UnaryOp(Opcode::f32_convert_i32_u),
    );
    add(
        "f32.convert_i64_s",
        InstructionSyntaxKind::UnaryOp(Opcode::f32_convert_i64_s),
    );
    add(
        "f32.convert_i64_u",
        InstructionSyntaxKind::UnaryOp(Opcode::f32_convert_i64_u),
    );
    add(
        "f64.convert_i32_s",
        InstructionSyntaxKind::UnaryOp(Opcode::f64_convert_i32_s),
    );
    add(
        "f64.convert_i32_u",
        InstructionSyntaxKind::UnaryOp(Opcode::f64_convert_i32_u),
    );
    add(
        "f64.convert_i64_s",
        InstructionSyntaxKind::UnaryOp(Opcode::f64_convert_i64_s),
    );
    add(
        "f64.convert_i64_u",
        InstructionSyntaxKind::UnaryOp(Opcode::f64_convert_i64_u),
    );

    // saturation convert float to int
    add(
        "i32.sat_convert_f32_s",
        InstructionSyntaxKind::UnaryOp(Opcode::i32_sat_convert_f32_s),
    );
    add(
        "i32.sat_convert_f32_u",
        InstructionSyntaxKind::UnaryOp(Opcode::i32_sat_convert_f32_u),
    );
    add(
        "i32.sat_convert_f64_s",
        InstructionSyntaxKind::UnaryOp(Opcode::i32_sat_convert_f64_s),
    );
    add(
        "i32.sat_convert_f64_u",
        InstructionSyntaxKind::UnaryOp(Opcode::i32_sat_convert_f64_u),
    );
    add(
        "i64.sat_convert_f32_s",
        InstructionSyntaxKind::UnaryOp(Opcode::i64_sat_convert_f32_s),
    );
    add(
        "i64.sat_convert_f32_u",
        InstructionSyntaxKind::UnaryOp(Opcode::i64_sat_convert_f32_u),
    );
    add(
        "i64.sat_convert_f64_s",
        InstructionSyntaxKind::UnaryOp(Opcode::i64_sat_convert_f64_s),
    );
    add(
        "i64.sat_convert_f64_u",
        InstructionSyntaxKind::UnaryOp(Opcode::i64_sat_convert_f64_u),
    );

    // reinterpret
    add(
        "i32.reinterpret_f32",
        InstructionSyntaxKind::UnaryOp(Opcode::i32_reinterpret_f32),
    );
    add(
        "i64.reinterpret_f64",
        InstructionSyntaxKind::UnaryOp(Opcode::i64_reinterpret_f64),
    );
    add(
        "f32.reinterpret_i32",
        InstructionSyntaxKind::UnaryOp(Opcode::f32_reinterpret_i32),
    );
    add(
        "f64.reinterpret_i64",
        InstructionSyntaxKind::UnaryOp(Opcode::f64_reinterpret_i64),
    );

    // comparsion i32
    add("i32.eqz", InstructionSyntaxKind::UnaryOp(Opcode::i32_eqz)); // UnaryOp
    add("i32.nez", InstructionSyntaxKind::UnaryOp(Opcode::i32_nez)); // UnaryOp
    add("i32.eq", InstructionSyntaxKind::BinaryOp(Opcode::i32_eq));
    add("i32.ne", InstructionSyntaxKind::BinaryOp(Opcode::i32_ne));
    add(
        "i32.lt_s",
        InstructionSyntaxKind::BinaryOp(Opcode::i32_lt_s),
    );
    add(
        "i32.lt_u",
        InstructionSyntaxKind::BinaryOp(Opcode::i32_lt_u),
    );
    add(
        "i32.gt_s",
        InstructionSyntaxKind::BinaryOp(Opcode::i32_gt_s),
    );
    add(
        "i32.gt_u",
        InstructionSyntaxKind::BinaryOp(Opcode::i32_gt_u),
    );
    add(
        "i32.le_s",
        InstructionSyntaxKind::BinaryOp(Opcode::i32_le_s),
    );
    add(
        "i32.le_u",
        InstructionSyntaxKind::BinaryOp(Opcode::i32_le_u),
    );
    add(
        "i32.ge_s",
        InstructionSyntaxKind::BinaryOp(Opcode::i32_ge_s),
    );
    add(
        "i32.ge_u",
        InstructionSyntaxKind::BinaryOp(Opcode::i32_ge_u),
    );

    // comparsion i64
    add("i64.eqz", InstructionSyntaxKind::UnaryOp(Opcode::i64_eqz)); // UnaryOp
    add("i64.nez", InstructionSyntaxKind::UnaryOp(Opcode::i64_nez)); // UnaryOp
    add("i64.eq", InstructionSyntaxKind::BinaryOp(Opcode::i64_eq));
    add("i64.ne", InstructionSyntaxKind::BinaryOp(Opcode::i64_ne));
    add(
        "i64.lt_s",
        InstructionSyntaxKind::BinaryOp(Opcode::i64_lt_s),
    );
    add(
        "i64.lt_u",
        InstructionSyntaxKind::BinaryOp(Opcode::i64_lt_u),
    );
    add(
        "i64.gt_s",
        InstructionSyntaxKind::BinaryOp(Opcode::i64_gt_s),
    );
    add(
        "i64.gt_u",
        InstructionSyntaxKind::BinaryOp(Opcode::i64_gt_u),
    );
    add(
        "i64.le_s",
        InstructionSyntaxKind::BinaryOp(Opcode::i64_le_s),
    );
    add(
        "i64.le_u",
        InstructionSyntaxKind::BinaryOp(Opcode::i64_le_u),
    );
    add(
        "i64.ge_s",
        InstructionSyntaxKind::BinaryOp(Opcode::i64_ge_s),
    );
    add(
        "i64.ge_u",
        InstructionSyntaxKind::BinaryOp(Opcode::i64_ge_u),
    );

    // comparsion f32
    add("f32.eq", InstructionSyntaxKind::BinaryOp(Opcode::f32_eq));
    add("f32.ne", InstructionSyntaxKind::BinaryOp(Opcode::f32_ne));
    add("f32.lt", InstructionSyntaxKind::BinaryOp(Opcode::f32_lt));
    add("f32.gt", InstructionSyntaxKind::BinaryOp(Opcode::f32_gt));
    add("f32.le", InstructionSyntaxKind::BinaryOp(Opcode::f32_le));
    add("f32.ge", InstructionSyntaxKind::BinaryOp(Opcode::f32_ge));

    // comparsion f64
    add("f64.eq", InstructionSyntaxKind::BinaryOp(Opcode::f64_eq));
    add("f64.ne", InstructionSyntaxKind::BinaryOp(Opcode::f64_ne));
    add("f64.lt", InstructionSyntaxKind::BinaryOp(Opcode::f64_lt));
    add("f64.gt", InstructionSyntaxKind::BinaryOp(Opcode::f64_gt));
    add("f64.le", InstructionSyntaxKind::BinaryOp(Opcode::f64_le));
    add("f64.ge", InstructionSyntaxKind::BinaryOp(Opcode::f64_ge));

    // arithmetic i32
    add("i32.add", InstructionSyntaxKind::BinaryOp(Opcode::i32_add));
    add("i32.sub", InstructionSyntaxKind::BinaryOp(Opcode::i32_sub));
    add("i32.mul", InstructionSyntaxKind::BinaryOp(Opcode::i32_mul));
    add(
        "i32.mul_hi_s",
        InstructionSyntaxKind::BinaryOp(Opcode::i32_mul_hi_s),
    );
    add(
        "i32.mul_hi_u",
        InstructionSyntaxKind::BinaryOp(Opcode::i32_mul_hi_u),
    );
    add(
        "i32.div_s",
        InstructionSyntaxKind::BinaryOp(Opcode::i32_div_s),
    );
    add(
        "i32.div_u",
        InstructionSyntaxKind::BinaryOp(Opcode::i32_div_u),
    );
    add(
        "i32.rem_s",
        InstructionSyntaxKind::BinaryOp(Opcode::i32_rem_s),
    );
    add(
        "i32.rem_u",
        InstructionSyntaxKind::BinaryOp(Opcode::i32_rem_u),
    );
    add(
        "i32.inc",
        InstructionSyntaxKind::UnaryOpWithImmI64(Opcode::i32_inc),
    ); // UnaryOpParamI16
    add(
        "i32.dec",
        InstructionSyntaxKind::UnaryOpWithImmI64(Opcode::i32_dec),
    ); // UnaryOpParamI16

    // arithmetic i64
    add("i64.add", InstructionSyntaxKind::BinaryOp(Opcode::i64_add));
    add("i64.sub", InstructionSyntaxKind::BinaryOp(Opcode::i64_sub));
    add("i64.mul", InstructionSyntaxKind::BinaryOp(Opcode::i64_mul));
    add(
        "i64.mul_hi_s",
        InstructionSyntaxKind::BinaryOp(Opcode::i64_mul_hi_s),
    );
    add(
        "i64.mul_hi_u",
        InstructionSyntaxKind::BinaryOp(Opcode::i64_mul_hi_u),
    );
    add(
        "i64.div_s",
        InstructionSyntaxKind::BinaryOp(Opcode::i64_div_s),
    );
    add(
        "i64.div_u",
        InstructionSyntaxKind::BinaryOp(Opcode::i64_div_u),
    );
    add(
        "i64.rem_s",
        InstructionSyntaxKind::BinaryOp(Opcode::i64_rem_s),
    );
    add(
        "i64.rem_u",
        InstructionSyntaxKind::BinaryOp(Opcode::i64_rem_u),
    );
    add(
        "i64.inc",
        InstructionSyntaxKind::UnaryOpWithImmI64(Opcode::i64_inc),
    ); // UnaryOpParamI16
    add(
        "i64.dec",
        InstructionSyntaxKind::UnaryOpWithImmI64(Opcode::i64_dec),
    ); // UnaryOpParamI16

    // arithmetic f32
    add("f32.add", InstructionSyntaxKind::BinaryOp(Opcode::f32_add));
    add("f32.sub", InstructionSyntaxKind::BinaryOp(Opcode::f32_sub));
    add("f32.mul", InstructionSyntaxKind::BinaryOp(Opcode::f32_mul));
    add("f32.div", InstructionSyntaxKind::BinaryOp(Opcode::f32_div));

    // arithmetic f64
    add("f64.add", InstructionSyntaxKind::BinaryOp(Opcode::f64_add));
    add("f64.sub", InstructionSyntaxKind::BinaryOp(Opcode::f64_sub));
    add("f64.mul", InstructionSyntaxKind::BinaryOp(Opcode::f64_mul));
    add("f64.div", InstructionSyntaxKind::BinaryOp(Opcode::f64_div));

    // bitwise i32
    add("i32.and", InstructionSyntaxKind::BinaryOp(Opcode::i32_and));
    add("i32.or", InstructionSyntaxKind::BinaryOp(Opcode::i32_or));
    add("i32.xor", InstructionSyntaxKind::BinaryOp(Opcode::i32_xor));
    add(
        "i32.shift_left",
        InstructionSyntaxKind::BinaryOp(Opcode::i32_shift_left),
    );
    add(
        "i32.shift_right_s",
        InstructionSyntaxKind::BinaryOp(Opcode::i32_shift_right_s),
    );
    add(
        "i32.shift_right_u",
        InstructionSyntaxKind::BinaryOp(Opcode::i32_shift_right_u),
    );
    add(
        "i32.rotate_left",
        InstructionSyntaxKind::BinaryOp(Opcode::i32_rotate_left),
    );
    add(
        "i32.rotate_right",
        InstructionSyntaxKind::BinaryOp(Opcode::i32_rotate_right),
    );
    add("i32.not", InstructionSyntaxKind::UnaryOp(Opcode::i32_not)); // UnaryOp
    add(
        "i32.leading_zeros",
        InstructionSyntaxKind::UnaryOp(Opcode::i32_leading_zeros),
    ); // UnaryOp
    add(
        "i32.leading_ones",
        InstructionSyntaxKind::UnaryOp(Opcode::i32_leading_ones),
    ); // UnaryOp
    add(
        "i32.trailing_zeros",
        InstructionSyntaxKind::UnaryOp(Opcode::i32_trailing_zeros),
    ); // UnaryOp
    add(
        "i32.count_ones",
        InstructionSyntaxKind::UnaryOp(Opcode::i32_count_ones),
    ); // UnaryOp

    // bitwise i64
    add("i64.and", InstructionSyntaxKind::BinaryOp(Opcode::i64_and));
    add("i64.or", InstructionSyntaxKind::BinaryOp(Opcode::i64_or));
    add("i64.xor", InstructionSyntaxKind::BinaryOp(Opcode::i64_xor));
    add(
        "i64.shift_left",
        InstructionSyntaxKind::BinaryOp(Opcode::i64_shift_left),
    );
    add(
        "i64.shift_right_s",
        InstructionSyntaxKind::BinaryOp(Opcode::i64_shift_right_s),
    );
    add(
        "i64.shift_right_u",
        InstructionSyntaxKind::BinaryOp(Opcode::i64_shift_right_u),
    );
    add(
        "i64.rotate_left",
        InstructionSyntaxKind::BinaryOp(Opcode::i64_rotate_left),
    );
    add(
        "i64.rotate_right",
        InstructionSyntaxKind::BinaryOp(Opcode::i64_rotate_right),
    );
    add("i64.not", InstructionSyntaxKind::UnaryOp(Opcode::i64_not)); // UnaryOp
    add(
        "i64.leading_zeros",
        InstructionSyntaxKind::UnaryOp(Opcode::i64_leading_zeros),
    ); // UnaryOp
    add(
        "i64.leading_ones",
        InstructionSyntaxKind::UnaryOp(Opcode::i64_leading_ones),
    ); // UnaryOp
    add(
        "i64.trailing_zeros",
        InstructionSyntaxKind::UnaryOp(Opcode::i64_trailing_zeros),
    ); // UnaryOp
    add(
        "i64.count_ones",
        InstructionSyntaxKind::UnaryOp(Opcode::i64_count_ones),
    ); // UnaryOp

    // math i32
    add("i32.abs", InstructionSyntaxKind::UnaryOp(Opcode::i32_abs));
    add("i32.neg", InstructionSyntaxKind::UnaryOp(Opcode::i32_neg));

    // math i64
    add("i64.abs", InstructionSyntaxKind::UnaryOp(Opcode::i64_abs));
    add("i64.neg", InstructionSyntaxKind::UnaryOp(Opcode::i64_neg));

    // math f32
    add("f32.abs", InstructionSyntaxKind::UnaryOp(Opcode::f32_abs));
    add("f32.neg", InstructionSyntaxKind::UnaryOp(Opcode::f32_neg));
    add("f32.ceil", InstructionSyntaxKind::UnaryOp(Opcode::f32_ceil));
    add(
        "f32.floor",
        InstructionSyntaxKind::UnaryOp(Opcode::f32_floor),
    );
    add(
        "f32.round_half_to_even",
        InstructionSyntaxKind::UnaryOp(Opcode::f32_round_half_to_even),
    );
    add(
        "f32.trunc",
        InstructionSyntaxKind::UnaryOp(Opcode::f32_trunc),
    );
    add("f32.sqrt", InstructionSyntaxKind::UnaryOp(Opcode::f32_sqrt));
    add(
        "f32.copysign",
        InstructionSyntaxKind::BinaryOp(Opcode::f32_copysign),
    ); // BinaryOp
    add("f32.min", InstructionSyntaxKind::BinaryOp(Opcode::f32_min)); // BinaryOp
    add("f32.max", InstructionSyntaxKind::BinaryOp(Opcode::f32_max)); // BinaryOp

    // math f64
    add("f64.abs", InstructionSyntaxKind::UnaryOp(Opcode::f64_abs));
    add("f64.neg", InstructionSyntaxKind::UnaryOp(Opcode::f64_neg));
    add("f64.ceil", InstructionSyntaxKind::UnaryOp(Opcode::f64_ceil));
    add(
        "f64.floor",
        InstructionSyntaxKind::UnaryOp(Opcode::f64_floor),
    );
    add(
        "f64.round_half_to_even",
        InstructionSyntaxKind::UnaryOp(Opcode::f64_round_half_to_even),
    );
    add(
        "f64.trunc",
        InstructionSyntaxKind::UnaryOp(Opcode::f64_trunc),
    );

    add("f64.sqrt", InstructionSyntaxKind::UnaryOp(Opcode::f64_sqrt));
    add(
        "f64.copysign",
        InstructionSyntaxKind::BinaryOp(Opcode::f64_copysign),
    ); // BinaryOp
    add("f64.min", InstructionSyntaxKind::BinaryOp(Opcode::f64_min)); // BinaryOp
    add("f64.max", InstructionSyntaxKind::BinaryOp(Opcode::f64_max)); // BinaryOp

    // machine
    add("trap", InstructionSyntaxKind::Trap);

    // memory address
    add(
        "addr.local",
        InstructionSyntaxKind::LocalLoad(Opcode::addr_local),
    );
    add(
        "addr.data",
        InstructionSyntaxKind::DataLoad(Opcode::addr_data),
    );
    add(
        "addr.thread_local_data",
        InstructionSyntaxKind::DataLoad(Opcode::addr_thread_local_data),
    );
    add("addr.function", InstructionSyntaxKind::AddrFunction);

    // atomic i32
    add(
        "i32.atomic_rmw_add",
        InstructionSyntaxKind::AtomicRmw(Opcode::i32_atomic_rmw_add),
    );
    add(
        "i32.atomic_rmw_sub",
        InstructionSyntaxKind::AtomicRmw(Opcode::i32_atomic_rmw_sub),
    );
    add(
        "i32.atomic_rmw_and",
        InstructionSyntaxKind::AtomicRmw(Opcode::i32_atomic_rmw_and),
    );
    add(
        "i32.atomic_rmw_or",
        InstructionSyntaxKind::AtomicRmw(Opcode::i32_atomic_rmw_or),
    );
    add(
        "i32.atomic_rmw_xor",
        InstructionSyntaxKind::AtomicRmw(Opcode::i32_atomic_rmw_xor),
    );
    add(
        "i32.atomic_rmw_exchange",
        InstructionSyntaxKind::AtomicRmw(Opcode::i32_atomic_rmw_exchange),
    );
    add("i32.atomic_cas", InstructionSyntaxKind::AtomicCas);

    // atomic i64
    add(
        "i64.atomic_rmw_add",
        InstructionSyntaxKind::AtomicRmw(Opcode::i64_atomic_rmw_add),
    );
    add(
        "i64.atomic_rmw_sub",
        InstructionSyntaxKind::AtomicRmw(Opcode::i64_atomic_rmw_sub),
    );
    add(
        "i64.atomic_rmw_and",
        InstructionSyntaxKind::AtomicRmw(Opcode::i64_atomic_rmw_and),
    );
    add(
        "i64.atomic_rmw_or",
        InstructionSyntaxKind::AtomicRmw(Opcode::i64_atomic_rmw_or),
    );
    add(
        "i64.atomic_rmw_xor",
        InstructionSyntaxKind::AtomicRmw(Opcode::i64_atomic_rmw_xor),
    );
    add(
        "i64.atomic_rmw_exchange",
        InstructionSyntaxKind::AtomicRmw(Opcode::i64_atomic_rmw_exchange),
    );
    add("i64.atomic_cas", InstructionSyntaxKind::AtomicCas);

    // pesudo instructions
    add("i32.imm", InstructionSyntaxKind::ImmI32);
    add("i64.imm", InstructionSyntaxKind::ImmI64);
    add("f32.imm", InstructionSyntaxKind::ImmF32);
    add("f64.imm", InstructionSyntaxKind::ImmF64);

    add("when", InstructionSyntaxKind::When);
    add("if", InstructionSyntaxKind::If);
    add("branch", InstructionSyntaxKind::Branch);
    add("for", InstructionSyntaxKind::For);

    add("do", InstructionSyntaxKind::Sequence("do"));
    add("break", InstructionSyntaxKind::Sequence("break"));
    add("return", InstructionSyntaxKind::Sequence("return"));
    add("recur", InstructionSyntaxKind::Sequence("recur"));
    add("rerun", InstructionSyntaxKind::Sequence("rerun"));

    add("call", InstructionSyntaxKind::Call);
    add("dyncall", InstructionSyntaxKind::DynCall);
    add("syscall", InstructionSyntaxKind::SysCall);

    unsafe { INSTRUCTION_MAP = Some(table) };
}
