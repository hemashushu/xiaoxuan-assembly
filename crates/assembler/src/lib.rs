// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

// mod object_file_test;
pub mod codegen;

use std::{
    any::Any,
    fmt::{Debug, Display},
};

// Semantic Versioning
// - https://semver.org/
//
// a module will only run if its required major and minor
// versions match the current runtime version 100%.
pub const COMPILER_MAJOR_VERSION: u16 = 1;
pub const COMPILER_MINOR_VERSION: u16 = 0;
pub const COMPILER_PATCH_VERSION: u16 = 0;

#[repr(u8)]
// https://doc.rust-lang.org/nomicon/other-reprs.html
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DataType {
    I32 = 0x0,
    I64,
    F32,
    F64,
}

/// the data type of
/// - local variables
/// - data in the DATA sections
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
pub enum DataSectionType {
    ReadOnly = 0x0,
    ReadWrite,
    Uninit,
}

impl Display for DataSectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            DataSectionType::ReadOnly => "read_only",
            DataSectionType::ReadWrite => "read_write",
            DataSectionType::Uninit => "uninit",
        };
        f.write_str(name)
    }
}

// for foreign function interface (FFI)
// that is, for calling function (in a module of the VM) from the outside,
// or returning values to the outside.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ForeignValue {
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
}

impl ForeignValue {
    pub fn as_u32(&self) -> u32 {
        match self {
            ForeignValue::U32(v) => *v,
            _ => panic!("The data type of the foreign value does not match."),
        }
    }

    pub fn as_u64(&self) -> u64 {
        match self {
            ForeignValue::U64(v) => *v,
            _ => panic!("The data type of the foreign value does not match."),
        }
    }

    pub fn as_f32(&self) -> f32 {
        match self {
            ForeignValue::F32(v) => *v,
            _ => panic!("The data type of the foreign value does not match."),
        }
    }

    pub fn as_f64(&self) -> f64 {
        match self {
            ForeignValue::F64(v) => *v,
            _ => panic!("The data type of the foreign value does not match."),
        }
    }
}

pub trait CompileError: Debug + Display + Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;
}
