//! Synoema → Cranelift native code compiler
//!
//! Compiles Core IR to native machine code using Cranelift JIT.

use std::collections::HashMap;

use cranelift_codegen::ir::{types, AbiParam, InstBuilder, condcodes::IntCC};
use cranelift_codegen::settings::{self, Configurable};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};

use synoema_core::*;
use synoema_parser::Lit;

#[derive(Debug)]
pub struct CompileError(pub String);

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Compile error: {}", self.0)
    }
}

type CResult<T> = Result<T, CompileError>;
fn cerr(msg: impl Into<String>) -> CompileError { CompileError(msg.into()) }

/// The Synoema JIT compiler
pub struct Compiler {
    module: JITModule,
    ctx: cranelift_codegen::Context,
    functions: HashMap<String, cranelift_module::FuncId>,
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

        let module = JITModule::new(builder);

        Ok(Compiler {
            ctx: module.make_context(),
            module,
            functions: HashMap::new(),
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

        let decl = |this: &mut Self, name: &str, alias: &str, sig: &cranelift_codegen::ir::Signature| -> CResult<()> {
            let id = this.module.declare_function(name, Linkage::Import, sig)
                .map_err(|e| cerr(format!("Declare '{}': {}", name, e)))?;
            this.functions.insert(alias.to_string(), id);
            Ok(())
        };

        // fn() -> i64
        decl(self, "synoema_nil", "synoema_nil", &sig0)?;
        // fn(i64) -> i64
        decl(self, "synoema_print_int", "show", &sig1)?;
        decl(self, "synoema_print_int", "print", &sig1)?;
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
        // fn(i64, i64) -> i64
        decl(self, "synoema_cons", "synoema_cons", &sig2)?;
        decl(self, "synoema_concat", "synoema_concat", &sig2)?;

        Ok(())
    }

    /// Compile a program and return the result of main()
    pub fn compile_and_run(&mut self, program: &CoreProgram) -> CResult<i64> {
        // Declare runtime functions (FFI)
        self.declare_runtime_functions()?;

        // Pass 1: declare all user functions
        for def in &program.defs {
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

        // Pass 2: define all functions
        for def in &program.defs {
            self.compile_function(&def.name, &def.body)?;
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

        for (i, pname) in params.iter().enumerate() {
            let var = Variable::from_u32(vc);
            vc += 1;
            builder.declare_var(var, types::I64);
            let pval = builder.block_params(entry)[i];
            builder.def_var(var, pval);
            vars.insert(pname.clone(), var);
        }

        let result = compile_expr(
            &mut builder, &mut vars, &mut vc,
            &self.functions, &mut self.module, inner,
        )?;

        builder.ins().return_(&[result]);
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
    expr: &CoreExpr,
) -> CResult<cranelift_codegen::ir::Value> {
    match expr {
        CoreExpr::Lit(Lit::Int(n)) => Ok(builder.ins().iconst(types::I64, *n)),
        CoreExpr::Lit(Lit::Bool(b)) => Ok(builder.ins().iconst(types::I64, if *b {1} else {0})),
        CoreExpr::Lit(_) => Err(cerr("Only Int/Bool literals supported in codegen")),

        CoreExpr::Var(name) => {
            if let Some(&var) = vars.get(name) {
                Ok(builder.use_var(var))
            } else {
                Err(cerr(format!("Undefined: {}", name)))
            }
        }

        CoreExpr::Let(name, val, body) | CoreExpr::LetRec(name, val, body) => {
            let v = compile_expr(builder, vars, vc, funcs, module, val)?;
            let var = Variable::from_u32(*vc);
            *vc += 1;
            builder.declare_var(var, types::I64);
            builder.def_var(var, v);
            vars.insert(name.clone(), var);
            compile_expr(builder, vars, vc, funcs, module, body)
        }

        CoreExpr::App(func, arg) => {
            // PrimOp binary: App(App(PrimOp, lhs), rhs)
            if let CoreExpr::App(inner, lhs) = func.as_ref() {
                if let CoreExpr::PrimOp(op) = inner.as_ref() {
                    // Cons and Concat need runtime calls
                    match op {
                        PrimOp::Cons => {
                            let l = compile_expr(builder, vars, vc, funcs, module, lhs)?;
                            let r = compile_expr(builder, vars, vc, funcs, module, arg)?;
                            let fid = *funcs.get("synoema_cons").ok_or_else(|| cerr("synoema_cons"))?;
                            let fref = module.declare_func_in_func(fid, builder.func);
                            let call = builder.ins().call(fref, &[l, r]);
                            return Ok(builder.inst_results(call)[0]);
                        }
                        PrimOp::Concat => {
                            let l = compile_expr(builder, vars, vc, funcs, module, lhs)?;
                            let r = compile_expr(builder, vars, vc, funcs, module, arg)?;
                            let fid = *funcs.get("synoema_concat").ok_or_else(|| cerr("synoema_concat"))?;
                            let fref = module.declare_func_in_func(fid, builder.func);
                            let call = builder.ins().call(fref, &[l, r]);
                            return Ok(builder.inst_results(call)[0]);
                        }
                        _ => {
                            let l = compile_expr(builder, vars, vc, funcs, module, lhs)?;
                            let r = compile_expr(builder, vars, vc, funcs, module, arg)?;
                            return compile_binop(builder, *op, l, r);
                        }
                    }
                }
            }
            // PrimOp unary: App(PrimOp, arg)
            if let CoreExpr::PrimOp(op) = func.as_ref() {
                let a = compile_expr(builder, vars, vc, funcs, module, arg)?;
                return compile_unop(builder, *op, a);
            }
            // Function call: flatten App chain
            let (callee, args) = flatten_apps(func, arg);
            if let Some(name) = callee {
                if let Some(&fid) = funcs.get(&name) {
                    let mut arg_vals = Vec::new();
                    for a in &args {
                        arg_vals.push(compile_expr(builder, vars, vc, funcs, module, a)?);
                    }
                    let local_func = module.declare_func_in_func(fid, builder.func);
                    let call = builder.ins().call(local_func, &arg_vals);
                    return Ok(builder.inst_results(call)[0]);
                }
            }
            Err(cerr(format!("Cannot compile app: {}", expr)))
        }

        CoreExpr::Case(scrut, alts) => {
            let sv = compile_expr(builder, vars, vc, funcs, module, scrut)?;
            compile_case(builder, vars, vc, funcs, module, sv, alts, 0)
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
                let elem_val = compile_expr(builder, vars, vc, funcs, module, elem)?;
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

        CoreExpr::Lam(_, _) => {
            Err(cerr("First-class closures not yet supported in JIT (use interpreter)"))
        }

        _ => Err(cerr(format!("Unsupported in codegen: {}", expr))),
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
    scrut: cranelift_codegen::ir::Value,
    alts: &[Alt],
    idx: usize,
) -> CResult<cranelift_codegen::ir::Value> {
    if idx >= alts.len() {
        return Err(cerr("Non-exhaustive patterns"));
    }
    let alt = &alts[idx];
    match &alt.pat {
        CorePat::Wildcard => compile_expr(builder, vars, vc, funcs, module, &alt.body),
        CorePat::Var(name) => {
            let var = Variable::from_u32(*vc); *vc += 1;
            builder.declare_var(var, types::I64);
            builder.def_var(var, scrut);
            vars.insert(name.clone(), var);
            compile_expr(builder, vars, vc, funcs, module, &alt.body)
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
            let tv = compile_expr(builder, vars, vc, funcs, module, &alt.body)?;
            builder.def_var(rv, tv);
            builder.ins().jump(merge_b, &[]);

            builder.switch_to_block(else_b);
            builder.seal_block(else_b);
            let ev = compile_case(builder, vars, vc, funcs, module, scrut, alts, idx+1)?;
            builder.def_var(rv, ev);
            builder.ins().jump(merge_b, &[]);

            builder.switch_to_block(merge_b);
            builder.seal_block(merge_b);
            Ok(builder.use_var(rv))
        }
        CorePat::Lit(Lit::Bool(false)) => {
            compile_expr(builder, vars, vc, funcs, module, &alt.body)
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
                let tv = compile_expr(builder, vars, vc, funcs, module, &alt.body)?;
                builder.def_var(rv, tv);
                builder.ins().jump(merge_b, &[]);

                builder.switch_to_block(else_b);
                builder.seal_block(else_b);
                let ev = compile_case(builder, vars, vc, funcs, module, scrut, alts, idx+1)?;
                builder.def_var(rv, ev);
                builder.ins().jump(merge_b, &[]);

                builder.switch_to_block(merge_b);
                builder.seal_block(merge_b);
                Ok(builder.use_var(rv))
            } else {
                compile_expr(builder, vars, vc, funcs, module, &alt.body)
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
                let tv = compile_expr(builder, vars, vc, funcs, module, &alt.body)?;
                builder.def_var(rv, tv);
                builder.ins().jump(merge_b, &[]);

                builder.switch_to_block(else_b);
                builder.seal_block(else_b);
                let ev = compile_case(builder, vars, vc, funcs, module, scrut, alts, idx+1)?;
                builder.def_var(rv, ev);
                builder.ins().jump(merge_b, &[]);

                builder.switch_to_block(merge_b);
                builder.seal_block(merge_b);
                Ok(builder.use_var(rv))
            } else {
                compile_expr(builder, vars, vc, funcs, module, &alt.body)
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

            // Cons branch: extract head and tail, bind to pattern vars
            builder.switch_to_block(then_b);
            builder.seal_block(then_b);

            // Call synoema_head(scrut) and synoema_tail(scrut)
            let head_fn = *funcs.get("synoema_head").ok_or_else(|| cerr("synoema_head"))?;
            let tail_fn = *funcs.get("synoema_tail").ok_or_else(|| cerr("synoema_tail"))?;
            let head_ref = module.declare_func_in_func(head_fn, builder.func);
            let tail_ref = module.declare_func_in_func(tail_fn, builder.func);

            let head_call = builder.ins().call(head_ref, &[scrut]);
            let head_val = builder.inst_results(head_call)[0];
            let tail_call = builder.ins().call(tail_ref, &[scrut]);
            let tail_val = builder.inst_results(tail_call)[0];

            // Bind head pattern
            if let CorePat::Var(hname) = &sub_pats[0] {
                let hvar = Variable::from_u32(*vc); *vc += 1;
                builder.declare_var(hvar, types::I64);
                builder.def_var(hvar, head_val);
                vars.insert(hname.clone(), hvar);
            }
            // Bind tail pattern
            if let CorePat::Var(tname) = &sub_pats[1] {
                let tvar = Variable::from_u32(*vc); *vc += 1;
                builder.declare_var(tvar, types::I64);
                builder.def_var(tvar, tail_val);
                vars.insert(tname.clone(), tvar);
            }

            let tv = compile_expr(builder, vars, vc, funcs, module, &alt.body)?;
            builder.def_var(rv, tv);
            builder.ins().jump(merge_b, &[]);

            // Else branch: try next alternative
            builder.switch_to_block(else_b);
            builder.seal_block(else_b);
            let ev = if idx + 1 < alts.len() {
                compile_case(builder, vars, vc, funcs, module, scrut, alts, idx+1)?
            } else {
                builder.ins().iconst(types::I64, 0) // fallback
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
