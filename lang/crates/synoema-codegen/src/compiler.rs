// SPDX-License-Identifier: BUSL-1.1
// Copyright (c) 2025-present Andrey Bubnov

//! Synoema → Cranelift native code compiler
//!
//! Compiles Core IR to native machine code using Cranelift JIT.

use std::collections::HashMap;

use cranelift_codegen::ir::{types, AbiParam, InstBuilder, MemFlags, condcodes::IntCC};
use cranelift_codegen::settings::{self, Configurable};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};

use synoema_core::*;
use synoema_parser::Lit;
use crate::runtime;

#[derive(Debug)]
pub struct CompileError(pub String);

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Compile error: {}", self.0)
    }
}

type CResult<T> = Result<T, CompileError>;
fn cerr(msg: impl Into<String>) -> CompileError { CompileError(msg.into()) }

/// Context for tail-call optimization: tracks the current function's
/// name, loop header block, and parameter Variables so that a self-recursive
/// tail call can be compiled as a jump instead of a call.
struct TcoContext {
    self_name: String,
    loop_block: cranelift_codegen::ir::Block,
    params: Vec<Variable>,
    emit_regions: bool,
}

/// Static analysis: does a Core IR expression contain any heap-allocating nodes?
/// Used to skip region_enter/region_exit for leaf (non-allocating) functions.
fn needs_heap(expr: &CoreExpr) -> bool {
    match expr {
        CoreExpr::Var(_) | CoreExpr::PrimOp(_) | CoreExpr::RuntimeError(_) => false,
        CoreExpr::Lit(lit) => matches!(lit, Lit::Str(_)),
        CoreExpr::App(f, a) => needs_heap(f) || needs_heap(a),
        CoreExpr::Lam(_, body) => needs_heap(body),
        CoreExpr::Let(_, val, body) | CoreExpr::LetRec(_, val, body) => {
            needs_heap(val) || needs_heap(body)
        }
        CoreExpr::Case(scrut, alts) => {
            needs_heap(scrut) || alts.iter().any(|alt| needs_heap(&alt.body))
        }
        CoreExpr::FieldAccess(base, _) => needs_heap(base),
        CoreExpr::Region(body) => needs_heap(body),
        // Heap-allocating nodes:
        CoreExpr::MkList(_) | CoreExpr::MkClosure { .. } | CoreExpr::Record(_)
        | CoreExpr::RecordUpdate { .. } | CoreExpr::Con(_)
        | CoreExpr::Scope(_) | CoreExpr::Spawn(_) => true,
    }
}

/// The Synoema JIT compiler
pub struct Compiler {
    module: JITModule,
    ctx: cranelift_codegen::Context,
    functions: HashMap<String, cranelift_module::FuncId>,
    ctor_tags: HashMap<String, (i64, usize)>,
}

impl Compiler {
    pub fn new() -> CResult<Self> {
        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false").unwrap();
        flag_builder.set("is_pic", "false").unwrap();

        let isa_builder = cranelift_native::builder()
            .map_err(|e| cerr(format!("ISA: {}", e)))?;
        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .map_err(|e| cerr(format!("ISA finish: {}", e)))?;

        let mut builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());

        // Register Synoema runtime functions
        use crate::runtime;
        builder.symbol("synoema_make_closure", runtime::synoema_make_closure as *const u8);
        builder.symbol("synoema_env_alloc", runtime::synoema_env_alloc as *const u8);
        builder.symbol("synoema_print_int", runtime::synoema_print_int as *const u8);
        builder.symbol("synoema_nil", runtime::synoema_nil as *const u8);
        builder.symbol("synoema_cons", runtime::synoema_cons as *const u8);
        builder.symbol("synoema_head", runtime::synoema_head as *const u8);
        builder.symbol("synoema_tail", runtime::synoema_tail as *const u8);
        builder.symbol("synoema_is_nil", runtime::synoema_is_nil as *const u8);
        builder.symbol("synoema_concat", runtime::synoema_concat as *const u8);
        builder.symbol("synoema_length", runtime::synoema_length as *const u8);
        builder.symbol("synoema_print_list", runtime::synoema_print_list as *const u8);
        builder.symbol("synoema_println_val", runtime::synoema_println_val as *const u8);
        builder.symbol("synoema_sum", runtime::synoema_sum as *const u8);
        builder.symbol("synoema_str_new", runtime::synoema_str_new as *const u8);
        builder.symbol("synoema_show_int", runtime::synoema_show_int as *const u8);
        builder.symbol("synoema_str_concat", runtime::synoema_str_concat as *const u8);
        builder.symbol("synoema_str_length", runtime::synoema_str_length as *const u8);
        builder.symbol("synoema_str_eq", runtime::synoema_str_eq as *const u8);
        // String stdlib
        builder.symbol("synoema_str_slice", runtime::synoema_str_slice as *const u8);
        builder.symbol("synoema_str_find", runtime::synoema_str_find as *const u8);
        builder.symbol("synoema_str_starts_with", runtime::synoema_str_starts_with as *const u8);
        builder.symbol("synoema_str_trim", runtime::synoema_str_trim as *const u8);
        builder.symbol("synoema_str_len", runtime::synoema_str_len as *const u8);
        builder.symbol("synoema_str_join", runtime::synoema_str_join as *const u8);
        builder.symbol("synoema_json_escape", runtime::synoema_json_escape as *const u8);
        builder.symbol("synoema_json_parse", runtime::synoema_json_parse as *const u8);
        builder.symbol("synoema_error", runtime::synoema_error as *const u8);
        builder.symbol("synoema_concatmap", runtime::synoema_concatmap as *const u8);
        builder.symbol("synoema_record_new", runtime::synoema_record_new as *const u8);
        builder.symbol("synoema_record_set", runtime::synoema_record_set as *const u8);
        builder.symbol("synoema_record_get", runtime::synoema_record_get as *const u8);
        builder.symbol("synoema_record_clone", runtime::synoema_record_clone as *const u8);
        builder.symbol("synoema_record_set_field", runtime::synoema_record_set_field as *const u8);
        builder.symbol("synoema_val_eq", runtime::synoema_val_eq as *const u8);
        builder.symbol("synoema_make_con", runtime::synoema_make_con as *const u8);
        builder.symbol("synoema_con_set", runtime::synoema_con_set as *const u8);
        builder.symbol("synoema_con_get_tag", runtime::synoema_con_get_tag as *const u8);
        builder.symbol("synoema_con_get_field", runtime::synoema_con_get_field as *const u8);
        // Float runtime functions
        builder.symbol("synoema_float_new", runtime::synoema_float_new as *const u8);
        builder.symbol("synoema_float_add", runtime::synoema_float_add as *const u8);
        builder.symbol("synoema_float_sub", runtime::synoema_float_sub as *const u8);
        builder.symbol("synoema_float_mul", runtime::synoema_float_mul as *const u8);
        builder.symbol("synoema_float_div", runtime::synoema_float_div as *const u8);
        builder.symbol("synoema_float_lt",  runtime::synoema_float_lt  as *const u8);
        builder.symbol("synoema_float_gt",  runtime::synoema_float_gt  as *const u8);
        builder.symbol("synoema_float_lte", runtime::synoema_float_lte as *const u8);
        builder.symbol("synoema_float_gte", runtime::synoema_float_gte as *const u8);
        builder.symbol("synoema_float_eq",  runtime::synoema_float_eq  as *const u8);
        builder.symbol("synoema_float_pow", runtime::synoema_float_pow as *const u8);
        builder.symbol("synoema_float_sqrt", runtime::synoema_float_sqrt as *const u8);
        builder.symbol("synoema_float_abs", runtime::synoema_float_abs as *const u8);
        builder.symbol("synoema_float_floor", runtime::synoema_float_floor as *const u8);
        builder.symbol("synoema_float_ceil", runtime::synoema_float_ceil as *const u8);
        builder.symbol("synoema_float_round", runtime::synoema_float_round as *const u8);
        builder.symbol("synoema_int_pow", runtime::synoema_int_pow as *const u8);
        builder.symbol("synoema_abs_int", runtime::synoema_abs_int as *const u8);
        // IO functions
        builder.symbol("synoema_print_val", runtime::synoema_print_val as *const u8);
        builder.symbol("synoema_readline", runtime::synoema_readline as *const u8);
        builder.symbol("synoema_show_any", runtime::synoema_show_any as *const u8);
        builder.symbol("synoema_show_bool", runtime::synoema_show_bool as *const u8);
        builder.symbol("synoema_show_list", runtime::synoema_show_list as *const u8);
        builder.symbol("synoema_show_con", runtime::synoema_show_con as *const u8);
        builder.symbol("synoema_list_eq", runtime::synoema_list_eq as *const u8);
        builder.symbol("synoema_range", runtime::synoema_range as *const u8);
        builder.symbol("synoema_map", runtime::synoema_map as *const u8);
        builder.symbol("synoema_filter", runtime::synoema_filter as *const u8);
        builder.symbol("synoema_foldl", runtime::synoema_foldl as *const u8);
        builder.symbol("synoema_zip", runtime::synoema_zip as *const u8);
        builder.symbol("synoema_index", runtime::synoema_index as *const u8);
        builder.symbol("synoema_take", runtime::synoema_take as *const u8);
        builder.symbol("synoema_drop", runtime::synoema_drop as *const u8);
        builder.symbol("synoema_reverse", runtime::synoema_reverse as *const u8);
        // Concurrency channel functions
        builder.symbol("synoema_chan_new",  runtime::synoema_chan_new  as *const u8);
        builder.symbol("synoema_chan_send", runtime::synoema_chan_send as *const u8);
        builder.symbol("synoema_chan_recv", runtime::synoema_chan_recv as *const u8);
        builder.symbol("synoema_match_error", runtime::synoema_match_error as *const u8);
        // Region inference FFI
        builder.symbol("synoema_region_enter", runtime::synoema_region_enter as *const u8);
        builder.symbol("synoema_region_exit", runtime::synoema_region_exit as *const u8);

        let module = JITModule::new(builder);

