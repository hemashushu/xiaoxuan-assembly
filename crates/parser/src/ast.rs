// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use anna_types::{opcode::Opcode, DataType, MemoryDataType, ModuleShareType};

#[derive(Debug, PartialEq)]
pub struct ModuleNode {
    // the name of main module or submodule
    //
    // note that the module names within an application (or a module) can not be duplicated
    pub name_path: String,

    pub compiler_version_major: u16,
    pub compiler_version_minor: u16,

    // the relative name path of constructor function
    pub constructor_function_name_path: Option<String>,

    // the relative name path of destructor function
    pub destructor_function_name_path: Option<String>,

    pub element_nodes: Vec<ModuleElementNode>,
}

#[derive(Debug, PartialEq)]
pub enum ModuleElementNode {
    FunctionNode(FunctionNode),
    DataNode(DataNode),

    // for using the external (C-lang) functions or data
    ExternalNode(ExternalNode),

    // for using the functions or data of other XiaoXuan Native shared modules
    ImportNode(ImportNode),
}

#[derive(Debug, PartialEq)]
pub struct FunctionNode {
    // nate that the names of functions can not be duplicated within a module,
    // including the name of imported functions.
    pub name: String,

    pub export: bool,
    pub convention: Option<String>,
    pub export_name: Option<String>,

    pub params: Vec<ParamNode>,
    pub results: Vec<DataType>,
    pub locals: Vec<LocalNode>,
    pub code: Vec<Instruction>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ParamNode {
    // nate that the names of all parameters and local variables within a function
    // can not be duplicated.
    pub name: String,
    pub data_type: DataType,
}

#[derive(Debug, PartialEq, Clone)]
pub struct LocalNode {
    // nate that the names of all parameters and local variables within a function
    // can not be duplicated.
    pub name: String,

