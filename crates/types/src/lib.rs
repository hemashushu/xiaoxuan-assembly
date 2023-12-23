// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

pub mod opcode;

use std::{
    any::Any,
    fmt::{Debug, Display},
};

pub const COMPILER_MAJOR_VERSION: u16 = 1;
pub const COMPILER_MINOR_VERSION: u16 = 0;
pub const COMPILER_PATCH_VERSION: u16 = 0;

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DataType {
    I32 = 0x0,
    I64,
    F32,
    F64,
}

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MemoryDataType {
    I32 = 0x0,
    I64,
    F32,
    F64,
    Bytes,
}

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ModuleShareType {
    User = 0x0,
    Share,
}

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DataSectionType {
    ReadOnly = 0x0,
    ReadWrite,
    Uninit,
    ThreadLocalReadWrite,
    ThreadLocalUninit,
}

// impl Display for DataSectionType {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let name = match self {
//             DataSectionType::ReadOnly => "read_only",
//             DataSectionType::ReadWrite => "read_write",
//             DataSectionType::Uninit => "uninit",
//             DataSectionType::ThreadLocalReadWrite => "thread_local_read_write",
//             DataSectionType::ThreadLocalUninit => "thread_local_uninit",
//         };
//         f.write_str(name)
//     }
// }

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ForeignValue {
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
}

impl ForeignValue {
    pub fn as_u32(&self) -> Option<u32> {
        if let ForeignValue::U32(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        if let ForeignValue::U64(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_f32(&self) -> Option<f32> {
        if let ForeignValue::F32(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        if let ForeignValue::F64(v) = self {
            Some(*v)
        } else {
            None
        }
    }
}

pub trait CompileError: Debug + Display + Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;
}