        Ok(Compiler {
            ctx: module.make_context(),
            module,
            functions: HashMap::new(),
            ctor_tags: HashMap::new(),
        })
    }

    /// Declare all runtime FFI functions in the module
    fn declare_runtime_functions(&mut self) -> CResult<()> {
        // Helper: declare fn(i64) -> i64
        let sig1 = {
            let mut s = self.module.make_signature();
            s.params.push(AbiParam::new(types::I64));
            s.returns.push(AbiParam::new(types::I64));
            s
        };
        // Helper: declare fn() -> i64
        let sig0 = {
            let mut s = self.module.make_signature();
            s.returns.push(AbiParam::new(types::I64));
            s
        };
        // Helper: declare fn(i64, i64) -> i64
        let sig2 = {
            let mut s = self.module.make_signature();
            s.params.push(AbiParam::new(types::I64));
            s.params.push(AbiParam::new(types::I64));
            s.returns.push(AbiParam::new(types::I64));
            s
        };
        // Helper: declare fn(i64, i64, i64, i64) -> void (record_set)
        let sig4 = {
            let mut s = self.module.make_signature();
            s.params.push(AbiParam::new(types::I64));
            s.params.push(AbiParam::new(types::I64));
            s.params.push(AbiParam::new(types::I64));
            s.params.push(AbiParam::new(types::I64));
            // no return value — declared without a return
            s
        };
        // Helper: declare fn(i64, i64, i64, i64) -> i64 (make_con: 4 params, 1 return)
        let sig4_ret = {
            let mut s = self.module.make_signature();
            s.params.push(AbiParam::new(types::I64));
            s.params.push(AbiParam::new(types::I64));
            s.params.push(AbiParam::new(types::I64));
            s.params.push(AbiParam::new(types::I64));
            s.returns.push(AbiParam::new(types::I64));
            s
        };
        // Helper: declare fn(i64, i64, i64) -> i64 (foldl: 3 params, 1 return)
        let sig3_ret = {
            let mut s = self.module.make_signature();
            s.params.push(AbiParam::new(types::I64));
            s.params.push(AbiParam::new(types::I64));
            s.params.push(AbiParam::new(types::I64));
            s.returns.push(AbiParam::new(types::I64));
            s
        };
        // Helper: declare fn(i64, i64, i64) -> void (con_set)
        let sig3_void = {
            let mut s = self.module.make_signature();
            s.params.push(AbiParam::new(types::I64));
            s.params.push(AbiParam::new(types::I64));
            s.params.push(AbiParam::new(types::I64));
            s // no return
        };

        let decl = |this: &mut Self, name: &str, alias: &str, sig: &cranelift_codegen::ir::Signature| -> CResult<()> {
            let id = this.module.declare_function(name, Linkage::Import, sig)
                .map_err(|e| cerr(format!("Declare '{}': {}", name, e)))?;
            this.functions.insert(alias.to_string(), id);
            Ok(())
        };

        // fn() -> i64
        decl(self, "synoema_nil", "synoema_nil", &sig0)?;
        decl(self, "synoema_readline", "readline", &sig0)?;
        // fn(i64) -> i64
        decl(self, "synoema_show_any", "show", &sig1)?;   // show any → tagged string ptr
        decl(self, "synoema_show_any", "synoema_show_any", &sig1)?;
        decl(self, "synoema_show_bool", "synoema_show_bool", &sig1)?;
        decl(self, "synoema_show_bool", "show_bool", &sig1)?;
        decl(self, "synoema_show_list", "synoema_show_list", &sig1)?;
        decl(self, "synoema_print_val", "print", &sig1)?;  // print any value, returns 0 (unit)
        decl(self, "synoema_print_val", "synoema_print_val", &sig1)?;
        decl(self, "synoema_head", "synoema_head", &sig1)?;
        decl(self, "synoema_tail", "synoema_tail", &sig1)?;
        decl(self, "synoema_is_nil", "synoema_is_nil", &sig1)?;
        decl(self, "synoema_length", "synoema_length", &sig1)?;
        decl(self, "synoema_print_list", "synoema_print_list", &sig1)?;
        decl(self, "synoema_println_val", "synoema_println_val", &sig1)?;
        decl(self, "synoema_sum", "synoema_sum", &sig1)?;
        decl(self, "synoema_length", "length", &sig1)?;
        decl(self, "synoema_sum", "sum", &sig1)?;
        decl(self, "synoema_head", "head", &sig1)?;
        decl(self, "synoema_tail", "tail", &sig1)?;
        decl(self, "synoema_str_length", "synoema_str_length", &sig1)?;
        decl(self, "synoema_show_int", "synoema_show_int", &sig1)?;
        // String stdlib: fn(i64) -> i64
        decl(self, "synoema_str_trim", "str_trim", &sig1)?;
        decl(self, "synoema_str_len", "str_len", &sig1)?;
        decl(self, "synoema_str_join", "str_join", &sig2)?;
        decl(self, "synoema_json_escape", "json_escape", &sig1)?;
        decl(self, "synoema_json_parse", "json_parse", &sig1)?;
        decl(self, "synoema_error", "error", &sig1)?;
        // fn(i64, i64) -> i64
        decl(self, "synoema_cons", "synoema_cons", &sig2)?;
        decl(self, "synoema_concat", "synoema_concat", &sig2)?;
        decl(self, "synoema_make_closure", "synoema_make_closure", &sig2)?;
        decl(self, "synoema_str_new", "synoema_str_new", &sig2)?;
        decl(self, "synoema_str_concat", "synoema_str_concat", &sig2)?;
        decl(self, "synoema_str_eq", "synoema_str_eq", &sig2)?;
        // String stdlib: fn(i64, i64) -> i64
        decl(self, "synoema_str_starts_with", "str_starts_with", &sig2)?;
        // synoema_concatmap: fn(i64, i64) -> i64  (closure_ptr, list -> list)
        decl(self, "synoema_concatmap", "concatMap", &sig2)?;
        // fn(i64) -> i64
        decl(self, "synoema_env_alloc", "synoema_env_alloc", &sig1)?;
        // Records: fn(i64) -> i64
        decl(self, "synoema_record_new", "synoema_record_new", &sig1)?;
        decl(self, "synoema_record_clone", "synoema_record_clone", &sig1)?;
        // Records: fn(i64, i64) -> i64
        decl(self, "synoema_record_get", "synoema_record_get", &sig2)?;
        // Records: fn(i64, i64, i64) -> void
        decl(self, "synoema_record_set_field", "synoema_record_set_field", &sig3_void)?;
        // Records: fn(i64, i64, i64, i64) -> void (record_set)
        decl(self, "synoema_record_set", "synoema_record_set", &sig4)?;
        // Universal equality + list equality
        decl(self, "synoema_val_eq", "synoema_val_eq", &sig2)?;
        decl(self, "synoema_list_eq", "synoema_list_eq", &sig2)?;
        // Range: fn(i64, i64) -> i64  ([from..to] → list)
        decl(self, "synoema_range", "synoema_range", &sig2)?;
        // Higher-order stdlib: map/filter fn(i64,i64)->i64, foldl fn(i64,i64,i64)->i64
        decl(self, "synoema_map",    "map",    &sig2)?;
        decl(self, "synoema_filter", "filter", &sig2)?;
        decl(self, "synoema_foldl",  "foldl",  &sig3_ret)?;
        // List builtins
        decl(self, "synoema_zip",     "zip",     &sig2)?;
        decl(self, "synoema_index",   "index",   &sig2)?;
        decl(self, "synoema_take",    "take",    &sig2)?;
        decl(self, "synoema_drop",    "drop",    &sig2)?;
        decl(self, "synoema_reverse", "reverse", &sig1)?;
        // String stdlib: fn(i64, i64, i64) -> i64
        decl(self, "synoema_str_slice", "str_slice", &sig3_ret)?;
        decl(self, "synoema_str_find", "str_find", &sig3_ret)?;
        // ADT ConNode functions
        decl(self, "synoema_make_con", "synoema_make_con", &sig4_ret)?; // fn(i64,i64,i64,i64)->i64
        decl(self, "synoema_show_con", "synoema_show_con", &sig1)?;
        decl(self, "synoema_con_get_tag", "synoema_con_get_tag", &sig1)?;  // fn(i64)->i64
        decl(self, "synoema_con_get_field", "synoema_con_get_field", &sig2)?; // fn(i64,i64)->i64
        decl(self, "synoema_con_set", "synoema_con_set", &sig3_void)?; // fn(i64,i64,i64)->void
        // Float functions: fn(i64) -> i64
        decl(self, "synoema_float_new", "synoema_float_new", &sig1)?;
        // Float arithmetic: fn(i64, i64) -> i64
        decl(self, "synoema_float_add", "synoema_float_add", &sig2)?;
        decl(self, "synoema_float_sub", "synoema_float_sub", &sig2)?;
        decl(self, "synoema_float_mul", "synoema_float_mul", &sig2)?;
        decl(self, "synoema_float_div", "synoema_float_div", &sig2)?;
        decl(self, "synoema_float_lt",  "synoema_float_lt",  &sig2)?;
        decl(self, "synoema_float_gt",  "synoema_float_gt",  &sig2)?;
        decl(self, "synoema_float_lte", "synoema_float_lte", &sig2)?;
        decl(self, "synoema_float_gte", "synoema_float_gte", &sig2)?;
        decl(self, "synoema_float_eq",  "synoema_float_eq",  &sig2)?;
        // Float power: fn(i64, i64) -> i64
        decl(self, "synoema_float_pow", "synoema_float_pow", &sig2)?;
        // Float unary math: fn(i64) -> i64
        decl(self, "synoema_float_sqrt",  "synoema_float_sqrt",  &sig1)?;
        decl(self, "synoema_float_abs",   "synoema_float_abs",   &sig1)?;
        decl(self, "synoema_float_floor", "synoema_float_floor", &sig1)?;
        decl(self, "synoema_float_ceil",  "synoema_float_ceil",  &sig1)?;
        decl(self, "synoema_float_round", "synoema_float_round", &sig1)?;
        // Float math builtins exposed as named functions
        decl(self, "synoema_float_sqrt",  "sqrt",  &sig1)?;
        decl(self, "synoema_float_floor", "floor", &sig1)?;
        decl(self, "synoema_float_ceil",  "ceil",  &sig1)?;
        decl(self, "synoema_float_round", "round", &sig1)?;
        decl(self, "synoema_float_abs",   "fabs",  &sig1)?;
        // Integer power: fn(i64, i64) -> i64
        decl(self, "synoema_int_pow", "synoema_int_pow", &sig2)?;
        // Integer abs: fn(i64) -> i64
        decl(self, "synoema_abs_int", "synoema_abs_int", &sig1)?;
        decl(self, "synoema_abs_int", "abs", &sig1)?;

        // Concurrency (Phase C): channel primitives
        decl(self, "synoema_chan_new",  "chan",  &sig0)?;   // fn() -> i64
        decl(self, "synoema_chan_send", "send",  &sig2)?;   // fn(i64, i64) -> i64
        decl(self, "synoema_chan_recv", "recv",  &sig1)?;   // fn(i64) -> i64

        // Runtime error: fn(i64, i64) -> i64 (ptr, len → never returns)
        decl(self, "synoema_match_error", "synoema_match_error", &sig2)?;

        // Region inference: fn() -> i64
        decl(self, "synoema_region_enter", "synoema_region_enter", &sig0)?;
        decl(self, "synoema_region_exit", "synoema_region_exit", &sig0)?;

        Ok(())
    }

    /// Compile a program and return the result of main()
    pub fn compile_and_run(&mut self, program: &CoreProgram) -> CResult<i64> {
        // Declare runtime functions (FFI)
        self.declare_runtime_functions()?;

        // Load constructor tags from the program
        self.ctor_tags = program.ctor_tags.clone();

        // Lambda lifting: extract all Lam expressions to top-level closure functions.
        // Only inner lambdas (inside function bodies) are lifted; outer parameter
        // lambdas are left in place and handled by compile_function via peel_lambdas.
        let globals: std::collections::HashSet<String> =
            program.defs.iter().map(|d| d.name.clone()).collect();
        let mut lifted: HashMap<String, (Vec<String>, String, CoreExpr)> = HashMap::new();
        let mut lift_counter: u32 = 0;
        let mut lifted_defs: Vec<CoreDef> = Vec::new();

        for def in &program.defs {
            let (params, inner) = peel_lambdas(&def.body);
            let param_set: std::collections::HashSet<String> = params.iter().cloned().collect();
            let new_inner = lift_expr(inner, &param_set, &globals, &mut lifted, &mut lift_counter);
            let mut new_body = new_inner;
            for p in params.iter().rev() {
                new_body = CoreExpr::Lam(p.clone(), Box::new(new_body));
            }
            lifted_defs.push(CoreDef { name: def.name.clone(), body: new_body });
        }

        // Pass 1: declare all user functions
        for def in &lifted_defs {
            let arity = count_lambdas(&def.body);
            let mut sig = self.module.make_signature();
            for _ in 0..arity {
                sig.params.push(AbiParam::new(types::I64));
            }
            sig.returns.push(AbiParam::new(types::I64));

            let id = self.module
                .declare_function(&def.name, Linkage::Local, &sig)
                .map_err(|e| cerr(format!("Declare '{}': {}", def.name, e)))?;
            self.functions.insert(def.name.clone(), id);
        }

        // Declare lifted closure functions (signature: fn(env_ptr: i64, arg: i64) -> i64)
        for func_name in lifted.keys() {
            let mut sig = self.module.make_signature();
            sig.params.push(AbiParam::new(types::I64)); // env_ptr
            sig.params.push(AbiParam::new(types::I64)); // arg
            sig.returns.push(AbiParam::new(types::I64));
            let id = self.module
                .declare_function(func_name, Linkage::Local, &sig)
                .map_err(|e| cerr(format!("Declare closure '{}': {}", func_name, e)))?;
            self.functions.insert(func_name.clone(), id);
        }

        // Pass 2: define all user functions
        for def in &lifted_defs {
            self.compile_function(&def.name, &def.body)?;
        }

        // Compile lifted closure functions
        let lifted_entries: Vec<(String, Vec<String>, String, CoreExpr)> = lifted
            .into_iter()
            .map(|(n, (fv, p, b))| (n, fv, p, b))
            .collect();
        for (func_name, free_vars, param, body) in &lifted_entries {
            self.compile_closure_function(func_name, free_vars, param, body)?;
        }

        // Finalize
        self.module.finalize_definitions()
            .map_err(|e| cerr(format!("Finalize: {}", e)))?;

        // Get main
        let main_id = self.functions.get("main")
            .ok_or_else(|| cerr("No 'main' function"))?;
        let ptr = self.module.get_finalized_function(*main_id);
        let main_fn: fn() -> i64 = unsafe { std::mem::transmute(ptr) };
        Ok(main_fn())
    }

    /// Compile a lifted closure function.
    /// Signature: fn(env_ptr: i64, param: i64) -> i64
    /// The env_ptr points to an array of i64 values (captured free variables).
    fn compile_closure_function(
        &mut self,
        name: &str,
        free_vars: &[String],
        param: &str,
        body: &CoreExpr,
    ) -> CResult<()> {
        let func_id = *self.functions.get(name)
            .ok_or_else(|| cerr(format!("Closure function not declared: {}", name)))?;

        self.ctx.func.signature.params.clear();
        self.ctx.func.signature.returns.clear();
        self.ctx.func.signature.params.push(AbiParam::new(types::I64)); // env_ptr
        self.ctx.func.signature.params.push(AbiParam::new(types::I64)); // param
        self.ctx.func.signature.returns.push(AbiParam::new(types::I64));

        let mut fb_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut fb_ctx);

        let entry = builder.create_block();
        builder.append_block_params_for_function_params(entry);
        builder.switch_to_block(entry);
        builder.seal_block(entry);

        let mut vars: HashMap<String, Variable> = HashMap::new();
        let mut vc: u32 = 0;

        // env_ptr = block_params[0] (used for loading free vars below)
        let env_ptr_val = builder.block_params(entry)[0];

        // param = block_params[1]
        let param_val = builder.block_params(entry)[1];
        let param_var = Variable::from_u32(vc); vc += 1;
        builder.declare_var(param_var, types::I64);
        builder.def_var(param_var, param_val);
        vars.insert(param.to_string(), param_var);

        // Load each captured free variable from env_ptr[i]
        for (i, fv) in free_vars.iter().enumerate() {
            let offset = (i * 8) as i32;
            let fv_val = builder.ins().load(types::I64, MemFlags::new(), env_ptr_val, offset);
            let fv_var = Variable::from_u32(vc); vc += 1;
            builder.declare_var(fv_var, types::I64);
            builder.def_var(fv_var, fv_val);
            vars.insert(fv.clone(), fv_var);
        }

        let ctor_tags = self.ctor_tags.clone();
        let result = compile_expr(
            &mut builder, &mut vars, &mut vc,
            &self.functions, &mut self.module, &ctor_tags, body, None,
        )?;

        builder.ins().return_(&[result]);
        builder.finalize();

        self.module.define_function(func_id, &mut self.ctx)
            .map_err(|e| cerr(format!("Define closure '{}': {}", name, e)))?;
        self.module.clear_context(&mut self.ctx);
        Ok(())
    }

    fn compile_function(&mut self, name: &str, body: &CoreExpr) -> CResult<()> {
        let func_id = *self.functions.get(name).unwrap();
        let (params, inner) = peel_lambdas(body);

        self.ctx.func.signature.params.clear();
        self.ctx.func.signature.returns.clear();
        for _ in 0..params.len() {
            self.ctx.func.signature.params.push(AbiParam::new(types::I64));
        }
        self.ctx.func.signature.returns.push(AbiParam::new(types::I64));

        let mut fb_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut fb_ctx);

        let entry = builder.create_block();
        builder.append_block_params_for_function_params(entry);
        builder.switch_to_block(entry);
        builder.seal_block(entry);

        let mut vars = HashMap::new();
        let mut vc: u32 = 0;

        let mut param_vars = Vec::new();
        for (i, pname) in params.iter().enumerate() {
            let var = Variable::from_u32(vc);
            vc += 1;
            builder.declare_var(var, types::I64);
            let pval = builder.block_params(entry)[i];
            builder.def_var(var, pval);
            vars.insert(pname.clone(), var);
            param_vars.push(var);
        }

        // TCO: create loop header block for self-recursive tail calls.
        let loop_header = builder.create_block();
        builder.ins().jump(loop_header, &[]);
        builder.switch_to_block(loop_header);
        // Don't seal loop_header yet — TCO jumps will add predecessors.

        // Skip region_enter/region_exit for functions that never allocate heap memory.
        let emit_regions = needs_heap(inner);

        if emit_regions {
            // TCO auto-region: enter a new region at each loop iteration.
            // region_exit is emitted before each back-edge (tail call → jump to loop_header).
            // Base-case returns do NOT exit the region — arena_reset handles final cleanup.
            let region_enter_id = *self.functions.get("synoema_region_enter")
                .ok_or_else(|| cerr("synoema_region_enter not declared"))?;
            let region_enter_ref = self.module.declare_func_in_func(region_enter_id, builder.func);
            builder.ins().call(region_enter_ref, &[]);
        }

        let tco_ctx = TcoContext {
            self_name: name.to_string(),
            loop_block: loop_header,
            params: param_vars,
            emit_regions,
        };

        let ctor_tags = self.ctor_tags.clone();
        let result = compile_expr(
            &mut builder, &mut vars, &mut vc,
            &self.functions, &mut self.module, &ctor_tags, inner,
            Some(&tco_ctx),
        )?;

        builder.ins().return_(&[result]);
        builder.seal_block(loop_header);
        builder.finalize();

        self.module.define_function(func_id, &mut self.ctx)
            .map_err(|e| cerr(format!("Define '{}': {}", name, e)))?;
        self.module.clear_context(&mut self.ctx);
        Ok(())
    }
}