    pub memory_data_type: MemoryDataType,
    pub data_length: u32,
    // pub align: u64,
}

#[derive(Debug, PartialEq)]
pub struct ExternalNode {
    pub external_items: Vec<ExternalItem>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ExternalItem {
    ExternalFunction(ExternalFunctionNode),
    ExternalData(ExternalDataNode),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ExternalFunctionNode {
    pub id: String,   // the identifier of the external function for 'call' instruction
    pub name: String, // the original exported name/symbol
    pub params: Vec<DataType>, // the parameters of external functions have no identifier
    pub results: Vec<DataType>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ExternalDataNode {
    // the identifier of the imported data for data loading/storing instructions
    pub id: String,

    pub name: String, // the original exported name/symbol
    pub data_kind_node: SimplifiedDataKindNode,
}

#[derive(Debug, PartialEq)]
pub struct ImportNode {
    pub import_module_node: ImportModuleNode,
    pub import_items: Vec<ImportItem>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ImportItem {
    ImportFunction(ImportFunctionNode),
    ImportData(ImportDataNode),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ImportModuleNode {
    pub module_share_type: ModuleShareType,
    pub name: String,
    pub version_major: u16,
    pub version_minor: u16,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ImportFunctionNode {
    // the identifier of the imported function for calling instructons
    pub id: String,

    // the original exported name path,
    // includes the submodule name path, but excludes the module name.
    //
    // e.g.
    // the name path of functon 'add' in module 'myapp' is 'add',
    // the name path of function 'add' in submodule 'myapp:utils' is 'utils::add'.
    pub name_path: String,

    // the parameters of external functions have no identifier
    pub params: Vec<DataType>,
    pub results: Vec<DataType>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ImportDataNode {
    // the identifier of the imported data for data loading/storing instructions
    pub id: String,

    // the original exported name path,
    // includes the submodule name path, but excludes the module name.
    //
    // e.g.
    // the name path of data 'buf' in module 'myapp' is 'buf',
    // the name path of data 'buf' in submodule 'myapp:utils' is 'utils::buf'.
    pub name_path: String,
    pub data_kind_node: SimplifiedDataKindNode,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    ImmI32(u32),
    ImmI64(u64),
    ImmF32(f32),
    ImmF64(f64),

    LocalLoad {
        opcode: Opcode,
        name: String,
        offset: u32,
    },

    LocalStore {
        opcode: Opcode,
        name: String,
        offset: u32,
        value: Box<Instruction>,
    },

    DataLoad {
        opcode: Opcode,

        // the data identifier, or the (relative/absolute) name path
        id: String,
        offset: u32,
    },

    DataStore {
        opcode: Opcode,

        // the data identifier, or the (relative/absolute) name path
        id: String,
        offset: u32,
        value: Box<Instruction>,
    },

    MemoryLoad {
        opcode: Opcode,
        offset: u32,
        addr: Box<Instruction>,
    },

    MemoryStore {
        opcode: Opcode,
        offset: u32,
        addr: Box<Instruction>,
        value: Box<Instruction>,
    },

    UnaryOp {
        opcode: Opcode,
        source: Box<Instruction>,
    },

    UnaryOpWithImmI64 {
        opcode: Opcode,
        imm: u64,
        source: Box<Instruction>,
    },

    BinaryOp {
        opcode: Opcode,
        left: Box<Instruction>,
        right: Box<Instruction>,
    },

    AtomicRmw {
        opcode: Opcode,
        rmw_op: RmwOp,
        addr: Box<Instruction>,
        value: Box<Instruction>,
    },

    AtomicCas {
        addr: Box<Instruction>,
        expect_value: Box<Instruction>,
        new_value: Box<Instruction>,
    },

    When {
        // structure 'when' has NO params and NO results
        test: Box<Instruction>,
        consequent: Box<Instruction>,
    },

    If {
        // structure 'If' has NO params, but can return values.
        results: Vec<DataType>,
        test: Box<Instruction>,
        consequent: Box<Instruction>,
        alternate: Box<Instruction>,
    },

    Branch {
        // structure 'Branch' has NO params, but can return values.
        results: Vec<DataType>,
        cases: Vec<BranchCase>,

        // the branch 'default' is optional, but for the structure 'branch' with
        // return value(s), it SHOULD add instruction 'unreachable' follow the last branch
        // to avoid missing matches.
        default: Option<Box<Instruction>>,
    },

    For {
        params: Vec<ParamNode>,
        results: Vec<DataType>,
        code: Box<Instruction>,
    },

    Do(Vec<Instruction>),

    // to break the nearest 'for' structure
    Break(Vec<Instruction>),

    // to recur the nearest 'for' structure
    Recur(Vec<Instruction>),

    Return(Vec<Instruction>),
    Rerun(Vec<Instruction>),

    Call {
        // the function identifier (name), or the (relative/absolute) name path
        id: String,
        args: Vec<Instruction>,
    },

    DynCall {
        // the target function address
        addr: Box<Instruction>,
        args: Vec<Instruction>,
    },

    SysCall {
        num: u32,
        args: Vec<Instruction>,
    },

    Trap {
        code: u32,
    },

    AddrFunction {
        // the function identifier (name), or the (relative/absolute) name path
        id: String,
    },
}

// https://docs.rs/cranelift-codegen/latest/cranelift_codegen/ir/enum.AtomicRmwOp.html
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RmwOp {
    Add,
    Sub,
    And,
    Nand,
    Or,
    Xor,
    Exchange,
}

#[derive(Debug, PartialEq, Clone)]
pub struct BranchCase {
    pub test: Box<Instruction>,
    pub consequent: Box<Instruction>,
}

#[derive(Debug, PartialEq)]
pub struct DataNode {
    // the names of data can not be duplicated within a module,
    // including the name of imported data.
    pub name: String,
    pub export: bool,
    pub data_kind: DataKindNode,
}

#[derive(Debug, PartialEq, Clone)]
pub enum DataKindNode {
    ReadOnly(InitedData),
    ReadWrite(InitedData),
    Uninit(UninitData),
    ThreadLocalReadWrite(InitedData),
    ThreadLocalUninit(UninitData),
}

#[derive(Debug, PartialEq, Clone)]
pub struct InitedData {
    pub memory_data_type: MemoryDataType,
    pub length: usize,

    // if the data is a byte array (includes string), the value should be 1,
    // if the data is a struct, the value should be the max one of the length of its fields.
    // currently the MIN value is 1.
    pub align: u64,
    pub value: Vec<u8>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct UninitData {
    pub memory_data_type: MemoryDataType,
    pub length: usize,

    // if the data is a byte array (includes string), the value should be 1,
    // if the data is a struct, the value should be the max one of the length of its fields.
    // currently the MIN value is 1.
    pub align: u64,
}

// for imported data node
#[derive(Debug, PartialEq, Clone)]
pub enum SimplifiedDataKindNode {
    ReadOnly(MemoryDataType),
    ReadWrite(MemoryDataType),
    Uninit(MemoryDataType),
}
