// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use cranelift_codegen::{
    isa,
    settings::{self, Configurable},
    Context,
};
use cranelift_frontend::FunctionBuilderContext;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{
    default_libcall_names, DataDescription, DataId, Linkage, Module, ModuleError,
};
use cranelift_object::{ObjectBuilder, ObjectModule};

// about the code generator Cranelift:
//
// - home: https://cranelift.dev/
// - source code: https://github.com/bytecodealliance/wasmtime/tree/main/cranelift
// - docs: https://github.com/bytecodealliance/wasmtime/blob/main/cranelift/docs/index.md
// - IR Reference: https://github.com/bytecodealliance/wasmtime/blob/main/cranelift/docs/ir.md
// - InstBuilder: https://docs.rs/cranelift-codegen/latest/cranelift_codegen/ir/trait.InstBuilder.html
// - Module: https://docs.rs/cranelift-module/latest/cranelift_module/trait.Module.html
// - cranelift_frontend: https://docs.rs/cranelift-frontend/latest/cranelift_frontend/

pub struct CodeGenerator<T>
where
    T: Module,
{
    pub module: T,
    pub context: Context,
    pub function_builder_context: FunctionBuilderContext,
    pub data_description: DataDescription,
}

impl CodeGenerator<JITModule> {
    // JITModule:
    // - source code: https://github.com/bytecodealliance/wasmtime/tree/main/cranelift/jit
    // - docs: https://docs.rs/cranelift-jit/latest/cranelift_jit/
    // - demo: https://github.com/bytecodealliance/wasmtime/blob/main/cranelift/jit/examples/jit-minimal.rs
    pub fn new_jit() -> Self {
        // all flags:
        // https://docs.rs/cranelift-codegen/0.100.0/cranelift_codegen/settings/struct.Flags.html
        let mut flag_builder = settings::builder();

        // Use colocated libcalls.
        // Generate code that assumes that libcalls can be declared “colocated”,
        // meaning they will be defined along with the current function,
        // such that they can use more efficient addressing.
        // https://docs.rs/cranelift-codegen/0.100.0/cranelift_codegen/settings/struct.Flags.html#method.use_colocated_libcalls
        flag_builder.set("use_colocated_libcalls", "false").unwrap();

        // Enable Position-Independent Code generation.
        // https://docs.rs/cranelift-codegen/0.100.0/cranelift_codegen/settings/struct.Flags.html#method.is_pic
        flag_builder.set("is_pic", "true").unwrap();

        // Optimization level for generated code.
        // Supported levels:
        // - none: Minimise compile time by disabling most optimizations.
        // - speed: Generate the fastest possible code
        // - speed_and_size: like “speed”, but also perform transformations aimed at reducing code size.
        // https://docs.rs/cranelift-codegen/0.100.0/cranelift_codegen/settings/struct.Flags.html#method.opt_level
        flag_builder.set("opt_level", "none").unwrap();

        // Preserve frame pointers
        // Preserving frame pointers – even inside leaf functions – makes it easy to capture
        // the stack of a running program, without requiring any side tables or
        // metadata (like .eh_frame sections).
        // Many sampling profilers and similar tools walk frame pointers to capture stacks.
        // Enabling this option will play nice with those tools.
        // https://docs.rs/cranelift-codegen/0.100.0/cranelift_codegen/settings/struct.Flags.html#method.preserve_frame_pointers
        flag_builder.set("preserve_frame_pointers", "true").unwrap();

        let isa_builder = cranelift_native::builder().unwrap_or_else(|msg| {
            panic!("host machine is not supported: {}", msg);
        });

        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .unwrap();

        let jit_builder = JITBuilder::with_isa(isa, default_libcall_names());

        // import external symbols
        // jit_builder.symbols(symbols);
        //
        // timport o single external symbol:
        // `jit_builder.symbol(name:String, ptr:*const u8)`

        let module = JITModule::new(jit_builder);
        let context = module.make_context();
        let function_builder_context = FunctionBuilderContext::new();
        let data_description = DataDescription::new();

        Self {
            module,
            context,
            function_builder_context,
            data_description,
        }
    }
}