fn compile_expr(
    builder: &mut FunctionBuilder,
    vars: &mut HashMap<String, Variable>,
    vc: &mut u32,
    funcs: &HashMap<String, cranelift_module::FuncId>,
    module: &mut JITModule,
    ctor_tags: &HashMap<String, (i64, usize)>,
    expr: &CoreExpr,
    tco_ctx: Option<&TcoContext>,
) -> CResult<cranelift_codegen::ir::Value> {
    match expr {
        CoreExpr::Lit(Lit::Int(n)) => Ok(builder.ins().iconst(types::I64, *n)),
        CoreExpr::Lit(Lit::Bool(b)) => Ok(builder.ins().iconst(types::I64, if *b {1} else {0})),
        CoreExpr::Lit(Lit::Unit) => Ok(builder.ins().iconst(types::I64, 0)), // unit = 0
        CoreExpr::Lit(Lit::Str(s)) => {
            // Use the AST string's pointer directly — the AST outlives JIT execution.
            // synoema_str_new copies bytes into the arena at runtime.
            let data_ptr = s.as_ptr() as i64;
            let len = s.len() as i64;
            let data_ptr_val = builder.ins().iconst(types::I64, data_ptr);
            let len_val = builder.ins().iconst(types::I64, len);
            let str_new_id = *funcs.get("synoema_str_new")
                .ok_or_else(|| cerr("synoema_str_new not declared"))?;
            let fref = module.declare_func_in_func(str_new_id, builder.func);
            let call = builder.ins().call(fref, &[data_ptr_val, len_val]);
            Ok(builder.inst_results(call)[0])
        }
        CoreExpr::Lit(Lit::Float(f)) => {
            // Allocate a FloatNode at JIT runtime via synoema_float_new(bits).
            // The bits are computed at compile time and embedded as an iconst.
            let bits = f.to_bits() as i64;
            let bits_val = builder.ins().iconst(types::I64, bits);
            let float_new_id = *funcs.get("synoema_float_new")
                .ok_or_else(|| cerr("synoema_float_new not declared"))?;
            let fref = module.declare_func_in_func(float_new_id, builder.func);
            let call = builder.ins().call(fref, &[bits_val]);
            Ok(builder.inst_results(call)[0])
        }
        CoreExpr::Lit(_) => Err(cerr("Only Int/Bool/Str/Float literals supported in codegen")),

        CoreExpr::Var(name) => {
            if let Some(&var) = vars.get(name) {
                Ok(builder.use_var(var))
            } else if let Some(&fid) = funcs.get(name.as_str()) {
                // Check if this is a zero-arity global (e.g. `p = {x=3, y=4}` referenced as `p`)
                let n_params = module.declarations().get_function_decl(fid).signature.params.len();
                if n_params == 0 {
                    let fref = module.declare_func_in_func(fid, builder.func);
                    let call = builder.ins().call(fref, &[]);
                    Ok(builder.inst_results(call)[0])
                } else {
                    Err(cerr(format!("Undefined: {}", name)))
                }
            } else {
                Err(cerr(format!("Undefined: {}", name)))
            }
        }

        CoreExpr::Let(name, val, body) | CoreExpr::LetRec(name, val, body) => {
            let v = compile_expr(builder, vars, vc, funcs, module, ctor_tags, val, None)?;
            let var = Variable::from_u32(*vc);
            *vc += 1;
            builder.declare_var(var, types::I64);
            builder.def_var(var, v);
            vars.insert(name.clone(), var);
            compile_expr(builder, vars, vc, funcs, module, ctor_tags, body, tco_ctx)
        }

        CoreExpr::App(func, arg) => {
            // Check for ADT constructor application: App(App(Con("Just"), ...), x)
            if let Some((ctor_name, args)) = flatten_con_app(func, arg, ctor_tags) {
                let &(tag, arity) = ctor_tags.get(&ctor_name).unwrap();
                // Embed constructor name as a static data section
                let name_bytes = ctor_name.as_bytes();
                let mut name_desc = cranelift_module::DataDescription::new();
                name_desc.define(name_bytes.to_vec().into_boxed_slice());
                let name_id = module.declare_anonymous_data(false, false)
                    .map_err(|e| cerr(format!("con name data: {}", e)))?;
                module.define_data(name_id, &name_desc)
                    .map_err(|e| cerr(format!("con name define: {}", e)))?;
                let name_ref = module.declare_data_in_func(name_id, builder.func);
                let name_ptr = builder.ins().global_value(types::I64, name_ref);
                let name_len = builder.ins().iconst(types::I64, name_bytes.len() as i64);
                // Allocate ConNode: synoema_make_con(tag, arity, name_ptr, name_len) -> tagged ptr
                let tag_val = builder.ins().iconst(types::I64, tag);
                let arity_val = builder.ins().iconst(types::I64, arity as i64);
                let make_con_id = *funcs.get("synoema_make_con")
                    .ok_or_else(|| cerr("synoema_make_con not declared"))?;
                let make_con_ref = module.declare_func_in_func(make_con_id, builder.func);
                let make_call = builder.ins().call(make_con_ref, &[tag_val, arity_val, name_ptr, name_len]);
                let con_ptr = builder.inst_results(make_call)[0];
                // Set each payload field
                let con_set_id = *funcs.get("synoema_con_set")
                    .ok_or_else(|| cerr("synoema_con_set not declared"))?;
                for (i, a) in args.iter().enumerate() {
                    let v = compile_expr(builder, vars, vc, funcs, module, ctor_tags, a, None)?;
                    let idx_val = builder.ins().iconst(types::I64, i as i64);
                    let con_set_ref = module.declare_func_in_func(con_set_id, builder.func);
                    builder.ins().call(con_set_ref, &[con_ptr, idx_val, v]);
                }
                return Ok(con_ptr);
            }
            // PrimOp binary: App(App(PrimOp, lhs), rhs)
            if let CoreExpr::App(inner, lhs) = func.as_ref() {
                if let CoreExpr::PrimOp(op) = inner.as_ref() {
                    // Cons and Concat need runtime calls
                    match op {
                        PrimOp::Cons => {
                            let l = compile_expr(builder, vars, vc, funcs, module, ctor_tags, lhs, None)?;
                            let r = compile_expr(builder, vars, vc, funcs, module, ctor_tags, arg, None)?;
                            let fid = *funcs.get("synoema_cons").ok_or_else(|| cerr("synoema_cons"))?;
                            let fref = module.declare_func_in_func(fid, builder.func);
                            let call = builder.ins().call(fref, &[l, r]);
                            return Ok(builder.inst_results(call)[0]);
                        }
                        PrimOp::Concat => {
                            let l = compile_expr(builder, vars, vc, funcs, module, ctor_tags, lhs, None)?;
                            let r = compile_expr(builder, vars, vc, funcs, module, ctor_tags, arg, None)?;
                            let fid = *funcs.get("synoema_concat").ok_or_else(|| cerr("synoema_concat"))?;
                            let fref = module.declare_func_in_func(fid, builder.func);
                            let call = builder.ins().call(fref, &[l, r]);
                            return Ok(builder.inst_results(call)[0]);
                        }
                        PrimOp::Eq => {
                            // Runtime dispatch: synoema_val_eq handles both int and string equality
                            let l = compile_expr(builder, vars, vc, funcs, module, ctor_tags, lhs, None)?;
                            let r = compile_expr(builder, vars, vc, funcs, module, ctor_tags, arg, None)?;
                            let fid = *funcs.get("synoema_val_eq").ok_or_else(|| cerr("synoema_val_eq"))?;
                            let fref = module.declare_func_in_func(fid, builder.func);
                            let call = builder.ins().call(fref, &[l, r]);
                            return Ok(builder.inst_results(call)[0]);
                        }
                        PrimOp::Neq => {
                            // Runtime dispatch: 1 - synoema_val_eq(l, r)
                            let l = compile_expr(builder, vars, vc, funcs, module, ctor_tags, lhs, None)?;
                            let r = compile_expr(builder, vars, vc, funcs, module, ctor_tags, arg, None)?;
                            let fid = *funcs.get("synoema_val_eq").ok_or_else(|| cerr("synoema_val_eq"))?;
                            let fref = module.declare_func_in_func(fid, builder.func);
                            let call = builder.ins().call(fref, &[l, r]);
                            let eq_val = builder.inst_results(call)[0];
                            let one = builder.ins().iconst(types::I64, 1);
                            return Ok(builder.ins().isub(one, eq_val));
                        }
                        // Float arithmetic and comparison: call float runtime functions
                        PrimOp::FAdd | PrimOp::FSub | PrimOp::FMul | PrimOp::FDiv | PrimOp::FPow
                        | PrimOp::FLt | PrimOp::FGt | PrimOp::FLte | PrimOp::FGte | PrimOp::FEq => {
                            let l = compile_expr(builder, vars, vc, funcs, module, ctor_tags, lhs, None)?;
                            let r = compile_expr(builder, vars, vc, funcs, module, ctor_tags, arg, None)?;
                            let fn_name = match op {
                                PrimOp::FAdd  => "synoema_float_add",
                                PrimOp::FSub  => "synoema_float_sub",
                                PrimOp::FMul  => "synoema_float_mul",
                                PrimOp::FDiv  => "synoema_float_div",
                                PrimOp::FPow  => "synoema_float_pow",
                                PrimOp::FLt   => "synoema_float_lt",
                                PrimOp::FGt   => "synoema_float_gt",
                                PrimOp::FLte  => "synoema_float_lte",
                                PrimOp::FGte  => "synoema_float_gte",
                                PrimOp::FEq   => "synoema_float_eq",
                                _ => unreachable!(),
                            };
                            let fid = *funcs.get(fn_name).ok_or_else(|| cerr(format!("{} not declared", fn_name)))?;
                            let fref = module.declare_func_in_func(fid, builder.func);
                            let call = builder.ins().call(fref, &[l, r]);
                            return Ok(builder.inst_results(call)[0]);
                        }
                        // Integer power: call synoema_int_pow(base, exp)
                        PrimOp::Pow => {
                            let l = compile_expr(builder, vars, vc, funcs, module, ctor_tags, lhs, None)?;
                            let r = compile_expr(builder, vars, vc, funcs, module, ctor_tags, arg, None)?;
                            let fid = *funcs.get("synoema_int_pow").ok_or_else(|| cerr("synoema_int_pow not declared"))?;
                            let fref = module.declare_func_in_func(fid, builder.func);
                            let call = builder.ins().call(fref, &[l, r]);
                            return Ok(builder.inst_results(call)[0]);
                        }
                        // Range [from..to]: call synoema_range(from, to) → list
                        PrimOp::Range => {
                            let l = compile_expr(builder, vars, vc, funcs, module, ctor_tags, lhs, None)?;
                            let r = compile_expr(builder, vars, vc, funcs, module, ctor_tags, arg, None)?;
                            let fid = *funcs.get("synoema_range").ok_or_else(|| cerr("synoema_range not declared"))?;
                            let fref = module.declare_func_in_func(fid, builder.func);
                            let call = builder.ins().call(fref, &[l, r]);
                            return Ok(builder.inst_results(call)[0]);
                        }
                        _ => {
                            let l = compile_expr(builder, vars, vc, funcs, module, ctor_tags, lhs, None)?;
                            let r = compile_expr(builder, vars, vc, funcs, module, ctor_tags, arg, None)?;
                            return compile_binop(builder, *op, l, r);
                        }
                    }
                }
            }
            // PrimOp unary: App(PrimOp, arg)
            if let CoreExpr::PrimOp(op) = func.as_ref() {
                let a = compile_expr(builder, vars, vc, funcs, module, ctor_tags, arg, None)?;
                return compile_unop(builder, *op, a);
            }
            // Special case: show / show_bool → type-directed dispatch
            if let CoreExpr::Var(name) = func.as_ref() {
                if name == "show" || name == "show_bool" {
                    // Bool literal: fold at compile time
                    if let CoreExpr::Lit(synoema_parser::Lit::Bool(b)) = arg.as_ref() {
                        let s = if *b { "true" } else { "false" };
                        return compile_str_literal(s, builder, funcs, module);
                    }
                }
                if name == "show" {
                    // Bool expression (comparison, logical op): use synoema_show_bool at runtime
                    if is_bool_expr(arg) {
                        let v = compile_expr(builder, vars, vc, funcs, module, ctor_tags, arg, None)?;
                        let show_bool_id = *funcs.get("synoema_show_bool")
                            .ok_or_else(|| cerr("synoema_show_bool not declared"))?;
                        let show_bool_ref = module.declare_func_in_func(show_bool_id, builder.func);
                        let call = builder.ins().call(show_bool_ref, &[v]);
                        return Ok(builder.inst_results(call)[0]);
                    }
                    // Record literal: field names available at compile time → "{f = v, ...}"
                    if let CoreExpr::Record(fields) = arg.as_ref() {
                        return compile_show_record(builder, vars, vc, funcs, module, ctor_tags, fields);
                    }
                }
            }
            // Function call: flatten App chain for known static calls
            let (callee, args) = flatten_apps(func, arg);
            if let Some(ref name) = callee {
                // TCO: self-recursive tail call → jump to loop header
                if let Some(tco) = tco_ctx {
                    if name == &tco.self_name && args.len() == tco.params.len() {
                        let mut arg_vals = Vec::new();
                        for a in &args {
                            arg_vals.push(compile_expr(builder, vars, vc, funcs, module, ctor_tags, a, None)?);
                        }
                        for (var, val) in tco.params.iter().zip(arg_vals.iter()) {
                            builder.def_var(*var, *val);
                        }
                        // TCO auto-region: exit region before back-edge to free per-iteration heap
                        if tco.emit_regions {
                            let region_exit_id = *funcs.get("synoema_region_exit")
                                .ok_or_else(|| cerr("synoema_region_exit not declared"))?;
                            let region_exit_ref = module.declare_func_in_func(region_exit_id, builder.func);
                            builder.ins().call(region_exit_ref, &[]);
                        }
                        builder.ins().jump(tco.loop_block, &[]);
                        let unreachable = builder.create_block();
                        builder.switch_to_block(unreachable);
                        builder.seal_block(unreachable);
                        return Ok(builder.ins().iconst(types::I64, 0));
                    }
                }
                if let Some(&fid) = funcs.get(name.as_str()) {
                    let mut arg_vals = Vec::new();
                    for a in &args {
                        arg_vals.push(compile_expr(builder, vars, vc, funcs, module, ctor_tags, a, None)?);
                    }
                    let local_func = module.declare_func_in_func(fid, builder.func);
                    let call = builder.ins().call(local_func, &arg_vals);
                    return Ok(builder.inst_results(call)[0]);
                }
                // Name not a known function — fall through to indirect closure call
            }

            // Indirect closure call: compile func to a closure pointer, call via fn_ptr
            // Closures have ABI: fn(env_ptr: i64, arg: i64) -> i64
            let closure_val = compile_expr(builder, vars, vc, funcs, module, ctor_tags, func, None)?;
            let arg_val = compile_expr(builder, vars, vc, funcs, module, ctor_tags, arg, None)?;

            let fn_ptr = builder.ins().load(types::I64, MemFlags::new(), closure_val, 0);
            let env_ptr = builder.ins().load(types::I64, MemFlags::new(), closure_val, 8);

            let mut sig = module.make_signature();
            sig.params.push(AbiParam::new(types::I64)); // env_ptr
            sig.params.push(AbiParam::new(types::I64)); // arg
            sig.returns.push(AbiParam::new(types::I64));
            let sig_ref = builder.import_signature(sig);
            let call = builder.ins().call_indirect(sig_ref, fn_ptr, &[env_ptr, arg_val]);
            Ok(builder.inst_results(call)[0])
        }

        CoreExpr::Case(scrut, alts) => {
            let sv = compile_expr(builder, vars, vc, funcs, module, ctor_tags, scrut, None)?;
            compile_case(builder, vars, vc, funcs, module, ctor_tags, sv, alts, 0, tco_ctx)
        }

        CoreExpr::MkList(elems) => {
            // Build list from right to left: cons(e0, cons(e1, ... cons(eN, nil())...))
            let nil_fn = *funcs.get("synoema_nil").ok_or_else(|| cerr("synoema_nil not declared"))?;
            let cons_fn = *funcs.get("synoema_cons").ok_or_else(|| cerr("synoema_cons not declared"))?;

            let nil_ref = module.declare_func_in_func(nil_fn, builder.func);
            let cons_ref = module.declare_func_in_func(cons_fn, builder.func);

            // Start with nil
            let nil_call = builder.ins().call(nil_ref, &[]);
            let mut list_val = builder.inst_results(nil_call)[0];

            // Cons each element from right to left
            for elem in elems.iter().rev() {
                let elem_val = compile_expr(builder, vars, vc, funcs, module, ctor_tags, elem, None)?;
                let cons_call = builder.ins().call(cons_ref, &[elem_val, list_val]);
                list_val = builder.inst_results(cons_call)[0];
            }
            Ok(list_val)
        }

        CoreExpr::Con(name) if name == "Nil" || name == "[]" => {
            // Nil constructor = empty list
            let nil_fn = *funcs.get("synoema_nil").ok_or_else(|| cerr("synoema_nil not declared"))?;
            let nil_ref = module.declare_func_in_func(nil_fn, builder.func);
            let call = builder.ins().call(nil_ref, &[]);
            Ok(builder.inst_results(call)[0])
        }

        CoreExpr::Con(name) if ctor_tags.contains_key(name) => {
            // User-defined 0-arity constructor: allocate ConNode with no fields
            let &(tag, arity) = ctor_tags.get(name).unwrap();
            let name_bytes = name.as_bytes();
            let mut name_desc = cranelift_module::DataDescription::new();
            name_desc.define(name_bytes.to_vec().into_boxed_slice());
            let name_id = module.declare_anonymous_data(false, false)
                .map_err(|e| cerr(format!("con0 name data: {}", e)))?;
            module.define_data(name_id, &name_desc)
                .map_err(|e| cerr(format!("con0 name define: {}", e)))?;
            let name_ref = module.declare_data_in_func(name_id, builder.func);
            let name_ptr = builder.ins().global_value(types::I64, name_ref);
            let name_len = builder.ins().iconst(types::I64, name_bytes.len() as i64);
            let tag_val = builder.ins().iconst(types::I64, tag);
            let arity_val = builder.ins().iconst(types::I64, arity as i64);
            let make_con_id = *funcs.get("synoema_make_con")
                .ok_or_else(|| cerr("synoema_make_con not declared"))?;
            let make_con_ref = module.declare_func_in_func(make_con_id, builder.func);
            let make_call = builder.ins().call(make_con_ref, &[tag_val, arity_val, name_ptr, name_len]);
            Ok(builder.inst_results(make_call)[0])
        }

        CoreExpr::Lam(_, _) => {
            Err(cerr("Internal: Lam reached JIT codegen (lambda lifting should have removed this)"))
        }

        CoreExpr::MkClosure { func, free_vars } => {
            // Get function pointer for the lifted closure function
            let fid = *funcs.get(func.as_str())
                .ok_or_else(|| cerr(format!("Unknown closure function: {}", func)))?;
            let fref = module.declare_func_in_func(fid, builder.func);
            let fn_ptr = builder.ins().func_addr(types::I64, fref);

            // Allocate environment array and store captured variables
            let env_ptr = if free_vars.is_empty() {
                builder.ins().iconst(types::I64, 0)
            } else {
                let n = builder.ins().iconst(types::I64, free_vars.len() as i64);
                let env_alloc_id = *funcs.get("synoema_env_alloc")
                    .ok_or_else(|| cerr("synoema_env_alloc not declared"))?;
                let env_alloc_ref = module.declare_func_in_func(env_alloc_id, builder.func);
                let call = builder.ins().call(env_alloc_ref, &[n]);
                let ep = builder.inst_results(call)[0];
                for (i, fv) in free_vars.iter().enumerate() {
                    let fv_val = vars.get(fv.as_str())
                        .map(|&v| builder.use_var(v))
                        .ok_or_else(|| cerr(format!("Free var '{}' not in scope for closure '{}'", fv, func)))?;
                    builder.ins().store(MemFlags::new(), fv_val, ep, (i * 8) as i32);
                }
                ep
            };

            // Allocate closure struct { fn_ptr, env_ptr }
            let make_closure_id = *funcs.get("synoema_make_closure")
                .ok_or_else(|| cerr("synoema_make_closure not declared"))?;
            let make_closure_ref = module.declare_func_in_func(make_closure_id, builder.func);
            let call = builder.ins().call(make_closure_ref, &[fn_ptr, env_ptr]);
            Ok(builder.inst_results(call)[0])
        }

        CoreExpr::Record(fields) => {
            // 1. Allocate the record: synoema_record_new(len) -> rec_ptr
            let len_val = builder.ins().iconst(types::I64, fields.len() as i64);
            let rec_new_id = *funcs.get("synoema_record_new")
                .ok_or_else(|| cerr("synoema_record_new not declared"))?;
            let rec_new_ref = module.declare_func_in_func(rec_new_id, builder.func);
            let rec_call = builder.ins().call(rec_new_ref, &[len_val]);
            let rec_ptr = builder.inst_results(rec_call)[0];

            // 2. For each field: compile value, then call synoema_record_set(rec, idx, hash, val)
            let rec_set_id = *funcs.get("synoema_record_set")
                .ok_or_else(|| cerr("synoema_record_set not declared"))?;
            for (i, (name, val_expr)) in fields.iter().enumerate() {
                let val = compile_expr(builder, vars, vc, funcs, module, ctor_tags, val_expr, None)?;
                let idx_val = builder.ins().iconst(types::I64, i as i64);
                let hash = crate::runtime::field_name_hash(name);
                let hash_val = builder.ins().iconst(types::I64, hash);
                let rec_set_ref = module.declare_func_in_func(rec_set_id, builder.func);
                builder.ins().call(rec_set_ref, &[rec_ptr, idx_val, hash_val, val]);
            }

            Ok(rec_ptr)
        }

        CoreExpr::FieldAccess(obj, field) => {
            // 1. Compile object → rec_ptr
            let rec_ptr = compile_expr(builder, vars, vc, funcs, module, ctor_tags, obj, None)?;
            // 2. Hash the field name at compile time
            let hash = crate::runtime::field_name_hash(field);
            let hash_val = builder.ins().iconst(types::I64, hash);
            // 3. Call synoema_record_get(rec_ptr, hash) → value
            let rec_get_id = *funcs.get("synoema_record_get")
                .ok_or_else(|| cerr("synoema_record_get not declared"))?;
            let rec_get_ref = module.declare_func_in_func(rec_get_id, builder.func);
            let get_call = builder.ins().call(rec_get_ref, &[rec_ptr, hash_val]);
            Ok(builder.inst_results(get_call)[0])
        }

        CoreExpr::RecordUpdate { base, updates } => {
            // 1. Compile base → base_ptr
            let base_ptr = compile_expr(builder, vars, vc, funcs, module, ctor_tags, base, None)?;

            // 2. Clone the base record: synoema_record_clone(base_ptr) → new_ptr
            let clone_id = *funcs.get("synoema_record_clone")
                .ok_or_else(|| cerr("synoema_record_clone not declared"))?;
            let clone_ref = module.declare_func_in_func(clone_id, builder.func);
            let clone_call = builder.ins().call(clone_ref, &[base_ptr]);
            let new_ptr = builder.inst_results(clone_call)[0];

            // 3. For each update: compile val, call synoema_record_set_field(new_ptr, hash, val)
            let set_field_id = *funcs.get("synoema_record_set_field")
                .ok_or_else(|| cerr("synoema_record_set_field not declared"))?;
            for (field_name, val_expr) in updates {
                let val = compile_expr(builder, vars, vc, funcs, module, ctor_tags, val_expr, None)?;
                let hash = crate::runtime::field_name_hash(field_name);
                let hash_val = builder.ins().iconst(types::I64, hash);
                let set_ref = module.declare_func_in_func(set_field_id, builder.func);
                builder.ins().call(set_ref, &[new_ptr, hash_val, val]);
            }

            Ok(new_ptr)
        }

        // ── Region inference: enter/exit arena region around body ────────────────
        CoreExpr::Region(body) => {
            let enter_id = *funcs.get("synoema_region_enter")
                .ok_or_else(|| cerr("synoema_region_enter not declared"))?;
            let enter_ref = module.declare_func_in_func(enter_id, builder.func);
            builder.ins().call(enter_ref, &[]);
            let result = compile_expr(builder, vars, vc, funcs, module, ctor_tags, body, tco_ctx)?;
            let exit_id = *funcs.get("synoema_region_exit")
                .ok_or_else(|| cerr("synoema_region_exit not declared"))?;
            let exit_ref = module.declare_func_in_func(exit_id, builder.func);
            builder.ins().call(exit_ref, &[]);
            Ok(result)
        }

        // ── Concurrency — Phase B sequential stubs ───────────────────────────────
        // Scope: evaluate body sequentially (no thread management in JIT yet).
        CoreExpr::Scope(body) => {
            compile_expr(builder, vars, vc, funcs, module, ctor_tags, body, tco_ctx)
        }

        // Spawn: evaluate expr sequentially, discard result → return Unit (0).
        CoreExpr::Spawn(expr) => {
            compile_expr(builder, vars, vc, funcs, module, ctor_tags, expr, None)?;
            Ok(builder.ins().iconst(types::I64, 0))
        }

        // RuntimeError: call FFI to print message and abort.
        CoreExpr::RuntimeError(msg) => {
            let fid = *funcs.get("synoema_match_error")
                .ok_or_else(|| cerr("synoema_match_error not declared"))?;
            let fref = module.declare_func_in_func(fid, builder.func);
            let ptr = builder.ins().iconst(types::I64, msg.as_ptr() as i64);
            let len = builder.ins().iconst(types::I64, msg.len() as i64);
            let call = builder.ins().call(fref, &[ptr, len]);
            Ok(builder.inst_results(call)[0])
        }

        _ => Err(cerr(format!("Unsupported in codegen: {}", expr))),
    }
}

