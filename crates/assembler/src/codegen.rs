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
        condcodes::IntCC,
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
        // let addr_t: Type = generator.module.isa().pointer_type();
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
        let mut generator = CodeGenerator::new_object_file("main");

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
        let exit_code_opt = run_executable_binary_and_get_exit_code(
            &module_binary,
            "anna_unit_test_codegen_object_file",
        );

        assert_eq!(exit_code_opt, Some(11));
    }

    fn get_temp_file_path(filename: &str) -> String {
        let mut dir = std::env::temp_dir();
        dir.push(filename);
        dir.to_str().unwrap().to_owned()
    }

    fn link_object_file(
        object_file: &str,
        lib_path: Option<&str>,
        lib_soname: Option<&str>,
        output_file: &str,
    ) -> std::io::Result<ExitStatus> {
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

        let mut args = vec![];

        args.push("--dynamic-linker");
        args.push("/lib64/ld-linux-x86-64.so.2");
        args.push("-pie");
        args.push("-o");
        args.push(output_file);
        args.push("/usr/lib/Scrt1.o");
        args.push("/usr/lib/crti.o");
        args.push("-L/lib/");
        args.push("-L/usr/lib");

        if let Some(lib_path_str) = lib_path {
            args.push("-L");
            args.push(lib_path_str);
        }

        args.push(object_file);

        if let Some(lib_soname_str) = lib_soname {
            args.push("-l");
            args.push(lib_soname_str);
        }

        args.push("-lc");
        args.push("/usr/lib/crtn.o");

        Command::new("/usr/bin/ld").args(args).status()
    }

    fn delete_file(filepath: &str) {
        std::fs::remove_file(filepath).unwrap();
    }

    fn get_userlib_path() -> String {
        let mut pwd = std::env::current_dir().unwrap();

        if !pwd.ends_with("assembler") {
            // in the VSCode editor `Debug` environment, the `current_dir()` returns
            // the project's root folder.
            // while in both `$ cargo test` and VSCode editor `Run Test` environment,
            // the `current_dir()` returns the current crate path.
            // here canonicalize the test resources path.
            pwd.push("crates");
            pwd.push("assembler");
        }

        pwd.push("tests");
        pwd.push("lib");
        pwd.to_str().unwrap().to_string()
    }

    fn run_executable_binary_and_get_exit_code(module_binary: &[u8], name: &str) -> Option<i32> {
        // write object file
        let object_file_path = get_temp_file_path(&format!("{}.o", name));
        let mut file = File::create(&object_file_path).unwrap();
        file.write_all(&module_binary).unwrap();

        // link file
        let exec_file_path = get_temp_file_path(&format!("{}.elf", name));
        link_object_file(&object_file_path, None, None, &exec_file_path).unwrap();

        // Run the executable file and get the exit code
        // `$ ./anna.elf`
        // `$ echo $?`

        // run executable file and get exit code
        let exit_code_opt = Command::new(&exec_file_path).status().unwrap().code();

        // clean up
        delete_file(&object_file_path);
        delete_file(&exec_file_path);

        exit_code_opt
    }

    #[test]
    fn test_codegen_function_call() {
        let mut generator = CodeGenerator::new_object_file("main");

        let mut sig_swap = generator.module.make_signature();
        sig_swap.params.push(AbiParam::new(types::I32));
        sig_swap.params.push(AbiParam::new(types::I32));
        sig_swap.returns.push(AbiParam::new(types::I32));
        sig_swap.returns.push(AbiParam::new(types::I32));

        let func_swap_id = generator
            .module
            .declare_function("swap", Linkage::Local, &sig_swap)
            .unwrap();

        {
            let mut func = Function::with_name_signature(
                UserFuncName::user(0, func_swap_id.as_u32()),
                sig_swap,
            );

            let mut func_builder: FunctionBuilder = FunctionBuilder::new(
                // &mut generator.context.func,
                &mut func,
                &mut generator.function_builder_context,
            );
            let block = func_builder.create_block();
            func_builder.append_block_params_for_function_params(block);
            func_builder.switch_to_block(block);

            let value_a = func_builder.block_params(block)[0];
            let value_b = func_builder.block_params(block)[1];

            // return (b, a)
            func_builder.ins().return_(&[value_b, value_a]);

            func_builder.seal_all_blocks();
            func_builder.finalize();

            // generate the function code

            generator.context.func = func;

            generator
                .module
                .define_function(func_swap_id, &mut generator.context)
                .unwrap();

            generator.module.clear_context(&mut generator.context);
        }

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

            let func_ref0 = generator
                .module
                .declare_func_in_func(func_swap_id, &mut func);

            let mut func_builder: FunctionBuilder = FunctionBuilder::new(
                // &mut generator.context.func,
                &mut func,
                &mut generator.function_builder_context,
            );

            // ()                                 (i32)
            // start ---> check0 ---> check1 ---> exit
            //                    |           ^
            //                    \-----------/

            let block_start = func_builder.create_block();
            func_builder.append_block_params_for_function_params(block_start);

            let block_check0 = func_builder.create_block();
            let block_check1 = func_builder.create_block();

            let block_exit = func_builder.create_block();
            func_builder.append_block_params_for_function_returns(block_exit);

            // build block_start
            func_builder.switch_to_block(block_start);

            // call swap(11, 13) -> (13, 11)
            let value_0 = func_builder.ins().iconst(types::I32, 11);
            let value_1 = func_builder.ins().iconst(types::I32, 13);

            let call0 = func_builder.ins().call(func_ref0, &[value_0, value_1]);
            let call0_results = func_builder.inst_results(call0).to_vec();
            func_builder.ins().jump(block_check0, &[]);

            // build block_check0
            func_builder.switch_to_block(block_check0);

            // check results 1/2
            let check_result_0 = func_builder
                .ins()
                .icmp_imm(IntCC::Equal, call0_results[0], 13);
            let exit_code_imm_1 = func_builder.ins().iconst(types::I32, 1);

            func_builder.ins().brif(
                check_result_0,
                block_check1,
                &[],
                block_exit,
                &[exit_code_imm_1],
            );

            // build block_check1
            func_builder.switch_to_block(block_check1);

            // check results 2/2
            let check_result_1 = func_builder
                .ins()
                .icmp_imm(IntCC::Equal, call0_results[1], 11);
            let exit_code_imm_2 = func_builder.ins().iconst(types::I32, 2);
            let exit_code_imm_0 = func_builder.ins().iconst(types::I32, 0);

            func_builder.ins().brif(
                check_result_1,
                block_exit,
                &[exit_code_imm_0],
                block_exit,
                &[exit_code_imm_2],
            );

            // build block_exit
            func_builder.switch_to_block(block_exit);

            let exit_code_value = func_builder.block_params(block_exit)[0];
            func_builder.ins().return_(&[exit_code_value]);

            // all blocks are finish
            func_builder.seal_all_blocks();
            func_builder.finalize();

            // println!("{}", func.display());

            // generate the function code

            generator.context.func = func;

            generator
                .module
                .define_function(func_main_id, &mut generator.context)
                .unwrap();

            generator.module.clear_context(&mut generator.context);
        }

        // finish the module
        let object_procduct = generator.module.finish();
        let module_binary = object_procduct.emit().unwrap();
        let exit_code_opt = run_executable_binary_and_get_exit_code(
            &module_binary,
            "anna_unit_test_codegen_function_call",
        );

        assert_eq!(exit_code_opt, Some(0));
    }

    #[test]
    fn test_codegen_data() {
        let mut generator = CodeGenerator::new_object_file("main");

        let addr_t: Type = generator.module.isa().pointer_type();

        // define read-only data
        let data_ro_content = 11u32.to_le_bytes().to_vec();
        let data_ro_id = generator
            .define_inited_data("number0", data_ro_content, 4, Linkage::Local, false, false)
            .unwrap();

        // define read-write data
        let data_rw_content = 13u32.to_le_bytes().to_vec();
        let data_rw_id = generator
            .define_inited_data("number1", data_rw_content, 4, Linkage::Local, true, false)
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

            let gv_data_ro = generator.module.declare_data_in_func(data_ro_id, &mut func);
            let gv_data_rw = generator.module.declare_data_in_func(data_rw_id, &mut func);

            let mut func_builder: FunctionBuilder = FunctionBuilder::new(
                // &mut generator.context.func,
                &mut func,
                &mut generator.function_builder_context,
            );

            //            check ro    check rw    update and check rw
            // start ---> check0 ---> check1 ---> check2  ---> exit
            //                    |           |            ^
            //                    |           \------------|
            //                    \------------------------/

            let block_start = func_builder.create_block();
            func_builder.append_block_params_for_function_params(block_start);

            let block_check0 = func_builder.create_block();
            let block_check1 = func_builder.create_block();
            let block_check2 = func_builder.create_block();

            let block_exit = func_builder.create_block();
            func_builder.append_block_params_for_function_returns(block_exit);

            // build block_start
            func_builder.switch_to_block(block_start);
            func_builder.ins().jump(block_check0, &[]);

            // build block_check0
            func_builder.switch_to_block(block_check0);
            let data_ro_addr = func_builder.ins().symbol_value(addr_t, gv_data_ro);
            let value_ro_0 = func_builder
                .ins()
                .load(types::I32, MemFlags::new(), data_ro_addr, 0);

            let check_result_0 = func_builder.ins().icmp_imm(IntCC::Equal, value_ro_0, 11);
            let exit_code_imm_1 = func_builder.ins().iconst(types::I32, 1);

            func_builder.ins().brif(
                check_result_0,
                block_check1,
                &[],
                block_exit,
                &[exit_code_imm_1],
            );

            // build block_check1
            func_builder.switch_to_block(block_check1);
            let data_rw_addr = func_builder.ins().symbol_value(addr_t, gv_data_rw);
            let value_rw_0 = func_builder
                .ins()
                .load(types::I32, MemFlags::new(), data_rw_addr, 0);

            let check_result_1 = func_builder.ins().icmp_imm(IntCC::Equal, value_rw_0, 13);
            let exit_code_imm_2 = func_builder.ins().iconst(types::I32, 2);

            func_builder.ins().brif(
                check_result_1,
                block_check2,
                &[],
                block_exit,
                &[exit_code_imm_2],
            );

            // build block_check2
            func_builder.switch_to_block(block_check2);
            let value_imm_17 = func_builder.ins().iconst(types::I32, 17);
            func_builder
                .ins()
                .store(MemFlags::new(), value_imm_17, data_rw_addr, 0);

            let value_rw_1 = func_builder
                .ins()
                .load(types::I32, MemFlags::new(), data_rw_addr, 0);

            let check_result_2 = func_builder.ins().icmp_imm(IntCC::Equal, value_rw_1, 17);
            let exit_code_imm_0 = func_builder.ins().iconst(types::I32, 0);
            let exit_code_imm_3 = func_builder.ins().iconst(types::I32, 3);

            func_builder.ins().brif(
                check_result_2,
                block_exit,
                &[exit_code_imm_0],
                block_exit,
                &[exit_code_imm_3],
            );

            // build block_exit
            func_builder.switch_to_block(block_exit);

            let exit_code_value = func_builder.block_params(block_exit)[0];
            func_builder.ins().return_(&[exit_code_value]);

            // all blocks are finish
            func_builder.seal_all_blocks();
            func_builder.finalize();

            println!("{}", func.display());

            generator.context.func = func;

            generator
                .module
                .define_function(func_main_id, &mut generator.context)
                .unwrap();

            generator.module.clear_context(&mut generator.context);
        }

        // note:
        // the flow for JIT module:
        //
        // 1.linking
        // `generator.module.finalize_definitions().unwrap();`
        //
        // 2. get function pointers
        // `let func_main_ptr = generator.module.get_finalized_function(func_main_id);`
        //
        // 3. get data pointer
        //
        // ```rust
        // let (buf_ptr, buf_size) = generator.module.get_finalized_data(data_id);
        // let buf = unsafe { std::slice::from_raw_parts(buf_ptr, buf_size) };
        // ```
        //
        // note that the pointers of functions and data only available after 'module.finalize_definitions()'
        //
        // 4. cast ptr to Rust function
        // `let func_main: extern "C" fn() -> i32 = unsafe { std::mem::transmute(func_main_ptr) };`
        //
        // 5. execute the function:
        // `assert_eq!(func_main(), 13);`

        // finish the module
        let object_procduct = generator.module.finish();
        let module_binary = object_procduct.emit().unwrap();
        let exit_code_opt =
            run_executable_binary_and_get_exit_code(&module_binary, "anna_unit_test_codegen_data");

        assert_eq!(exit_code_opt, Some(0));
    }
}
