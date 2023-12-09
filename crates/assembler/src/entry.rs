// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use crate::{DataSectionType, DataType, ExternalLibraryType, MemoryDataType, ModuleShareType};

#[derive(Debug)]
pub struct ModuleEntry {
    pub name: String,
    pub runtime_version_major: u16,
    pub runtime_version_minor: u16,

    pub import_function_count: usize,
    pub import_read_only_data_count: usize,
    pub import_read_write_data_count: usize,
    pub import_uninit_data_count: usize,

    pub constructor_function_public_index: Option<u32>,
    pub destructor_function_public_index: Option<u32>,

    pub type_entries: Vec<TypeEntry>,
    pub local_list_entries: Vec<LocalListEntry>,
    pub function_entries: Vec<FunctionEntry>,

    pub read_only_data_entries: Vec<InitedDataEntry>,
    pub read_write_data_entries: Vec<InitedDataEntry>,
    pub uninit_data_entries: Vec<UninitDataEntry>,

    pub external_library_entries: Vec<ExternalLibraryEntry>,
    pub external_function_entries: Vec<ExternalFunctionEntry>,

    // the dependencies
    pub import_module_entries: Vec<ImportModuleEntry>,

    // the import_function_entries, import_data_entries,
    // function_name_entries, data_name_entries are
    // used for linking.
    pub import_function_entries: Vec<ImportFunctionEntry>,
    pub import_data_entries: Vec<ImportDataEntry>,

    pub function_name_entries: Vec<FunctionNameEntry>,
    pub data_name_entries: Vec<DataNameEntry>,
}

#[derive(Debug)]
pub struct IndexEntry {
    // essential
    pub function_index_module_entries: Vec<FunctionIndexModuleEntry>,
    // optional
    pub data_index_module_entries: Vec<DataIndexModuleEntry>,
    pub unified_external_library_entries: Vec<UnifiedExternalLibraryEntry>,
    pub unified_external_function_entries: Vec<UnifiedExternalFunctionEntry>,
    pub external_function_index_module_entries: Vec<ExternalFunctionIndexModuleEntry>,
    // pub start_function_index_entries: Vec<ModuleFunctionIndexEntry>,
    // pub exit_function_index_entries: Vec<ModuleFunctionIndexEntry>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TypeEntry {
    pub params: Vec<DataType>,
    pub results: Vec<DataType>,
}

// both function and block can contains a 'local variables list'
#[derive(Debug, PartialEq, Clone)]
pub struct LocalListEntry {
    pub local_variable_entries: Vec<LocalVariableEntry>,
}