/// Detect App(App(...App(Con(name), arg0)..., arg_{n-2}), arg_{n-1}) for user-defined constructors.
/// Returns (constructor_name, [arg0, ..., arg_{n-1}]) if the innermost callee is a Con in ctor_tags.
fn flatten_con_app<'a>(
    func: &'a CoreExpr,
    last_arg: &'a CoreExpr,
    ctor_tags: &HashMap<String, (i64, usize)>,
) -> Option<(String, Vec<&'a CoreExpr>)> {
    let mut args = vec![last_arg];
    let mut cur = func;
    loop {
        match cur {
            CoreExpr::App(f, a) => { args.push(a.as_ref()); cur = f.as_ref(); }
            CoreExpr::Con(name) if ctor_tags.contains_key(name) => {
                args.reverse();
                return Some((name.clone(), args));
            }
            _ => return None,
        }
    }
}

fn compile_binop(
    builder: &mut FunctionBuilder, op: PrimOp,
    l: cranelift_codegen::ir::Value, r: cranelift_codegen::ir::Value,
) -> CResult<cranelift_codegen::ir::Value> {
    Ok(match op {
        PrimOp::Add => builder.ins().iadd(l, r),
        PrimOp::Sub => builder.ins().isub(l, r),
        PrimOp::Mul => builder.ins().imul(l, r),
        PrimOp::Div => builder.ins().sdiv(l, r),
        PrimOp::Mod => builder.ins().srem(l, r),
        PrimOp::Eq  => { let c = builder.ins().icmp(IntCC::Equal, l, r); builder.ins().uextend(types::I64, c) }
        PrimOp::Neq => { let c = builder.ins().icmp(IntCC::NotEqual, l, r); builder.ins().uextend(types::I64, c) }
        PrimOp::Lt  => { let c = builder.ins().icmp(IntCC::SignedLessThan, l, r); builder.ins().uextend(types::I64, c) }
        PrimOp::Gt  => { let c = builder.ins().icmp(IntCC::SignedGreaterThan, l, r); builder.ins().uextend(types::I64, c) }
        PrimOp::Lte => { let c = builder.ins().icmp(IntCC::SignedLessThanOrEqual, l, r); builder.ins().uextend(types::I64, c) }
        PrimOp::Gte => { let c = builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, l, r); builder.ins().uextend(types::I64, c) }
        PrimOp::And => builder.ins().band(l, r),
        PrimOp::Or  => builder.ins().bor(l, r),
        _ => return Err(cerr(format!("Unsupported binop: {:?}", op))),
    })
}