impl CodeGenerator<ObjectModule> {
    // ObjectModule:
    // - source code: https://github.com/bytecodealliance/wasmtime/tree/main/cranelift/object
    // - docs: https://docs.rs/cranelift-object/latest/cranelift_object/
    // - demo: https://github.com/bytecodealliance/wasmtime/blob/main/cranelift/object/tests/basic.rs
    pub fn new_object_file(module_name: &str) -> Self {
        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false").unwrap();
        flag_builder.enable("is_pic").unwrap();
        flag_builder.set("opt_level", "none").unwrap();
        flag_builder.set("preserve_frame_pointers", "true").unwrap();

        let isa_builder =
            isa::lookup_by_name("x86_64-unknown-linux-gnu").unwrap_or_else(|lookup_error| {
                panic!("host machine is not supported: {}", lookup_error);
            });

        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .unwrap();

        let module = ObjectModule::new(
            ObjectBuilder::new(isa, module_name, default_libcall_names()).unwrap(),
        );

        let context = module.make_context();
        let function_builder_context = FunctionBuilderContext::new();
        let data_description = DataDescription::new();

        Self {
            module,
            context,
            function_builder_context,
            data_description,
        }
    }
}

impl<T> CodeGenerator<T>
where
    T: Module,
{
    // https://docs.rs/cranelift-module/latest/cranelift_module/struct.DataDescription.html
    pub fn define_inited_data(
        &mut self,
        name: &str,
        data: Vec<u8>,
        align: u64,
        linkage: Linkage,
        writable: bool,
        thread_local: bool,
    ) -> Result<DataId, ModuleError> {
        self.data_description.define(data.into_boxed_slice());
        self.data_description.set_align(align);
        let data_id = self
            .module
            .declare_data(name, linkage, writable, thread_local)?;
        self.module.define_data(data_id, &self.data_description)?;
        self.data_description.clear();

        Ok(data_id)
    }

    pub fn define_uninited_data(
        &mut self,
        name: &str,
        size: usize,
        align: u64,
        linkage: Linkage,
        thread_local: bool,
    ) -> Result<DataId, ModuleError> {
        self.data_description.define_zeroinit(size);
        self.data_description.set_align(align);
        let data_id = self
            .module
            .declare_data(name, linkage, true, thread_local)?;
        self.module.define_data(data_id, &self.data_description)?;
        self.data_description.clear();

        Ok(data_id)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::Write,
        process::{Command, ExitStatus},
    };

    use cranelift_codegen::ir::{
        types::{self},
        AbiParam, Function, InstBuilder, MemFlags, Type, UserFuncName,
    };
    use cranelift_frontend::FunctionBuilder;
    use cranelift_module::{Linkage, Module};

    use super::CodeGenerator;

    #[test]
    fn test_codegen_jit() {
        let mut generator = CodeGenerator::new_jit();

        // to get the pointer type (i32, i64 etc.):
        //
        // ```rust
        // let ptr_t: Type = generator.module.isa().pointer_type();
        // ```
        //
        // to create a signature:
        //
        // ```rust
        // let sig_main = Signature {
        //     params: vec![],
        //     returns: vec![AbiParam::new(types::I32)],
        //     call_conv: CallConv::SystemV,
        // };
        // ```
        //
        // about the calling convention:
        // https://docs.rs/cranelift-codegen/0.102.1/cranelift_codegen/ir/struct.Signature.html
        // https://docs.rs/cranelift-codegen/0.102.1/cranelift_codegen/isa/enum.CallConv.html
        //
        //
        // Name	Description
        // ----------------
        // fast         not-ABI-stable convention for best performance
        // cold         not-ABI-stable convention for infrequently executed code
        // system_v     System V-style convention used on many platforms
        // fastcall     Windows "fastcall" convention, also used for x64 and ARM

        let mut sig_main = generator.module.make_signature();
        sig_main.returns.push(AbiParam::new(types::I32));

        // the function 'main' should be 'export', so the linker can find it.
        let func_main_id = generator
            .module
            .declare_function("main", Linkage::Export, &sig_main)
            .unwrap();

        // about the 'declare_function':
        // https://docs.rs/cranelift-module/0.102.1/cranelift_module/trait.Module.html#tymethod.declare_function

        {
            let mut func = Function::with_name_signature(
                UserFuncName::user(0, func_main_id.as_u32()),
                sig_main,
            );

            let mut func_builder: FunctionBuilder = FunctionBuilder::new(
                // &mut generator.context.func,
                &mut func,
                &mut generator.function_builder_context,
            );
            let block = func_builder.create_block();
            func_builder.append_block_params_for_function_params(block);
            func_builder.switch_to_block(block);

            // return const 11
            let value_0 = func_builder.ins().iconst(types::I32, 11);
            func_builder.ins().return_(&[value_0]);

            func_builder.seal_all_blocks();
            func_builder.finalize();

            // generate the function code

            generator.context.func = func;

            generator
                .module
                .define_function(func_main_id, &mut generator.context)
                .unwrap();

            generator.module.clear_context(&mut generator.context);
        }

        // finish the module
        generator.module.finalize_definitions().unwrap();

        // get function pointers
        let func_main_ptr = generator.module.get_finalized_function(func_main_id);

        // cast ptr to Rust function
        let func_main: extern "C" fn() -> i32 = unsafe { std::mem::transmute(func_main_ptr) };

        assert_eq!(func_main(), 11);
    }

    #[test]
    fn test_codegen_object_file() {
        let mut generator = CodeGenerator::new_object_file("foo");

        let mut sig_main = generator.module.make_signature();
        sig_main.returns.push(AbiParam::new(types::I32));

        // the function 'main' should be 'export', so the linker can find it.
        let func_main_id = generator
            .module
            .declare_function("main", Linkage::Export, &sig_main)
            .unwrap();

        {
            let mut func = Function::with_name_signature(
                UserFuncName::user(0, func_main_id.as_u32()),
                sig_main,
            );

            let mut func_builder: FunctionBuilder = FunctionBuilder::new(
                // &mut generator.context.func,
                &mut func,
                &mut generator.function_builder_context,
            );
            let block = func_builder.create_block();
            func_builder.append_block_params_for_function_params(block);
            func_builder.switch_to_block(block);

            // return const 11
            let value_0 = func_builder.ins().iconst(types::I32, 11);
            func_builder.ins().return_(&[value_0]);

            func_builder.seal_all_blocks();
            func_builder.finalize();

            // generate the function code

            generator.context.func = func;

            generator
                .module
                .define_function(func_main_id, &mut generator.context)
                .unwrap();

            generator.module.clear_context(&mut generator.context);
        }

        // ObjectProduct:
        // https://docs.rs/cranelift-object/latest/cranelift_object/struct.ObjectProduct.html

        // finish the module
        let object_procduct = generator.module.finish();
        let module_binary = object_procduct.emit().unwrap();

        // write object file
        let object_file_path = get_temp_file_path("anna_code_gen_unit_test0.o");
        let mut file = File::create(&object_file_path).unwrap();
        file.write_all(&module_binary).unwrap();

        // link file
        let exec_file_path = get_temp_file_path("anna_code_gen_unit_test0.elf");
        link_object_file(&object_file_path, &exec_file_path).unwrap();

        // Run the executable file and get the exit code
        // `$ ./anna.elf`
        // `$ echo $?`
        let exit_code_opt = run_executable_file_and_get_exit_code(&exec_file_path);

        assert_eq!(exit_code_opt, Some(11));

        // clean up
        delete_file(&object_file_path);
        delete_file(&exec_file_path);
    }

    fn get_temp_file_path(filename: &str) -> String {
        let mut dir = std::env::temp_dir();
        dir.push(filename);
        dir.to_str().unwrap().to_owned()
    }

    fn link_object_file(object_file: &str, output_file: &str) -> std::io::Result<ExitStatus> {
        // link the object file with GCC:
        //
        // `$ gcc -o anna.elf anna.o`
        //
        // link the object file with binutils 'ld':
        //
        // ```sh
        // ld \
        //     -dynamic-linker /lib64/ld-linux-x86-64.so.2 \
        //     -pie \
        //     -o anna.elf \
        //     /usr/lib/Scrt1.o \
        //     /usr/lib/crti.o \
        //     -L/lib/ \
        //     -L/usr/lib \
        //     anna.o \
        //     -lc \
        //     /usr/lib/crtn.o
        // ```
        //
        // reference: the result of command `$ gcc -v -o anna.elf anna.o`

        // Mini FAQ about the misc libc/gcc crt files.
        // https://dev.gentoo.org/~vapier/crt.txt
        //
        // Some definitions:
        // - PIC - position independent code (-fPIC)
        // - PIE - position independent executable (-fPIE -pie)
        // - crt - C runtime
        //
        // - crt0.o crt1.o etc...
        //   Some systems use crt0.o, while some use crt1.o (and a few even use crt2.o
        //   or higher).  Most likely due to a transitionary phase that some targets
        //   went through.  The specific number is otherwise entirely arbitrary -- look
        //   at the internal gcc port code to figure out what your target expects.  All
        //   that matters is that whatever gcc has encoded, your C library better use
        //   the same name.
        //
        //   This object is expected to contain the _start symbol which takes care of
        //   bootstrapping the initial execution of the program.  What exactly that
        //   entails is highly libc dependent and as such, the object is provided by
        //   the C library and cannot be mixed with other ones.
        //
        //   On uClibc/glibc systems, this object initializes very early ABI requirements
        //   (like the stack or frame pointer), setting up the argc/argv/env values, and
        //   then passing pointers to the init/fini/main funcs to the internal libc main
        //   which in turn does more general bootstrapping before finally calling the real
        //   main function.
        //
        //   glibc ports call this file 'start.S' while uClibc ports call this crt0.S or
        //   crt1.S (depending on what their gcc expects).
        //
        // - crti.o
        //   Defines the function prologs for the .init and .fini sections (with the _init
        //   and _fini symbols respectively).  This way they can be called directly.  These
        //   symbols also trigger the linker to generate DT_INIT/DT_FINI dynamic ELF tags.
        //
        //   These are to support the old style constructor/destructor system where all
        //   .init/.fini sections get concatenated at link time.  Not to be confused with
        //   newer prioritized constructor/destructor .init_array/.fini_array sections and
        //   DT_INIT_ARRAY/DT_FINI_ARRAY ELF tags.
        //
        //   glibc ports used to call this 'initfini.c', but now use 'crti.S'.  uClibc
        //   also uses 'crti.S'.
        //
        // - crtn.o
        //   Defines the function epilogs for the .init/.fini sections.  See crti.o.
        //
        //   glibc ports used to call this 'initfini.c', but now use 'crtn.S'.  uClibc
        //   also uses 'crtn.S'.
        //
        // - Scrt1.o
        //   Used in place of crt1.o when generating PIEs.
        // - gcrt1.o
        //   Used in place of crt1.o when generating code with profiling information.
        //   Compile with -pg.  Produces output suitable for the gprof util.
        // - Mcrt1.o
        //   Like gcrt1.o, but is used with the prof utility.  glibc installs this as
        //   a dummy file as it's useless on linux systems.
        //
        // - crtbegin.o
        //   GCC uses this to find the start of the constructors.
        // - crtbeginS.o
        //   Used in place of crtbegin.o when generating shared objects/PIEs.
        // - crtbeginT.o
        //   Used in place of crtbegin.o when generating static executables.
        // - crtend.o
        //   GCC uses this to find the start of the destructors.
        // - crtendS.o
        //   Used in place of crtend.o when generating shared objects/PIEs.
        //
        // General linking order:
        // ```
        // crt1.o crti.o crtbegin.o
        //     [-L paths] [user objects] [gcc libs] [C libs] [gcc libs]
        //     crtend.o crtn.o
        // ```
        //
        // More references:
        // - http://gcc.gnu.org/onlinedocs/gccint/Initialization.html
        // - https://stackoverflow.com/a/16436294/23069938
        //
        // file 'Scrt1.o' is owned by package 'glibc', check:
        // `$ pacman -Qo Scrt1.o`
        // `$ pacman -Ql glibc | grep crt`

        Command::new("/usr/bin/ld")
            .arg("--dynamic-linker")
            .arg("/lib64/ld-linux-x86-64.so.2")
            .arg("-pie")
            .arg("-o")
            .arg(output_file)
            .arg("/usr/lib/Scrt1.o")
            .arg("/usr/lib/crti.o")
            .arg("-L/lib/")
            .arg("-L/usr/lib")
            .arg(object_file)
            .arg("-lc")
            .arg("/usr/lib/crtn.o")
            .status()
    }

    fn run_executable_file_and_get_exit_code(exec_file: &str) -> Option<i32> {
        Command::new(exec_file).status().unwrap().code()
    }

    fn delete_file(filepath: &str) {
        std::fs::remove_file(filepath).unwrap();
    }

    #[test]
    fn test_codegen_function_call() {
        let mut generator = CodeGenerator::new_jit();

        let mut sig_swap = generator.module.make_signature();
        sig_swap.params.push(AbiParam::new(types::I32));
        sig_swap.params.push(AbiParam::new(types::I32));
        sig_swap.returns.push(AbiParam::new(types::I32));

        let func_swap_id = generator
            .module
            .declare_function("swap", Linkage::Export, &sig_swap)
            .unwrap();

        {
            let mut func = Function::with_name_signature(
                UserFuncName::user(0, func_main_id.as_u32()),
                sig_main,
            );

            let mut func_builder: FunctionBuilder = FunctionBuilder::new(
                // &mut generator.context.func,
                &mut func,
                &mut generator.function_builder_context,
            );
            let block = func_builder.create_block();
            func_builder.append_block_params_for_function_params(block);
            func_builder.switch_to_block(block);

            // return const 11
            let value_0 = func_builder.ins().iconst(types::I32, 11);
            func_builder.ins().return_(&[value_0]);

            func_builder.seal_all_blocks();
            func_builder.finalize();

            // generate the function code

            generator.context.func = func;

            generator
                .module
                .define_function(func_main_id, &mut generator.context)
                .unwrap();

            generator.module.clear_context(&mut generator.context);
        }

        // -------------------------

        let mut sig_main = generator.module.make_signature();
        sig_main.returns.push(AbiParam::new(types::I32));

        // the function 'main' should be 'export', so the linker can find it.
        let func_main_id = generator
            .module
            .declare_function("main", Linkage::Export, &sig_main)
            .unwrap();

        {
            let mut func = Function::with_name_signature(
                UserFuncName::user(0, func_main_id.as_u32()),
                sig_main,
            );

            let mut func_builder: FunctionBuilder = FunctionBuilder::new(
                // &mut generator.context.func,
                &mut func,
                &mut generator.function_builder_context,
            );
            let block = func_builder.create_block();
            func_builder.append_block_params_for_function_params(block);
            func_builder.switch_to_block(block);

            // return const 11
            let value_0 = func_builder.ins().iconst(types::I32, 11);
            func_builder.ins().return_(&[value_0]);

            func_builder.seal_all_blocks();
            func_builder.finalize();

            // generate the function code

            generator.context.func = func;

            generator
                .module
                .define_function(func_main_id, &mut generator.context)
                .unwrap();

            generator.module.clear_context(&mut generator.context);
        }

        // finish the module
        generator.module.finalize_definitions().unwrap();

        // get function pointers
        let func_main_ptr = generator.module.get_finalized_function(func_main_id);

        // cast ptr to Rust function
        let func_main: extern "C" fn() -> i32 = unsafe { std::mem::transmute(func_main_ptr) };

        assert_eq!(func_main(), 11);
    }


    #[test]
    fn test_codegen_define_data() {
        let mut generator = CodeGenerator::new_jit();

        let ptr_t: Type = generator.module.isa().pointer_type();

        // define data
        let d0 = 13u32.to_le_bytes().to_vec();
        let d0_id = generator
            .define_inited_data("exit_code", d0, 4, Linkage::Local, false, false)
            .unwrap();

        // define function
        let mut sig_main = generator.module.make_signature();
        sig_main.returns.push(AbiParam::new(types::I32));

        // the function 'main' should be 'export', so the linker can find it.
        let func_main_id = generator
            .module
            .declare_function("main", Linkage::Export, &sig_main)
            .unwrap();

        {
            let mut func = Function::with_name_signature(
                UserFuncName::user(0, func_main_id.as_u32()),
                sig_main,
            );

            let gv_d0 = generator.module.declare_data_in_func(d0_id, &mut func);

            let mut func_builder: FunctionBuilder = FunctionBuilder::new(
                // &mut generator.context.func,
                &mut func,
                &mut generator.function_builder_context,
            );
            let block = func_builder.create_block();
            func_builder.append_block_params_for_function_params(block);
            func_builder.switch_to_block(block);

            let value_0_ptr = func_builder.ins().symbol_value(ptr_t, gv_d0);
            let value_0 = func_builder
                .ins()
                .load(types::I32, MemFlags::new(), value_0_ptr, 0);

            func_builder.ins().return_(&[value_0]);

            func_builder.seal_all_blocks();
            func_builder.finalize();

            // println!("{}", func.display());

            generator.context.func = func;

            generator
                .module
                .define_function(func_main_id, &mut generator.context)
                .unwrap();

            generator.module.clear_context(&mut generator.context);
        }

        // linking
        generator.module.finalize_definitions().unwrap();

        // get function pointers
        let func_main_ptr = generator.module.get_finalized_function(func_main_id);

        // example of to get data pointer
        //
        // ```rust
        // let (buf_ptr, buf_size) = generator.module.get_finalized_data(data_id);
        // let buf = unsafe { std::slice::from_raw_parts(buf_ptr, buf_size) };
        // ```
        //
        // note that the pointers of functions and data only available after 'module.finalize_definitions()'

        // cast ptr to Rust function
        let func_main: extern "C" fn() -> i32 = unsafe { std::mem::transmute(func_main_ptr) };

        assert_eq!(func_main(), 13);
    }
}