impl LocalListEntry {
    pub fn new(local_variable_entries: Vec<LocalVariableEntry>) -> Self {
        Self {
            local_variable_entries,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct LocalVariableEntry {
    pub memory_data_type: MemoryDataType,

    // actual length of the variable/data
    pub length: u32,

    pub align: u16,
}

impl LocalVariableEntry {
    pub fn from_i32() -> Self {
        Self {
            memory_data_type: MemoryDataType::I32,
            length: 4,
            align: 4,
        }
    }

    pub fn from_i64() -> Self {
        Self {
            memory_data_type: MemoryDataType::I64,
            length: 8,
            align: 8,
        }
    }

    pub fn from_f32() -> Self {
        Self {
            memory_data_type: MemoryDataType::F32,
            length: 4,
            align: 4,
        }
    }

    pub fn from_f64() -> Self {
        Self {
            memory_data_type: MemoryDataType::F64,
            length: 8,
            align: 8,
        }
    }

    pub fn from_bytes(length: u32, align: u16) -> Self {
        Self {
            memory_data_type: MemoryDataType::Bytes,
            length,
            align,
        }
    }

    pub fn from_datatype(datatype: DataType) -> Self {
        match datatype {
            DataType::I32 => Self::from_i32(),
            DataType::I64 => Self::from_i64(),
            DataType::F32 => Self::from_f32(),
            DataType::F64 => Self::from_f64(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct FunctionEntry {
    pub type_index: usize,
    pub local_list_index: usize,
    pub code: Vec<u8>,
}

#[derive(Debug, PartialEq)]
pub struct InitedDataEntry {
    pub memory_data_type: MemoryDataType,
    pub data: Vec<u8>,
    pub length: u32,
    pub align: u16, // should not be '0'
}

impl InitedDataEntry {
    /// note that 'i32' in function name means a 32-bit integer, which is equivalent to
    /// the 'uint32_t' in C or 'u32' in Rust. do not confuse it with 'i32' in Rust.
    /// the same applies to the i8, i16 and i64.
    pub fn from_i32(value: u32) -> Self {
        let mut data: Vec<u8> = Vec::with_capacity(8);
        data.extend(value.to_le_bytes().iter());

        Self {
            memory_data_type: MemoryDataType::I32,
            data,
            length: 4,
            align: 4,
        }
    }

    pub fn from_i64(value: u64) -> Self {
        let mut data: Vec<u8> = Vec::with_capacity(8);
        data.extend(value.to_le_bytes().iter());

        Self {
            memory_data_type: MemoryDataType::I64,
            data,
            length: 8,
            align: 8,
        }
    }

    pub fn from_f32(value: f32) -> Self {
        let mut data: Vec<u8> = Vec::with_capacity(8);
        data.extend(value.to_le_bytes().iter());

        Self {
            memory_data_type: MemoryDataType::F32,
            data,
            length: 4,
            align: 4,
        }
    }

    pub fn from_f64(value: f64) -> Self {
        let mut data: Vec<u8> = Vec::with_capacity(8);
        data.extend(value.to_le_bytes().iter());

        Self {
            memory_data_type: MemoryDataType::F64,
            data,
            length: 8,
            align: 8,
        }
    }

    pub fn from_bytes(data: Vec<u8>, align: u16) -> Self {
        let length = data.len() as u32;

        Self {
            memory_data_type: MemoryDataType::Bytes,
            data,
            length,
            align,
        }
    }
}

#[derive(Debug)]
pub struct UninitDataEntry {
    pub memory_data_type: MemoryDataType,
    pub length: u32,
    pub align: u16, // should not be '0'
}

impl UninitDataEntry {
    pub fn from_i32() -> Self {
        Self {
            memory_data_type: MemoryDataType::I32,
            length: 4,
            align: 4,
        }
    }

    pub fn from_i64() -> Self {
        Self {
            memory_data_type: MemoryDataType::I64,
            length: 8,
            align: 8,
        }
    }

    pub fn from_f32() -> Self {
        Self {
            memory_data_type: MemoryDataType::F32,
            length: 4,
            align: 4,
        }
    }

    pub fn from_f64() -> Self {
        Self {
            memory_data_type: MemoryDataType::F64,
            length: 8,
            align: 8,
        }
    }

    pub fn from_bytes(length: u32, align: u16) -> Self {
        Self {
            memory_data_type: MemoryDataType::Bytes,
            length,
            align,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ExternalLibraryEntry {
    pub name: String,
    pub external_library_type: ExternalLibraryType,
}

impl ExternalLibraryEntry {
    pub fn new(name: String, external_library_type: ExternalLibraryType) -> Self {
        Self {
            name,
            external_library_type,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ExternalFunctionEntry {
    pub name: String,
    pub external_library_index: usize,
    pub type_index: usize,
}

impl ExternalFunctionEntry {
    pub fn new(name: String, external_library_index: usize, type_index: usize) -> Self {
        Self {
            name,
            external_library_index,
            type_index,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ImportModuleEntry {
    pub name: String,
    pub module_share_type: ModuleShareType,
    pub version_major: u16,
    pub version_minor: u16,
}

impl ImportModuleEntry {
    pub fn new(
        name: String,
        module_share_type: ModuleShareType,
        version_major: u16,
        version_minor: u16,
    ) -> Self {
        Self {
            name,
            module_share_type,
            version_major,
            version_minor,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ImportFunctionEntry {
    // the original exported name path,
    // includes the submodule name path, but excludes the module name.
    //
    // e.g.
    // the name path of functon 'add' in module 'myapp' is 'add',
    // the name path of function 'add' in submodule 'myapp:utils' is 'utils::add'.
    pub name_path: String,
    pub import_module_index: usize,
    pub type_index: usize, // used for validation when linking
}

impl ImportFunctionEntry {
    pub fn new(name_path: String, import_module_index: usize, type_index: usize) -> Self {
        Self {
            name_path,
            import_module_index,
            type_index,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ImportDataEntry {
    // the original exported name path,
    // includes the submodule name path, but excludes the module name.
    //
    // e.g.
    // the name path of data 'buf' in module 'myapp' is 'buf',
    // the name path of data 'buf' in submodule 'myapp:utils' is 'utils::buf'.
    pub name_path: String,
    pub import_module_index: usize,
    pub data_section_type: DataSectionType, // for validation when linking
    pub memory_data_type: MemoryDataType,   // for validation when linking
}

impl ImportDataEntry {
    pub fn new(
        name_path: String,
        import_module_index: usize,
        data_section_type: DataSectionType,
        memory_data_type: MemoryDataType,
    ) -> Self {
        Self {
            name_path,
            import_module_index,
            data_section_type,
            memory_data_type,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct FunctionNameEntry {
    // the exported name path,
    // includes the submodule name path, but excludes the module name.
    //
    // e.g.
    // the name path of functon 'add' in module 'myapp' is 'add',
    // the name path of function 'add' in submodule 'myapp:utils' is 'utils::add'.
    pub name_path: String,
    pub function_public_index: usize,
    pub export: bool,
}

impl FunctionNameEntry {
    pub fn new(name_path: String, function_public_index: usize, export: bool) -> Self {
        Self {
            name_path,
            function_public_index,
            export,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct DataNameEntry {
    // the exported name path,
    // includes the submodule name path, but excludes the module name.
    //
    // e.g.
    // the name path of data 'buf' in module 'myapp' is 'buf',
    // the name path of data 'buf' in submodule 'myapp:utils' is 'utils::buf'.
    pub name_path: String,
    pub data_public_index: usize,
    pub export: bool,
}

impl DataNameEntry {
    pub fn new(name_path: String, data_public_index: usize, export: bool) -> Self {
        Self {
            name_path,
            data_public_index,
            export,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct FunctionIndexEntry {
    pub function_public_index: usize,
    pub target_module_index: usize,
    pub function_internal_index: usize,
}

impl FunctionIndexEntry {
    pub fn new(
        function_public_index: usize,
        target_module_index: usize,
        function_internal_index: usize,
    ) -> Self {
        Self {
            function_public_index,
            target_module_index,
            function_internal_index,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct FunctionIndexModuleEntry {
    pub index_entries: Vec<FunctionIndexEntry>,
}

impl FunctionIndexModuleEntry {
    pub fn new(index_entries: Vec<FunctionIndexEntry>) -> Self {
        Self { index_entries }
    }
}

#[derive(Debug, PartialEq)]
pub struct DataIndexEntry {
    pub data_public_index: usize,
    pub target_module_index: usize,
    pub data_internal_index: usize,
    pub target_data_section_type: DataSectionType,
}

impl DataIndexEntry {
    pub fn new(
        data_public_index: usize,
        target_module_index: usize,
        data_internal_index: usize,
        target_data_section_type: DataSectionType,
    ) -> Self {
        Self {
            data_public_index,
            target_module_index,
            data_internal_index,
            target_data_section_type,
        }
    }
}

#[derive(Debug)]
pub struct DataIndexModuleEntry {
    pub index_entries: Vec<DataIndexEntry>,
}

impl DataIndexModuleEntry {
    pub fn new(index_entries: Vec<DataIndexEntry>) -> Self {
        Self { index_entries }
    }
}

#[derive(Debug, PartialEq)]
pub struct UnifiedExternalLibraryEntry {
    pub name: String,
    pub external_library_type: ExternalLibraryType,
}

impl UnifiedExternalLibraryEntry {
    pub fn new(name: String, external_library_type: ExternalLibraryType) -> Self {
        Self {
            name,
            external_library_type,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct UnifiedExternalFunctionEntry {
    pub name: String,
    pub unified_external_library_index: usize,
}

impl UnifiedExternalFunctionEntry {
    pub fn new(name: String, unified_external_library_index: usize) -> Self {
        Self {
            name,
            unified_external_library_index,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ExternalFunctionIndexEntry {
    pub external_function_index: usize,
    pub unified_external_function_index: usize,

    // copy the type_index from ExternalFuncSection of the specific module,
    // so that the ExternalFuncSection can be omitted at runtime.
    pub type_index: usize,
}

impl ExternalFunctionIndexEntry {
    pub fn new(
        external_function_index: usize,
        unified_external_function_index: usize,
        type_index: usize,
    ) -> Self {
        Self {
            external_function_index,
            unified_external_function_index,
            type_index,
        }
    }
}

#[derive(Debug)]
pub struct ExternalFunctionIndexModuleEntry {
    pub index_entries: Vec<ExternalFunctionIndexEntry>,
}

impl ExternalFunctionIndexModuleEntry {
    pub fn new(index_entries: Vec<ExternalFunctionIndexEntry>) -> Self {
        Self { index_entries }
    }
}

// #[derive(Debug, PartialEq)]
// pub struct ModuleFunctionIndexEntry {
//     pub module_index: usize,
//     pub function_public_index: usize,
// }
//
// impl ModuleFunctionIndexEntry {
//     pub fn new(module_index: usize, function_public_index: usize) -> Self {
//         Self {
//             module_index,
//             function_public_index,
//         }
//     }
// }