fn compile_unop(
    builder: &mut FunctionBuilder, op: PrimOp,
    a: cranelift_codegen::ir::Value,
) -> CResult<cranelift_codegen::ir::Value> {
    Ok(match op {
        PrimOp::Neg => { let z = builder.ins().iconst(types::I64, 0); builder.ins().isub(z, a) }
        PrimOp::Not => { let one = builder.ins().iconst(types::I64, 1); builder.ins().bxor(a, one) }
        _ => return Err(cerr(format!("Unsupported unop: {:?}", op))),
    })
}

fn compile_case(
    builder: &mut FunctionBuilder,
    vars: &mut HashMap<String, Variable>,
    vc: &mut u32,
    funcs: &HashMap<String, cranelift_module::FuncId>,
    module: &mut JITModule,
    ctor_tags: &HashMap<String, (i64, usize)>,
    scrut: cranelift_codegen::ir::Value,
    alts: &[Alt],
    idx: usize,
    tco_ctx: Option<&TcoContext>,
) -> CResult<cranelift_codegen::ir::Value> {
    if idx >= alts.len() {
        return Err(cerr("Non-exhaustive patterns"));
    }
    let alt = &alts[idx];
    match &alt.pat {
        CorePat::Wildcard => compile_expr(builder, vars, vc, funcs, module, ctor_tags, &alt.body, tco_ctx),
        CorePat::Var(name) => {
            let var = Variable::from_u32(*vc); *vc += 1;
            builder.declare_var(var, types::I64);
            builder.def_var(var, scrut);
            vars.insert(name.clone(), var);
            compile_expr(builder, vars, vc, funcs, module, ctor_tags, &alt.body, tco_ctx)
        }
        CorePat::Lit(Lit::Bool(true)) if idx + 1 < alts.len() => {
            // if scrut then alt.body else next
            let then_b = builder.create_block();
            let else_b = builder.create_block();
            let merge_b = builder.create_block();
            let rv = Variable::from_u32(*vc); *vc += 1;
            builder.declare_var(rv, types::I64);

            builder.ins().brif(scrut, then_b, &[], else_b, &[]);

            builder.switch_to_block(then_b);
            builder.seal_block(then_b);
            let tv = compile_expr(builder, vars, vc, funcs, module, ctor_tags, &alt.body, tco_ctx)?;
            builder.def_var(rv, tv);
            builder.ins().jump(merge_b, &[]);

            builder.switch_to_block(else_b);
            builder.seal_block(else_b);
            let ev = compile_case(builder, vars, vc, funcs, module, ctor_tags, scrut, alts, idx+1, tco_ctx)?;
            builder.def_var(rv, ev);
            builder.ins().jump(merge_b, &[]);

            builder.switch_to_block(merge_b);
            builder.seal_block(merge_b);
            Ok(builder.use_var(rv))
        }
        CorePat::Lit(Lit::Bool(false)) => {
            compile_expr(builder, vars, vc, funcs, module, ctor_tags, &alt.body, tco_ctx)
        }
        CorePat::Lit(Lit::Int(n)) => {
            let lit = builder.ins().iconst(types::I64, *n);
            let cmp = builder.ins().icmp(IntCC::Equal, scrut, lit);

            if idx + 1 < alts.len() {
                let then_b = builder.create_block();
                let else_b = builder.create_block();
                let merge_b = builder.create_block();
                let rv = Variable::from_u32(*vc); *vc += 1;
                builder.declare_var(rv, types::I64);

                builder.ins().brif(cmp, then_b, &[], else_b, &[]);

                builder.switch_to_block(then_b);
                builder.seal_block(then_b);
                let tv = compile_expr(builder, vars, vc, funcs, module, ctor_tags, &alt.body, tco_ctx)?;
                builder.def_var(rv, tv);
                builder.ins().jump(merge_b, &[]);

                builder.switch_to_block(else_b);
                builder.seal_block(else_b);
                let ev = compile_case(builder, vars, vc, funcs, module, ctor_tags, scrut, alts, idx+1, tco_ctx)?;
                builder.def_var(rv, ev);
                builder.ins().jump(merge_b, &[]);

                builder.switch_to_block(merge_b);
                builder.seal_block(merge_b);
                Ok(builder.use_var(rv))
            } else {
                compile_expr(builder, vars, vc, funcs, module, ctor_tags, &alt.body, tco_ctx)
            }
        }
        CorePat::Lit(Lit::Str(s)) => {
            // Pre-allocate the pattern string in the arena at compile time so it stays
            // alive for the duration of this compile_and_run invocation.
            let bytes = s.as_bytes();
            let tagged_pattern = runtime::synoema_str_new(bytes.as_ptr() as i64, bytes.len() as i64);
            let pattern_val = builder.ins().iconst(types::I64, tagged_pattern);
            let val_eq_id = *funcs.get("synoema_val_eq")
                .ok_or_else(|| cerr("synoema_val_eq not declared"))?;
            let fref = module.declare_func_in_func(val_eq_id, builder.func);
            let cmp_call = builder.ins().call(fref, &[scrut, pattern_val]);
            let cmp = builder.inst_results(cmp_call)[0];

            if idx + 1 < alts.len() {
                let then_b = builder.create_block();
                let else_b = builder.create_block();
                let merge_b = builder.create_block();
                let rv = Variable::from_u32(*vc); *vc += 1;
                builder.declare_var(rv, types::I64);

                builder.ins().brif(cmp, then_b, &[], else_b, &[]);

                builder.switch_to_block(then_b);
                builder.seal_block(then_b);
                let tv = compile_expr(builder, vars, vc, funcs, module, ctor_tags, &alt.body, tco_ctx)?;
                builder.def_var(rv, tv);
                builder.ins().jump(merge_b, &[]);

                builder.switch_to_block(else_b);
                builder.seal_block(else_b);
                let ev = compile_case(builder, vars, vc, funcs, module, ctor_tags, scrut, alts, idx+1, tco_ctx)?;
                builder.def_var(rv, ev);
                builder.ins().jump(merge_b, &[]);

                builder.switch_to_block(merge_b);
                builder.seal_block(merge_b);
                Ok(builder.use_var(rv))
            } else {
                compile_expr(builder, vars, vc, funcs, module, ctor_tags, &alt.body, tco_ctx)
            }
        }
        // Constructor patterns: Nil and Cons for lists
        CorePat::Con(name, sub_pats) if (name == "Nil" || name == "[]") && sub_pats.is_empty() => {
            // Nil pattern: check if scrut == 0 (null pointer = empty list)
            let nil_val = builder.ins().iconst(types::I64, 0);
            let cmp = builder.ins().icmp(IntCC::Equal, scrut, nil_val);

            if idx + 1 < alts.len() {
                let then_b = builder.create_block();
                let else_b = builder.create_block();
                let merge_b = builder.create_block();
                let rv = Variable::from_u32(*vc); *vc += 1;
                builder.declare_var(rv, types::I64);

                builder.ins().brif(cmp, then_b, &[], else_b, &[]);

                builder.switch_to_block(then_b);
                builder.seal_block(then_b);
                let tv = compile_expr(builder, vars, vc, funcs, module, ctor_tags, &alt.body, tco_ctx)?;
                builder.def_var(rv, tv);
                builder.ins().jump(merge_b, &[]);

                builder.switch_to_block(else_b);
                builder.seal_block(else_b);
                let ev = compile_case(builder, vars, vc, funcs, module, ctor_tags, scrut, alts, idx+1, tco_ctx)?;
                builder.def_var(rv, ev);
                builder.ins().jump(merge_b, &[]);

                builder.switch_to_block(merge_b);
                builder.seal_block(merge_b);
                Ok(builder.use_var(rv))
            } else {
                compile_expr(builder, vars, vc, funcs, module, ctor_tags, &alt.body, tco_ctx)
            }
        }

        CorePat::Con(name, sub_pats) if name == "Cons" && sub_pats.len() == 2 => {
            // Cons pattern: check if scrut != 0, extract head and tail
            let nil_val = builder.ins().iconst(types::I64, 0);
            let is_cons = builder.ins().icmp(IntCC::NotEqual, scrut, nil_val);

            let then_b = builder.create_block();
            let else_b = builder.create_block();
            let merge_b = builder.create_block();
            let rv = Variable::from_u32(*vc); *vc += 1;
            builder.declare_var(rv, types::I64);

            builder.ins().brif(is_cons, then_b, &[], else_b, &[]);

            // Cons branch: extract head and tail, bind sub-patterns
            builder.switch_to_block(then_b);
            builder.seal_block(then_b);

            let head_fn = *funcs.get("synoema_head").ok_or_else(|| cerr("synoema_head"))?;
            let tail_fn = *funcs.get("synoema_tail").ok_or_else(|| cerr("synoema_tail"))?;
            let head_ref = module.declare_func_in_func(head_fn, builder.func);
            let tail_ref = module.declare_func_in_func(tail_fn, builder.func);

            let head_call = builder.ins().call(head_ref, &[scrut]);
            let head_val = builder.inst_results(head_call)[0];
            let tail_call = builder.ins().call(tail_ref, &[scrut]);
            let tail_val = builder.inst_results(tail_call)[0];

            bind_sub_pat(builder, vars, vc, funcs, module, ctor_tags, head_val, &sub_pats[0], else_b)?;
            bind_sub_pat(builder, vars, vc, funcs, module, ctor_tags, tail_val, &sub_pats[1], else_b)?;

            let tv = compile_expr(builder, vars, vc, funcs, module, ctor_tags, &alt.body, tco_ctx)?;
            builder.def_var(rv, tv);
            builder.ins().jump(merge_b, &[]);

            // Else branch: try next alternative
            builder.switch_to_block(else_b);
            builder.seal_block(else_b);
            let ev = if idx + 1 < alts.len() {
                compile_case(builder, vars, vc, funcs, module, ctor_tags, scrut, alts, idx+1, tco_ctx)?
            } else {
                builder.ins().iconst(types::I64, 0) // fallback
            };
            builder.def_var(rv, ev);
            builder.ins().jump(merge_b, &[]);

            builder.switch_to_block(merge_b);
            builder.seal_block(merge_b);
            Ok(builder.use_var(rv))
        }

        CorePat::Record(field_pats) => {
            // Record pattern: always matches (no tag check needed).
            // For each field, extract the value via synoema_record_get and bind via bind_sub_pat.
            // We need an else_b for sub-pattern failures (e.g., literal sub-patterns).
            let else_b = builder.create_block();
            let merge_b = builder.create_block();
            let rv = Variable::from_u32(*vc); *vc += 1;
            builder.declare_var(rv, types::I64);

            let rec_get_id = *funcs.get("synoema_record_get")
                .ok_or_else(|| cerr("synoema_record_get not declared"))?;
            for (field_name, field_pat) in field_pats {
                let hash = crate::runtime::field_name_hash(field_name);
                let hash_val = builder.ins().iconst(types::I64, hash);
                let rec_get_ref = module.declare_func_in_func(rec_get_id, builder.func);
                let call = builder.ins().call(rec_get_ref, &[scrut, hash_val]);
                let field_val = builder.inst_results(call)[0];
                bind_sub_pat(builder, vars, vc, funcs, module, ctor_tags, field_val, field_pat, else_b)?;
            }

            let tv = compile_expr(builder, vars, vc, funcs, module, ctor_tags, &alt.body, tco_ctx)?;
            builder.def_var(rv, tv);
            builder.ins().jump(merge_b, &[]);

            // Else branch: try next alternative
            builder.switch_to_block(else_b);
            builder.seal_block(else_b);
            let ev = if idx + 1 < alts.len() {
                compile_case(builder, vars, vc, funcs, module, ctor_tags, scrut, alts, idx + 1, tco_ctx)?
            } else {
                builder.ins().iconst(types::I64, 0) // non-exhaustive fallback
            };
            builder.def_var(rv, ev);
            builder.ins().jump(merge_b, &[]);

            builder.switch_to_block(merge_b);
            builder.seal_block(merge_b);
            Ok(builder.use_var(rv))
        }

        CorePat::Con(name, sub_pats) if ctor_tags.contains_key(name) => {
            // User-defined constructor pattern: compare tag, bind sub-patterns
            let &(expected_tag, _arity) = ctor_tags.get(name).unwrap();
            let expected_tag_val = builder.ins().iconst(types::I64, expected_tag);

            // Load tag from ptr[0] via synoema_con_get_tag
            let get_tag_id = *funcs.get("synoema_con_get_tag")
                .ok_or_else(|| cerr("synoema_con_get_tag not declared"))?;
            let get_tag_ref = module.declare_func_in_func(get_tag_id, builder.func);
            let tag_call = builder.ins().call(get_tag_ref, &[scrut]);
            let actual_tag = builder.inst_results(tag_call)[0];

            let cmp = builder.ins().icmp(IntCC::Equal, actual_tag, expected_tag_val);

            let then_b = builder.create_block();
            let else_b = builder.create_block();
            let merge_b = builder.create_block();
            let rv = Variable::from_u32(*vc); *vc += 1;
            builder.declare_var(rv, types::I64);

            builder.ins().brif(cmp, then_b, &[], else_b, &[]);

            // Then branch: bind sub-patterns, compile body
            builder.switch_to_block(then_b);
            builder.seal_block(then_b);

            let get_field_id = *funcs.get("synoema_con_get_field")
                .ok_or_else(|| cerr("synoema_con_get_field not declared"))?;
            for (i, sub_pat) in sub_pats.iter().enumerate() {
                let idx_val = builder.ins().iconst(types::I64, i as i64);
                let get_field_ref = module.declare_func_in_func(get_field_id, builder.func);
                let field_call = builder.ins().call(get_field_ref, &[scrut, idx_val]);
                let field_val = builder.inst_results(field_call)[0];
                bind_sub_pat(builder, vars, vc, funcs, module, ctor_tags, field_val, sub_pat, else_b)?;
            }

            let tv = compile_expr(builder, vars, vc, funcs, module, ctor_tags, &alt.body, tco_ctx)?;
            builder.def_var(rv, tv);
            builder.ins().jump(merge_b, &[]);

            // Else branch: try next alternative
            builder.switch_to_block(else_b);
            builder.seal_block(else_b);
            let ev = if idx + 1 < alts.len() {
                compile_case(builder, vars, vc, funcs, module, ctor_tags, scrut, alts, idx + 1, tco_ctx)?
            } else {
                builder.ins().iconst(types::I64, 0) // non-exhaustive fallback
            };
            builder.def_var(rv, ev);
            builder.ins().jump(merge_b, &[]);

            builder.switch_to_block(merge_b);
            builder.seal_block(merge_b);
            Ok(builder.use_var(rv))
        }

        _ => Err(cerr(format!("Unsupported pattern: {:?}", alt.pat))),
    }
}

/// Recursively bind a sub-pattern `pat` against `val`.
///
/// On mismatch, jumps to `else_b` (terminating the current block).
/// On success, returns `Ok(())` with the builder positioned in a continuation block
/// (for Wildcard/Var: stays in the same block; for conditional patterns: creates a fresh block).
#[allow(clippy::too_many_arguments)]
fn bind_sub_pat(
    builder: &mut FunctionBuilder,
    vars: &mut HashMap<String, Variable>,
    vc: &mut u32,
    funcs: &HashMap<String, cranelift_module::FuncId>,
    module: &mut JITModule,
    ctor_tags: &HashMap<String, (i64, usize)>,
    val: cranelift_codegen::ir::Value,
    pat: &CorePat,
    else_b: cranelift_codegen::ir::Block,
) -> CResult<()> {
    match pat {
        CorePat::Wildcard => Ok(()),

        CorePat::Var(name) => {
            let var = Variable::from_u32(*vc); *vc += 1;
            builder.declare_var(var, types::I64);
            builder.def_var(var, val);
            vars.insert(name.clone(), var);
            Ok(())
        }

        CorePat::Lit(Lit::Int(n)) => {
            let lit = builder.ins().iconst(types::I64, *n);
            let cmp = builder.ins().icmp(IntCC::Equal, val, lit);
            let cont_b = builder.create_block();
            builder.ins().brif(cmp, cont_b, &[], else_b, &[]);
            builder.switch_to_block(cont_b);
            builder.seal_block(cont_b);
            Ok(())
        }

        CorePat::Lit(Lit::Bool(b)) => {
            let cont_b = builder.create_block();
            if *b {
                builder.ins().brif(val, cont_b, &[], else_b, &[]);
            } else {
                let zero = builder.ins().iconst(types::I64, 0);
                let cmp = builder.ins().icmp(IntCC::Equal, val, zero);
                builder.ins().brif(cmp, cont_b, &[], else_b, &[]);
            }
            builder.switch_to_block(cont_b);
            builder.seal_block(cont_b);
            Ok(())
        }

        CorePat::Lit(Lit::Str(s)) => {
            let bytes = s.as_bytes();
            let tagged_pattern = runtime::synoema_str_new(bytes.as_ptr() as i64, bytes.len() as i64);
            let pattern_val = builder.ins().iconst(types::I64, tagged_pattern);
            let val_eq_id = *funcs.get("synoema_val_eq")
                .ok_or_else(|| cerr("synoema_val_eq not declared"))?;
            let fref = module.declare_func_in_func(val_eq_id, builder.func);
            let cmp_call = builder.ins().call(fref, &[val, pattern_val]);
            let cmp = builder.inst_results(cmp_call)[0];
            let cont_b = builder.create_block();
            builder.ins().brif(cmp, cont_b, &[], else_b, &[]);
            builder.switch_to_block(cont_b);
            builder.seal_block(cont_b);
            Ok(())
        }

        CorePat::Con(name, sub_pats) if ctor_tags.contains_key(name.as_str()) => {
            let &(expected_tag, _) = ctor_tags.get(name.as_str()).unwrap();
            let expected_tag_val = builder.ins().iconst(types::I64, expected_tag);

            let get_tag_id = *funcs.get("synoema_con_get_tag")
                .ok_or_else(|| cerr("synoema_con_get_tag not declared"))?;
            let get_tag_ref = module.declare_func_in_func(get_tag_id, builder.func);
            let tag_call = builder.ins().call(get_tag_ref, &[val]);
            let actual_tag = builder.inst_results(tag_call)[0];

            let cmp = builder.ins().icmp(IntCC::Equal, actual_tag, expected_tag_val);
            let cont_b = builder.create_block();
            builder.ins().brif(cmp, cont_b, &[], else_b, &[]);
            builder.switch_to_block(cont_b);
            builder.seal_block(cont_b);

            let get_field_id = *funcs.get("synoema_con_get_field")
                .ok_or_else(|| cerr("synoema_con_get_field not declared"))?;
            for (i, sub_sub) in sub_pats.iter().enumerate() {
                let idx_val = builder.ins().iconst(types::I64, i as i64);
                let gf_ref = module.declare_func_in_func(get_field_id, builder.func);
                let fc = builder.ins().call(gf_ref, &[val, idx_val]);
                let fv = builder.inst_results(fc)[0];
                bind_sub_pat(builder, vars, vc, funcs, module, ctor_tags, fv, sub_sub, else_b)?;
            }
            Ok(())
        }

        CorePat::Record(field_pats) => {
            // Nested record sub-pattern: extract each field and recursively bind.
            let rec_get_id = *funcs.get("synoema_record_get")
                .ok_or_else(|| cerr("synoema_record_get not declared"))?;
            for (field_name, field_pat) in field_pats {
                let hash = crate::runtime::field_name_hash(field_name);
                let hash_val = builder.ins().iconst(types::I64, hash);
                let rec_get_ref = module.declare_func_in_func(rec_get_id, builder.func);
                let call = builder.ins().call(rec_get_ref, &[val, hash_val]);
                let field_val = builder.inst_results(call)[0];
                bind_sub_pat(builder, vars, vc, funcs, module, ctor_tags, field_val, field_pat, else_b)?;
            }
            Ok(())
        }

        _ => Err(cerr(format!("Unsupported sub-pattern: {:?}", pat))),
    }
}

// ── Helpers ─────────────────────────────────────────────

fn count_lambdas(e: &CoreExpr) -> usize {
    match e { CoreExpr::Lam(_, b) => 1 + count_lambdas(b), _ => 0 }
}

fn peel_lambdas(e: &CoreExpr) -> (Vec<String>, &CoreExpr) {
    let mut params = Vec::new();
    let mut cur = e;
    while let CoreExpr::Lam(n, b) = cur { params.push(n.clone()); cur = b; }
    (params, cur)
}

fn flatten_apps<'a>(func: &'a CoreExpr, last: &'a CoreExpr) -> (Option<String>, Vec<&'a CoreExpr>) {
    let mut args = vec![last];
    let mut cur = func;
    loop {
        match cur {
            CoreExpr::App(f, a) => { args.push(a.as_ref()); cur = f.as_ref(); }
            CoreExpr::Var(n) => { args.reverse(); return (Some(n.clone()), args); }
            _ => { args.reverse(); return (None, args); }
        }
    }
}

// ── Lambda Lifting ───────────────────────────────────────
//
// Lifts all inner Lam expressions to top-level closure functions.
// Returns a modified expression where every Lam is replaced with MkClosure,
// and populates `lifted` with the info needed to compile each closure function.

fn lift_expr(
    expr: &CoreExpr,
    bound: &std::collections::HashSet<String>,
    globals: &std::collections::HashSet<String>,
    lifted: &mut HashMap<String, (Vec<String>, String, CoreExpr)>,
    counter: &mut u32,
) -> CoreExpr {
    match expr {
        CoreExpr::Lam(param, body) => {
            let mut new_bound = bound.clone();
            new_bound.insert(param.clone());
            let new_body = lift_expr(body, &new_bound, globals, lifted, counter);

            // Compute which outer-scope vars are captured in the body
            let free_vars = collect_free_vars(&new_body, bound, globals);

            let name = format!("__closure${}", *counter);
            *counter += 1;
            lifted.insert(name.clone(), (free_vars.clone(), param.clone(), new_body));
            CoreExpr::MkClosure { func: name, free_vars }
        }

        CoreExpr::App(f, a) => CoreExpr::App(
            Box::new(lift_expr(f, bound, globals, lifted, counter)),
            Box::new(lift_expr(a, bound, globals, lifted, counter)),
        ),

        CoreExpr::Let(n, v, b) => {
            let new_v = lift_expr(v, bound, globals, lifted, counter);
            let mut new_bound = bound.clone();
            new_bound.insert(n.clone());
            let new_b = lift_expr(b, &new_bound, globals, lifted, counter);
            CoreExpr::Let(n.clone(), Box::new(new_v), Box::new(new_b))
        }

        CoreExpr::LetRec(n, v, b) => {
            let mut new_bound = bound.clone();
            new_bound.insert(n.clone());
            let new_v = lift_expr(v, &new_bound, globals, lifted, counter);
            let new_b = lift_expr(b, &new_bound, globals, lifted, counter);
            CoreExpr::LetRec(n.clone(), Box::new(new_v), Box::new(new_b))
        }

        CoreExpr::Case(scrut, alts) => {
            let new_scrut = lift_expr(scrut, bound, globals, lifted, counter);
            let new_alts = alts.iter().map(|alt| {
                let mut new_bound = bound.clone();
                collect_pat_vars(&alt.pat, &mut new_bound);
                Alt {
                    pat: alt.pat.clone(),
                    body: lift_expr(&alt.body, &new_bound, globals, lifted, counter),
                }
            }).collect();
            CoreExpr::Case(Box::new(new_scrut), new_alts)
        }

        CoreExpr::MkList(elems) => CoreExpr::MkList(
            elems.iter().map(|e| lift_expr(e, bound, globals, lifted, counter)).collect()
        ),

        CoreExpr::Record(fields) => CoreExpr::Record(
            fields.iter().map(|(name, val)| {
                (name.clone(), lift_expr(val, bound, globals, lifted, counter))
            }).collect()
        ),

        CoreExpr::FieldAccess(obj, field) => CoreExpr::FieldAccess(
            Box::new(lift_expr(obj, bound, globals, lifted, counter)),
            field.clone(),
        ),

        CoreExpr::Region(body) => CoreExpr::Region(Box::new(
            lift_expr(body, bound, globals, lifted, counter)
        )),
        CoreExpr::Scope(body) => CoreExpr::Scope(Box::new(
            lift_expr(body, bound, globals, lifted, counter)
        )),
        CoreExpr::Spawn(expr) => CoreExpr::Spawn(Box::new(
            lift_expr(expr, bound, globals, lifted, counter)
        )),

        // Leaf nodes — no lambdas inside (RuntimeError, Lit, Con, PrimOp)
        other => other.clone(),
    }
}

/// Collect variables that are referenced in `expr`, are in `bound` (outer scope),
/// and are NOT globals. These are the free variables a closure must capture.
fn collect_free_vars(
    expr: &CoreExpr,
    bound: &std::collections::HashSet<String>,
    globals: &std::collections::HashSet<String>,
) -> Vec<String> {
    let mut result = std::collections::HashSet::new();
    collect_free_vars_inner(expr, bound, globals, &std::collections::HashSet::new(), &mut result);
    let mut v: Vec<String> = result.into_iter().collect();
    v.sort(); // deterministic order
    v
}

fn collect_free_vars_inner(
    expr: &CoreExpr,
    outer_bound: &std::collections::HashSet<String>,
    globals: &std::collections::HashSet<String>,
    locally_bound: &std::collections::HashSet<String>,
    result: &mut std::collections::HashSet<String>,
) {
    match expr {
        CoreExpr::Var(name) => {
            if outer_bound.contains(name)
                && !locally_bound.contains(name)
                && !globals.contains(name)
            {
                result.insert(name.clone());
            }
        }
        CoreExpr::App(f, a) => {
            collect_free_vars_inner(f, outer_bound, globals, locally_bound, result);
            collect_free_vars_inner(a, outer_bound, globals, locally_bound, result);
        }
        CoreExpr::Lam(param, body) => {
            let mut lb = locally_bound.clone();
            lb.insert(param.clone());
            collect_free_vars_inner(body, outer_bound, globals, &lb, result);
        }
        CoreExpr::Let(n, v, b) | CoreExpr::LetRec(n, v, b) => {
            collect_free_vars_inner(v, outer_bound, globals, locally_bound, result);
            let mut lb = locally_bound.clone();
            lb.insert(n.clone());
            collect_free_vars_inner(b, outer_bound, globals, &lb, result);
        }
        CoreExpr::Case(scrut, alts) => {
            collect_free_vars_inner(scrut, outer_bound, globals, locally_bound, result);
            for alt in alts {
                let mut lb = locally_bound.clone();
                collect_pat_vars(&alt.pat, &mut lb);
                collect_free_vars_inner(&alt.body, outer_bound, globals, &lb, result);
            }
        }
        CoreExpr::MkList(elems) => {
            for e in elems {
                collect_free_vars_inner(e, outer_bound, globals, locally_bound, result);
            }
        }
        CoreExpr::MkClosure { free_vars, .. } => {
            // A nested closure's free vars that come from outer_bound propagate up
            for fv in free_vars {
                if outer_bound.contains(fv)
                    && !locally_bound.contains(fv)
                    && !globals.contains(fv)
                {
                    result.insert(fv.clone());
                }
            }
        }
        CoreExpr::Record(fields) => {
            for (_, val) in fields {
                collect_free_vars_inner(val, outer_bound, globals, locally_bound, result);
            }
        }
        CoreExpr::FieldAccess(obj, _) => {
            collect_free_vars_inner(obj, outer_bound, globals, locally_bound, result);
        }
        CoreExpr::Region(body) => {
            collect_free_vars_inner(body, outer_bound, globals, locally_bound, result);
        }
        CoreExpr::Scope(body) => {
            collect_free_vars_inner(body, outer_bound, globals, locally_bound, result);
        }
        CoreExpr::Spawn(expr) => {
            collect_free_vars_inner(expr, outer_bound, globals, locally_bound, result);
        }
        _ => {} // Lit, Con, PrimOp
    }
}

/// Emit a static string literal as a tagged string pointer via synoema_str_new.
fn compile_str_literal(
    s: &str,
    builder: &mut FunctionBuilder,
    funcs: &HashMap<String, cranelift_module::FuncId>,
    module: &mut JITModule,
) -> CResult<cranelift_codegen::ir::Value> {
    let bytes = s.as_bytes();
    let mut desc = cranelift_module::DataDescription::new();
    desc.define(bytes.to_vec().into_boxed_slice());
    let data_id = module.declare_anonymous_data(false, false)
        .map_err(|e| cerr(format!("str_lit data: {}", e)))?;
    module.define_data(data_id, &desc)
        .map_err(|e| cerr(format!("str_lit define: {}", e)))?;
    let data_ref = module.declare_data_in_func(data_id, builder.func);
    let data_ptr = builder.ins().global_value(types::I64, data_ref);
    let len_val  = builder.ins().iconst(types::I64, bytes.len() as i64);
    let str_new  = *funcs.get("synoema_str_new").ok_or_else(|| cerr("synoema_str_new"))?;
    let str_ref  = module.declare_func_in_func(str_new, builder.func);
    let call = builder.ins().call(str_ref, &[data_ptr, len_val]);
    Ok(builder.inst_results(call)[0])
}

/// Compile-time expansion of show {f1=v1, f2=v2, ...} → "{f1 = v1, f2 = v2}"
fn compile_show_record(
    builder: &mut FunctionBuilder,
    vars: &mut HashMap<String, Variable>,
    vc: &mut u32,
    funcs: &HashMap<String, cranelift_module::FuncId>,
    module: &mut JITModule,
    ctor_tags: &HashMap<String, (i64, usize)>,
    fields: &[(String, CoreExpr)],
) -> CResult<cranelift_codegen::ir::Value> {
    // Build "{f1 = v1, f2 = v2}" by concatenating string pieces
    let str_concat = *funcs.get("synoema_str_concat").ok_or_else(|| cerr("synoema_str_concat"))?;
    let show_fn    = *funcs.get("synoema_show_any").ok_or_else(|| cerr("synoema_show_any"))?;

    let mut acc = compile_str_literal("{", builder, funcs, module)?;
    for (i, (fname, fexpr)) in fields.iter().enumerate() {
        if i > 0 {
            let sep = compile_str_literal(", ", builder, funcs, module)?;
            let r = module.declare_func_in_func(str_concat, builder.func);
            let c = builder.ins().call(r, &[acc, sep]); acc = builder.inst_results(c)[0];
        }
        // "fname = "
        let label = compile_str_literal(&format!("{} = ", fname), builder, funcs, module)?;
        let r = module.declare_func_in_func(str_concat, builder.func);
        let c = builder.ins().call(r, &[acc, label]); acc = builder.inst_results(c)[0];
        // show(value)
        let fval = compile_expr(builder, vars, vc, funcs, module, ctor_tags, fexpr, None)?;
        let show_ref = module.declare_func_in_func(show_fn, builder.func);
        let show_call = builder.ins().call(show_ref, &[fval]);
        let shown = builder.inst_results(show_call)[0];
        let r = module.declare_func_in_func(str_concat, builder.func);
        let c = builder.ins().call(r, &[acc, shown]); acc = builder.inst_results(c)[0];
    }
    let close = compile_str_literal("}", builder, funcs, module)?;
    let r = module.declare_func_in_func(str_concat, builder.func);
    let c = builder.ins().call(r, &[acc, close]); acc = builder.inst_results(c)[0];
    Ok(acc)
}

/// Returns true if the expression always produces a Bool (0 or 1) at runtime.
/// Used to dispatch `show (bool_expr)` to synoema_show_bool instead of synoema_show_any.
fn is_bool_expr(expr: &CoreExpr) -> bool {
    match expr {
        CoreExpr::Lit(Lit::Bool(_)) => true,
        // Binary bool ops: App(App(PrimOp(op), lhs), rhs)
        CoreExpr::App(func, _) => {
            if let CoreExpr::App(inner, _) = func.as_ref() {
                if let CoreExpr::PrimOp(op) = inner.as_ref() {
                    return matches!(op,
                        PrimOp::Eq | PrimOp::Neq |
                        PrimOp::Lt | PrimOp::Gt | PrimOp::Lte | PrimOp::Gte |
                        PrimOp::And | PrimOp::Or |
                        PrimOp::FEq | PrimOp::FLt | PrimOp::FGt | PrimOp::FLte | PrimOp::FGte
                    );
                }
            }
            // Unary not: App(PrimOp(Not), arg)
            if let CoreExpr::PrimOp(op) = func.as_ref() {
                return matches!(op, PrimOp::Not);
            }
            false
        }
        _ => false,
    }
}

/// Add all variables bound by a pattern into `bound`.
fn collect_pat_vars(pat: &CorePat, bound: &mut std::collections::HashSet<String>) {
    match pat {
        CorePat::Var(n) => { bound.insert(n.clone()); }
        CorePat::Con(_, sub_pats) => {
            for p in sub_pats { collect_pat_vars(p, bound); }
        }
        CorePat::Record(field_pats) => {
            for (_, p) in field_pats { collect_pat_vars(p, bound); }
        }
        _ => {}
    }
}
