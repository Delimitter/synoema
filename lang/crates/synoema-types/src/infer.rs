// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

//! Algorithm W — Hindley-Milner type inference for Synoema.
//!
//! Implements the classic Damas-Milner algorithm (1982) extended for:
//! - Algebraic data types (constructors)
//! - Pattern matching
//! - List literals and comprehensions
//! - Pipe operator (desugared to application)
//! - Block expressions with local bindings (let-polymorphism)

use synoema_parser::*;
use synoema_lexer::Span;
use crate::types::*;
use crate::unify::unify;
use crate::error::{TypeError, TypeErrorKind};

type TResult<T> = Result<T, TypeError>;

/// Decompose a curried function type `T1 -> T2 -> ... -> R` into
/// argument types and the result type.
fn unroll_arrows(ty: &Type) -> (Vec<Type>, Type) {
    let mut args = Vec::new();
    let mut current = ty.clone();
    while let Type::Arrow(param, ret) = current {
        args.push(*param);
        current = *ret;
    }
    (args, current)
}

/// A collected type alias: name, params, body
type AliasMap = std::collections::HashMap<String, (Vec<String>, TypeExpr)>;

/// Main type inference engine
pub struct Infer {
    gen: TyVarGen,
    /// Type aliases collected from the program (expanded during type resolution)
    aliases: AliasMap,
}

impl Infer {
    pub fn new() -> Self {
        Infer { gen: TyVarGen::new(), aliases: AliasMap::new() }
    }

    /// Infer the type of a complete program, returning the final environment
    pub fn infer_program(&mut self, program: &Program) -> TResult<TypeEnv> {
        let mut env = self.builtin_env();

        // Pass 0: collect type aliases
        self.aliases.clear();
        for decl in &program.decls {
            if let Decl::TypeAlias { name, params, body, .. } = decl {
                // Check for recursive aliases (simple: alias body mentions its own name)
                if type_expr_mentions(body, name) {
                    return Err(TypeError::bare(TypeErrorKind::RecursiveAlias { name: name.clone() }));
                }
                self.aliases.insert(name.clone(), (params.clone(), body.clone()));
            }
        }

        // First pass: collect ADT constructors
        for decl in &program.decls {
            if let Decl::TypeDef { name, params, variants, .. } = decl {
                self.register_adt(&mut env, name, params, variants)?;
            }
        }

        // Second pass: collect explicit type signatures
        let mut sigs: std::collections::HashMap<String, TypeExpr> = std::collections::HashMap::new();
        for decl in &program.decls {
            if let Decl::TypeSig(sig) = decl {
                sigs.insert(sig.name.clone(), sig.ty.clone());
            }
        }

        // Third pass: infer impl method types (before Func, so derived methods are available)
        for decl in &program.decls {
            if let Decl::ImplDecl { methods, .. } = decl {
                for (method_name, equations) in methods {
                    if env.lookup(method_name).is_some() {
                        continue; // already known (builtin or previously defined)
                    }
                    let self_tv = self.gen.fresh_var();
                    env.insert(method_name.clone(), Scheme::mono(self_tv.clone()));

                    let (subst, ty) = self.infer_func(&env, equations)?;
                    let su = unify(&self_tv.apply(&subst), &ty, &mut self.gen)?;
                    let final_ty = ty.apply(&su);

                    // Skip env.apply for the entire env — earlier generalized schemes
                    // have no free vars affected by this substitution. Only the current
                    // method's mono entry needs updating, and we're about to remove it.
                    env.remove(method_name);
                    let scheme = env.generalize(&final_ty);
                    env.insert(method_name.clone(), scheme);
                }
            }
        }

        // Fourth pass: infer function types
        for decl in &program.decls {
            if let Decl::Func { name, equations, .. } = decl {
                // Pre-register with a fresh type variable to enable recursion
                let self_tv = self.gen.fresh_var();
                env.insert(name.clone(), Scheme::mono(self_tv.clone()));

                let (subst, ty) = self.infer_func(&env, equations)?;
                // Unify the pre-registered variable with the inferred type
                let su = unify(&self_tv.apply(&subst), &ty, &mut self.gen)?;
                let final_ty = ty.apply(&su);

                // Skip env.apply for the entire env — earlier generalized schemes
                // have no free vars affected by this substitution. Only the current
                // function's mono entry needs updating, and we're about to remove it.

                // Remove function's own mono entry before generalization
                // so its type variables are free to be quantified (let-polymorphism)
                env.remove(name);
                let scheme = env.generalize(&final_ty);
                env.insert(name.clone(), scheme);
            }
        }

        // Fourth pass: linearity check
        // For each function whose type signature contains -o arrows, verify that
        // linear parameters are used exactly once in every equation body.
        for decl in &program.decls {
            if let Decl::Func { name, equations, .. } = decl {
                if let Some(ty_sig) = sigs.get(name) {
                    check_linear_func(equations, ty_sig)?;
                }
            }
        }

        Ok(env)
    }

    /// Type-check with error recovery: continues past errors, collecting them.
    pub fn infer_program_recovering(&mut self, program: &Program) -> (Result<TypeEnv, TypeError>, Vec<TypeError>) {
        let mut env = self.builtin_env();
        let mut errors = Vec::new();

        // Pass 0: collect type aliases
        self.aliases.clear();
        for decl in &program.decls {
            if let Decl::TypeAlias { name, params, body, .. } = decl {
                if type_expr_mentions(body, name) {
                    errors.push(TypeError::bare(TypeErrorKind::RecursiveAlias { name: name.clone() }));
                    continue;
                }
                self.aliases.insert(name.clone(), (params.clone(), body.clone()));
            }
        }

        // First pass: collect ADT constructors
        for decl in &program.decls {
            if let Decl::TypeDef { name, params, variants, .. } = decl {
                if let Err(e) = self.register_adt(&mut env, name, params, variants) {
                    errors.push(e);
                }
            }
        }

        // Second pass: collect explicit type signatures
        let mut sigs: std::collections::HashMap<String, TypeExpr> = std::collections::HashMap::new();
        for decl in &program.decls {
            if let Decl::TypeSig(sig) = decl {
                sigs.insert(sig.name.clone(), sig.ty.clone());
            }
        }

        // Third pass: infer impl method types (before Func, so derived methods are available)
        for decl in &program.decls {
            if let Decl::ImplDecl { methods, .. } = decl {
                for (method_name, equations) in methods {
                    if env.lookup(method_name).is_some() {
                        continue;
                    }
                    let self_tv = self.gen.fresh_var();
                    env.insert(method_name.clone(), Scheme::mono(self_tv.clone()));

                    match self.infer_func(&env, equations) {
                        Ok((subst, ty)) => {
                            match unify(&self_tv.apply(&subst), &ty, &mut self.gen) {
                                Ok(su) => {
                                    let final_subst = subst.compose(&su);
                                    let final_ty = ty.apply(&su);
                                    env = env.apply(&final_subst);
                                    env.remove(method_name);
                                    let scheme = env.generalize(&final_ty);
                                    env.insert(method_name.clone(), scheme);
                                }
                                Err(e) => errors.push(e),
                            }
                        }
                        Err(e) => errors.push(e),
                    }
                }
            }
        }

        // Fourth pass: infer function types (with recovery)
        for decl in &program.decls {
            if let Decl::Func { name, equations, .. } = decl {
                let self_tv = self.gen.fresh_var();
                env.insert(name.clone(), Scheme::mono(self_tv.clone()));

                match self.infer_func(&env, equations) {
                    Ok((subst, ty)) => {
                        match unify(&self_tv.apply(&subst), &ty, &mut self.gen) {
                            Ok(su) => {
                                let final_subst = subst.compose(&su);
                                let final_ty = ty.apply(&su);
                                env = env.apply(&final_subst);
                                env.remove(name);
                                let scheme = env.generalize(&final_ty);
                                env.insert(name.clone(), scheme);
                            }
                            Err(e) => {
                                errors.push(e);
                            }
                        }
                    }
                    Err(e) => {
                        errors.push(e);
                    }
                }
            }
        }

        // Test declarations: infer body as Bool
        for decl in &program.decls {
            if let Decl::Test { body, .. } = decl {
                match self.infer_expr(&env, body) {
                    Ok((s, ty)) => {
                        if let Err(e) = unify(&ty.apply(&s), &Type::Con("Bool".into()), &mut self.gen) {
                            errors.push(e);
                        }
                    }
                    Err(e) => errors.push(e),
                }
            }
        }

        // Fourth pass: linearity check
        for decl in &program.decls {
            if let Decl::Func { name, equations, .. } = decl {
                if let Some(ty_sig) = sigs.get(name) {
                    if let Err(e) = check_linear_func(equations, ty_sig) {
                        errors.push(e);
                    }
                }
            }
        }

        if errors.is_empty() {
            (Ok(env), errors)
        } else {
            (Ok(env), errors)
        }
    }

    /// Infer the type of a single expression
    pub fn infer_expr(&mut self, env: &TypeEnv, expr: &Expr) -> TResult<(Subst, Type)> {
        self.infer(env, expr)
    }

    // ── Builtin Environment ─────────────────────────────

    fn builtin_env(&mut self) -> TypeEnv {
        let mut env = TypeEnv::new();

        // Arithmetic: Int -> Int -> Int
        let int_binop = Scheme::mono(Type::arrow(Type::int(), Type::arrow(Type::int(), Type::int())));
        for name in &["add#", "sub#", "mul#", "div#", "mod#"] {
            env.insert(name.to_string(), int_binop.clone());
        }

        // Float arithmetic
        let float_binop = Scheme::mono(Type::arrow(Type::float(), Type::arrow(Type::float(), Type::float())));
        for name in &["fadd#", "fsub#", "fmul#", "fdiv#"] {
            env.insert(name.to_string(), float_binop.clone());
        }

        // Comparison: ∀a. a -> a -> Bool
        let a = self.gen.fresh();
        let cmp_scheme = Scheme {
            vars: vec![a],
            ty: Type::arrow(Type::Var(a), Type::arrow(Type::Var(a), Type::bool())),
        };
        for name in &["eq#", "neq#", "lt#", "gt#", "lte#", "gte#"] {
            env.insert(name.to_string(), cmp_scheme.clone());
        }

        // Logic: Bool -> Bool -> Bool
        let bool_binop = Scheme::mono(Type::arrow(Type::bool(), Type::arrow(Type::bool(), Type::bool())));
        env.insert("and#".into(), bool_binop.clone());
        env.insert("or#".into(), bool_binop);

        // Concat: ∀a. List a -> List a -> List a
        let b = self.gen.fresh();
        env.insert("concat#".into(), Scheme {
            vars: vec![b],
            ty: Type::arrow(
                Type::list(Type::Var(b)),
                Type::arrow(Type::list(Type::Var(b)), Type::list(Type::Var(b))),
            ),
        });

        // Cons: ∀a. a -> List a -> List a
        let c = self.gen.fresh();
        env.insert("cons#".into(), Scheme {
            vars: vec![c],
            ty: Type::arrow(
                Type::Var(c),
                Type::arrow(Type::list(Type::Var(c)), Type::list(Type::Var(c))),
            ),
        });

        // Negate: Int -> Int
        env.insert("neg#".into(), Scheme::mono(Type::arrow(Type::int(), Type::int())));

        // show: ∀a. a -> String
        let d = self.gen.fresh();
        env.insert("show".into(), Scheme {
            vars: vec![d],
            ty: Type::arrow(Type::Var(d), Type::string()),
        });

        // print: ∀a. a -> ()
        let pa = self.gen.fresh();
        env.insert("print".into(), Scheme {
            vars: vec![pa],
            ty: Type::arrow(Type::Var(pa), Type::unit()),
        });

        // readline: String (0-arity IO action — reads from stdin)
        env.insert("readline".into(), Scheme::mono(Type::string()));

        // length: ∀a. List a -> Int
        let e = self.gen.fresh();
        env.insert("length".into(), Scheme {
            vars: vec![e],
            ty: Type::arrow(Type::list(Type::Var(e)), Type::int()),
        });

        // head: ∀a. List a -> a
        let f = self.gen.fresh();
        env.insert("head".into(), Scheme {
            vars: vec![f],
            ty: Type::arrow(Type::list(Type::Var(f)), Type::Var(f)),
        });

        // tail: ∀a. List a -> List a
        let g = self.gen.fresh();
        env.insert("tail".into(), Scheme {
            vars: vec![g],
            ty: Type::arrow(Type::list(Type::Var(g)), Type::list(Type::Var(g))),
        });

        // map: ∀a b. (a -> b) -> List a -> List b
        let ma = self.gen.fresh();
        let mb = self.gen.fresh();
        env.insert("map".into(), Scheme {
            vars: vec![ma, mb],
            ty: Type::arrow(
                Type::arrow(Type::Var(ma), Type::Var(mb)),
                Type::arrow(Type::list(Type::Var(ma)), Type::list(Type::Var(mb))),
            ),
        });

        // filter: ∀a. (a -> Bool) -> List a -> List a
        let fa = self.gen.fresh();
        env.insert("filter".into(), Scheme {
            vars: vec![fa],
            ty: Type::arrow(
                Type::arrow(Type::Var(fa), Type::bool()),
                Type::arrow(Type::list(Type::Var(fa)), Type::list(Type::Var(fa))),
            ),
        });

        // foldl: ∀a b. (a -> b -> a) -> a -> List b -> a
        let fla = self.gen.fresh();
        let flb = self.gen.fresh();
        env.insert("foldl".into(), Scheme {
            vars: vec![fla, flb],
            ty: Type::arrow(
                Type::arrow(Type::Var(fla), Type::arrow(Type::Var(flb), Type::Var(fla))),
                Type::arrow(Type::Var(fla), Type::arrow(Type::list(Type::Var(flb)), Type::Var(fla))),
            ),
        });

        // zip: ∀a. List a -> List a -> List (List a)
        let za = self.gen.fresh();
        env.insert("zip".into(), Scheme {
            vars: vec![za],
            ty: Type::arrow(
                Type::list(Type::Var(za)),
                Type::arrow(Type::list(Type::Var(za)), Type::list(Type::list(Type::Var(za)))),
            ),
        });

        // index: ∀a. Int -> List a -> a
        let ia = self.gen.fresh();
        env.insert("index".into(), Scheme {
            vars: vec![ia],
            ty: Type::arrow(Type::int(), Type::arrow(Type::list(Type::Var(ia)), Type::Var(ia))),
        });

        // take: ∀a. Int -> List a -> List a
        let ta = self.gen.fresh();
        env.insert("take".into(), Scheme {
            vars: vec![ta],
            ty: Type::arrow(Type::int(), Type::arrow(Type::list(Type::Var(ta)), Type::list(Type::Var(ta)))),
        });

        // drop: ∀a. Int -> List a -> List a
        let da = self.gen.fresh();
        env.insert("drop".into(), Scheme {
            vars: vec![da],
            ty: Type::arrow(Type::int(), Type::arrow(Type::list(Type::Var(da)), Type::list(Type::Var(da)))),
        });

        // reverse: ∀a. List a -> List a
        let rva = self.gen.fresh();
        env.insert("reverse".into(), Scheme {
            vars: vec![rva],
            ty: Type::arrow(Type::list(Type::Var(rva)), Type::list(Type::Var(rva))),
        });

        // sum: List Int -> Int
        env.insert("sum".into(), Scheme::mono(
            Type::arrow(Type::list(Type::int()), Type::int()),
        ));

        // even/odd: Int -> Bool
        env.insert("even".into(), Scheme::mono(Type::arrow(Type::int(), Type::bool())));
        env.insert("odd".into(), Scheme::mono(Type::arrow(Type::int(), Type::bool())));

        // not: Bool -> Bool
        env.insert("not".into(), Scheme::mono(Type::arrow(Type::bool(), Type::bool())));

        // Float math builtins: Float -> Float
        for name in &["sqrt", "floor", "ceil", "round"] {
            env.insert(name.to_string(), Scheme::mono(
                Type::arrow(Type::float(), Type::float()),
            ));
        }

        // abs: Int -> Int (also works on Float at runtime via tag dispatch)
        env.insert("abs".into(), Scheme::mono(Type::arrow(Type::int(), Type::int())));

        // pow#: Int -> Int -> Int
        env.insert("pow#".into(), Scheme::mono(
            Type::arrow(Type::int(), Type::arrow(Type::int(), Type::int())),
        ));

        // fpow#: Float -> Float -> Float
        env.insert("fpow#".into(), Scheme::mono(
            Type::arrow(Type::float(), Type::arrow(Type::float(), Type::float())),
        ));

        // String builtins
        // str_slice: String -> Int -> Int -> String
        env.insert("str_slice".into(), Scheme::mono(
            Type::arrow(Type::string(), Type::arrow(Type::int(), Type::arrow(Type::int(), Type::string()))),
        ));
        // str_find: String -> String -> Int -> Int
        env.insert("str_find".into(), Scheme::mono(
            Type::arrow(Type::string(), Type::arrow(Type::string(), Type::arrow(Type::int(), Type::int()))),
        ));
        // str_starts_with: String -> String -> Bool
        env.insert("str_starts_with".into(), Scheme::mono(
            Type::arrow(Type::string(), Type::arrow(Type::string(), Type::bool())),
        ));
        // str_trim: String -> String
        env.insert("str_trim".into(), Scheme::mono(Type::arrow(Type::string(), Type::string())));
        // str_len: String -> Int
        env.insert("str_len".into(), Scheme::mono(Type::arrow(Type::string(), Type::int())));
        // str_join: String -> [String] -> String
        env.insert("str_join".into(), Scheme::mono(Type::arrow(Type::string(), Type::arrow(Type::list(Type::string()), Type::string()))));
        // json_escape: String -> String
        env.insert("json_escape".into(), Scheme::mono(Type::arrow(Type::string(), Type::string())));
        // json_parse: String -> Result JsonValue String
        let json_result = Type::App(
            Box::new(Type::App(
                Box::new(Type::Con("Result".into())),
                Box::new(Type::Con("JsonValue".into())),
            )),
            Box::new(Type::string()),
        );
        env.insert("json_parse".into(), Scheme::mono(Type::arrow(Type::string(), json_result)));
        // file_read: String -> String
        env.insert("file_read".into(), Scheme::mono(Type::arrow(Type::string(), Type::string())));

        // I/O builtins (fd-based networking)
        // tcp_listen: Int -> Int
        env.insert("tcp_listen".into(), Scheme::mono(Type::arrow(Type::int(), Type::int())));
        // tcp_accept: Int -> Int
        env.insert("tcp_accept".into(), Scheme::mono(Type::arrow(Type::int(), Type::int())));
        // fd_readline: Int -> String
        env.insert("fd_readline".into(), Scheme::mono(Type::arrow(Type::int(), Type::string())));
        // fd_write: Int -> String -> ()
        env.insert("fd_write".into(), Scheme::mono(
            Type::arrow(Type::int(), Type::arrow(Type::string(), Type::unit())),
        ));
        // fd_close: Int -> ()
        env.insert("fd_close".into(), Scheme::mono(Type::arrow(Type::int(), Type::unit())));
        // fd_popen: String -> Int
        env.insert("fd_popen".into(), Scheme::mono(Type::arrow(Type::string(), Type::int())));
        // fd_open: String -> Int (open file for reading)
        env.insert("fd_open".into(), Scheme::mono(Type::arrow(Type::string(), Type::int())));
        // fd_open_write: String -> Int (open file for writing)
        env.insert("fd_open_write".into(), Scheme::mono(Type::arrow(Type::string(), Type::int())));

        // Concurrency builtins (Phase BC)
        // chan: ∀a. Chan a  (0-arity — fresh channel on each evaluation)
        let ca = self.gen.fresh();
        env.insert("chan".into(), Scheme { vars: vec![ca], ty: Type::chan(Type::Var(ca)) });
        // send: ∀a. Chan a -> a -> Unit
        let sa = self.gen.fresh();
        env.insert("send".into(), Scheme {
            vars: vec![sa],
            ty: Type::arrow(Type::chan(Type::Var(sa)), Type::arrow(Type::Var(sa), Type::unit())),
        });
        // recv: ∀a. Chan a -> a
        let ra = self.gen.fresh();
        env.insert("recv".into(), Scheme {
            vars: vec![ra],
            ty: Type::arrow(Type::chan(Type::Var(ra)), Type::Var(ra)),
        });

        // Environment variables
        // env: String -> String
        env.insert("env".into(), Scheme::mono(Type::arrow(Type::string(), Type::string())));
        // env_or: String -> String -> String
        env.insert("env_or".into(), Scheme::mono(
            Type::arrow(Type::string(), Type::arrow(Type::string(), Type::string()))
        ));
        // args: [String]
        env.insert("args".into(), Scheme::mono(Type::list(Type::string())));

        // error: ∀a. String -> a (runtime panic with message)
        let err_a = self.gen.fresh();
        env.insert("error".into(), Scheme {
            vars: vec![err_a],
            ty: Type::arrow(Type::string(), Type::Var(err_a)),
        });

        env
    }

    // ── ADT Registration ────────────────────────────────

    fn register_adt(
        &mut self,
        env: &mut TypeEnv,
        name: &str,
        params: &[String],
        variants: &[Variant],
    ) -> TResult<()> {
        // Create type variables for parameters
        let param_vars: Vec<(String, TyVarId)> = params.iter()
            .map(|p| (p.clone(), self.gen.fresh()))
            .collect();

        // Build the result type: Name α₁ ... αₙ
        let mut result_ty: Type = Type::Con(name.into());
        let mut quant_vars = Vec::new();
        for (_, var) in &param_vars {
            quant_vars.push(*var);
            result_ty = Type::App(Box::new(result_ty), Box::new(Type::Var(*var)));
        }

        // Register each constructor
        for variant in variants {
            let mut con_ty = result_ty.clone();

            // Build constructor type: field₁ → field₂ → ... → ResultType
            // (right to left, curried)
            for field in variant.fields.iter().rev() {
                let field_ty = self.resolve_type_expr(field, &param_vars)?;
                con_ty = Type::arrow(field_ty, con_ty);
            }

            let scheme = Scheme { vars: quant_vars.clone(), ty: con_ty };
            env.insert(variant.name.clone(), scheme);
        }

        Ok(())
    }

    /// Resolve a TypeExpr from the parser into a Type
    fn resolve_type_expr(
        &self,
        texpr: &TypeExpr,
        params: &[(String, TyVarId)],
    ) -> TResult<Type> {
        match &texpr.kind {
            TypeExprKind::Var(name) => {
                if let Some((_, id)) = params.iter().find(|(n, _)| n == name) {
                    Ok(Type::Var(*id))
                } else {
                    Err(TypeError::bare(TypeErrorKind::UnboundType { name: name.clone() }))
                }
            }
            TypeExprKind::Con(name) => {
                // Check if this is a type alias
                if let Some((alias_params, alias_body)) = self.aliases.get(name).cloned() {
                    if alias_params.is_empty() {
                        // Non-parametric alias: just resolve the body
                        return self.resolve_type_expr(&alias_body, params);
                    }
                    // Parametric alias without arguments applied — treat as type constructor
                    // (will be handled in App case when arguments are provided)
                }
                Ok(Type::Con(name.clone()))
            }
            TypeExprKind::Arrow(a, b) => {
                let a_ty = self.resolve_type_expr(a, params)?;
                let b_ty = self.resolve_type_expr(b, params)?;
                Ok(Type::arrow(a_ty, b_ty))
            }
            TypeExprKind::LinearArrow(a, b) => {
                let a_ty = self.resolve_type_expr(a, params)?;
                let b_ty = self.resolve_type_expr(b, params)?;
                Ok(Type::linear_arrow(a_ty, b_ty))
            }
            TypeExprKind::App(f, a) => {
                // Check for parametric type alias: collect all applied args
                let (head, args) = collect_type_app(texpr);
                if let TypeExprKind::Con(ref head_name) = head.kind {
                    if let Some((alias_params, alias_body)) = self.aliases.get(head_name).cloned() {
                        if args.len() == alias_params.len() {
                            // Substitute alias params with the actual type arguments
                            let expanded = substitute_type_expr(&alias_body, &alias_params, &args);
                            return self.resolve_type_expr(&expanded, params);
                        }
                    }
                }
                let f_ty = self.resolve_type_expr(f, params)?;
                let a_ty = self.resolve_type_expr(a, params)?;
                Ok(Type::App(Box::new(f_ty), Box::new(a_ty)))
            }
            TypeExprKind::Paren(inner) => self.resolve_type_expr(inner, params),
        }
    }

    // ── Instantiation ───────────────────────────────────

    /// Instantiate a scheme with fresh type variables
    fn instantiate(&mut self, scheme: &Scheme) -> Type {
        let fresh_vars: Vec<Type> = scheme.vars.iter()
            .map(|_| self.gen.fresh_var())
            .collect();
        let subst = Subst(
            scheme.vars.iter().copied()
                .zip(fresh_vars)
                .collect()
        );
        scheme.ty.apply(&subst)
    }

    // ── Core Inference (Algorithm W) ────────────────────

    fn infer(&mut self, env: &TypeEnv, expr: &Expr) -> TResult<(Subst, Type)> {
        self.infer_inner(env, expr)
            .map_err(|e| e.or_span(expr.span))
    }

    fn infer_inner(&mut self, env: &TypeEnv, expr: &Expr) -> TResult<(Subst, Type)> {
        match &expr.kind {
            ExprKind::Lit(lit) => Ok((Subst::new(), self.lit_type(lit))),

            ExprKind::Var(name) => {
                match env.lookup(name) {
                    Some(scheme) => Ok((Subst::new(), self.instantiate(scheme))),
                    None => Err(TypeError::new(TypeErrorKind::Unbound { name: name.clone() }, Some(expr.span))),
                }
            }

            ExprKind::Con(name) => {
                match env.lookup(name) {
                    Some(scheme) => Ok((Subst::new(), self.instantiate(scheme))),
                    None => Err(TypeError::new(TypeErrorKind::Unbound { name: name.clone() }, Some(expr.span))),
                }
            }

            // APP: Γ ⊢ e₁ : τ₁    Γ ⊢ e₂ : τ₂    unify(τ₁, τ₂ → α)
            ExprKind::App(func, arg) => {
                let (s1, t1) = self.infer(env, func)?;
                let env2 = env.apply(&s1);
                let (s2, t2) = self.infer(&env2, arg)?;
                let ret = self.gen.fresh_var();
                let s3 = unify(
                    &t1.apply(&s2),
                    &Type::arrow(t2, ret.clone()),
                    &mut self.gen,
                )?;
                Ok((s1.compose(&s2).compose(&s3), ret.apply(&s3)))
            }

            // LAM: Γ, x : α ⊢ body : τ  =>  α → τ
            ExprKind::Lam(pats, body) => {
                self.infer_lambda(env, pats, body)
            }

            // BINOP: desugar to function application
            ExprKind::BinOp(op, lhs, rhs) => {
                self.infer_binop(env, *op, lhs, rhs)
            }

            // NEG: Int -> Int  OR  Float -> Float
            ExprKind::Neg(inner) => {
                let (s1, t1) = self.infer(env, inner)?;
                let is_float = matches!(&t1, Type::Con(s) if s == "Float");
                let num_ty = if is_float { Type::float() } else { Type::int() };
                let s2 = unify(&t1, &num_ty, &mut self.gen)?;
                Ok((s1.compose(&s2), num_ty))
            }

            // COND: guard : Bool, both branches same type
            ExprKind::Cond(guard, then_e, else_e) => {
                let (s1, t1) = self.infer(env, guard)?;
                let s2 = unify(&t1, &Type::bool(), &mut self.gen)?;
                let env2 = env.apply(&s1.compose(&s2));
                let (s3, t_then) = self.infer(&env2, then_e)?;
                let env3 = env2.apply(&s3);
                let (s4, t_else) = self.infer(&env3, else_e)?;
                let s5 = unify(&t_then.apply(&s4), &t_else, &mut self.gen)?;
                let final_s = s1.compose(&s2).compose(&s3).compose(&s4).compose(&s5);
                Ok((final_s, t_else.apply(&s5)))
            }

            // LIST: all elements same type
            ExprKind::List(elems) => {
                if elems.is_empty() {
                    let a = self.gen.fresh_var();
                    Ok((Subst::new(), Type::list(a)))
                } else {
                    let (mut s, t_first) = self.infer(env, &elems[0])?;
                    for elem in &elems[1..] {
                        let env_cur = env.apply(&s);
                        let (si, ti) = self.infer(&env_cur, elem)?;
                        let su = unify(&t_first.apply(&s).apply(&si), &ti, &mut self.gen)?;
                        s = s.compose(&si).compose(&su);
                    }
                    Ok((s.clone(), Type::list(t_first.apply(&s))))
                }
            }

            // RANGE: [a..b] both Int, result List Int
            ExprKind::Range(from, to) => {
                let (s1, t1) = self.infer(env, from)?;
                let s2 = unify(&t1, &Type::int(), &mut self.gen)?;
                let env2 = env.apply(&s1.compose(&s2));
                let (s3, t2) = self.infer(&env2, to)?;
                let s4 = unify(&t2, &Type::int(), &mut self.gen)?;
                let final_s = s1.compose(&s2).compose(&s3).compose(&s4);
                Ok((final_s, Type::list(Type::int())))
            }

            // LIST COMPREHENSION: [e | generators]
            ExprKind::ListComp(body_expr, generators) => {
                self.infer_list_comp(env, body_expr, generators)
            }

            // BLOCK: let-bindings with polymorphism
            ExprKind::Block(bindings, result) => {
                let mut s_acc = Subst::new();
                let mut env_cur = env.clone();

                for binding in bindings {
                    let (si, ti) = self.infer(&env_cur, &binding.value)?;
                    env_cur = env_cur.apply(&si);
                    let scheme = env_cur.generalize(&ti);
                    env_cur.insert(binding.name.clone(), scheme);
                    s_acc = s_acc.compose(&si);
                }

                let (sr, tr) = self.infer(&env_cur, result)?;
                Ok((s_acc.compose(&sr), tr))
            }

            // RECORD LITERAL: {name = e1, age = e2} : {name: T1, age: T2}
            // Record literals produce CLOSED record types (no row tail).
            ExprKind::Record(fields) => {
                let mut subst = Subst::new();
                let mut field_types: Vec<(String, Type)> = Vec::new();
                for (name, expr) in fields {
                    let (s, ty) = self.infer(&env.apply(&subst), expr)?;
                    subst = s.compose(&subst);
                    field_types.push((name.clone(), ty));
                }
                Ok((subst, Type::Record(field_types, None)))
            }

            // FIELD ACCESS: r.field — row-polymorphic field access.
            //
            // Strategy: create a fresh type variable T for the field type and a fresh
            // row variable `r` for "any additional fields".  Unify the object's type
            // with the open record `{field: T | r}`.  This lets `get_x rec = rec.x`
            // accept ANY record that has at least an `x` field.
            ExprKind::Field(obj, field) => {
                let (s_obj, obj_ty) = self.infer(env, obj)?;
                let obj_ty = obj_ty.apply(&s_obj);

                // Fresh type variable for the field value
                let field_tv = self.gen.fresh_var();
                // Fresh row variable for "the rest of the record"
                let row_var = self.gen.fresh();

                // Unify the object type with {field: T | r}
                let open_record = Type::Record(
                    vec![(field.clone(), field_tv.clone())],
                    Some(row_var),
                );
                let s_unify = unify(&obj_ty, &open_record, &mut self.gen)?;

                let final_subst = s_obj.compose(&s_unify);
                let result_ty = field_tv.apply(&s_unify);
                Ok((final_subst, result_ty))
            }

            // RECORD UPDATE: {...base, f1 = v1, ...}
            // Strategy: infer base type, then unify with open record from updates.
            // Result type = base type (same shape).
            ExprKind::RecordUpdate { base, updates } => {
                let (s_base, base_ty) = self.infer(env, base)?;
                let base_ty = base_ty.apply(&s_base);

                // Infer each update value and collect update field types
                let mut subst = s_base;
                let mut update_types: Vec<(String, Type)> = Vec::new();
                for (name, expr) in updates {
                    let (s, ty) = self.infer(&env.apply(&subst), expr)?;
                    subst = s.compose(&subst);
                    update_types.push((name.clone(), ty));
                }

                // Create open record from update fields — allows "other fields" via row var
                let row_var = self.gen.fresh();
                let open_record = Type::Record(update_types, Some(row_var));

                // Unify: base must contain all updated fields with compatible types
                let s_unify = unify(&base_ty.apply(&subst), &open_record, &mut self.gen)?;
                let final_subst = subst.compose(&s_unify);

                // Result type = base type (record shape preserved)
                let result_ty = base_ty.apply(&final_subst);
                Ok((final_subst, result_ty))
            }

            // PAREN: transparent
            ExprKind::Paren(inner) => self.infer(env, inner),

            // CONCURRENCY (Phase BC — not yet type-checked)
            ExprKind::Scope(body) => self.infer(env, body),
            ExprKind::Spawn(body) => {
                self.infer(env, body)?;
                Ok((Subst::new(), Type::Con("Unit".into())))
            }

            ExprKind::Prop(vars, body) => {
                let mut env = env.clone();
                for var in vars {
                    let tv = self.gen.fresh_var();
                    env.insert(var.clone(), Scheme::mono(tv));
                }
                let (s, body_ty) = self.infer(&env, body)?;
                let s2 = unify(&body_ty.apply(&s), &Type::Con("Bool".into()), &mut self.gen)?;
                Ok((s.compose(&s2), Type::Con("Bool".into())))
            }

            ExprKind::When(cond, body) => {
                let (s1, cond_ty) = self.infer(env, cond)?;
                let s1b = unify(&cond_ty.apply(&s1), &Type::Con("Bool".into()), &mut self.gen)?;
                let s1c = s1.compose(&s1b);
                let env2 = env.apply(&s1c);
                let (s2, body_ty) = self.infer(&env2, body)?;
                let s2b = unify(&body_ty.apply(&s2), &Type::Con("Bool".into()), &mut self.gen)?;
                let s_final = s1c.compose(&s2).compose(&s2b);
                Ok((s_final, Type::Con("Bool".into())))
            }
        }
    }

    // ── Lambda Inference ────────────────────────────────

    fn infer_lambda(
        &mut self,
        env: &TypeEnv,
        pats: &[Pat],
        body: &Expr,
    ) -> TResult<(Subst, Type)> {
        let mut env_ext = env.clone();
        let mut param_types = Vec::new();

        for pat in pats {
            let (pat_ty, bindings) = self.infer_pattern(env, pat)?;
            param_types.push(pat_ty);
            for (name, ty) in bindings {
                env_ext.insert(name, Scheme::mono(ty));
            }
        }

        let (s_body, t_body) = self.infer(&env_ext, body)?;

        // Build curried function type: τ₁ → τ₂ → ... → τₙ → τ_body
        let mut result_ty = t_body;
        for pt in param_types.into_iter().rev() {
            result_ty = Type::arrow(pt.apply(&s_body), result_ty);
        }

        Ok((s_body, result_ty))
    }

    // ── Function Inference ──────────────────────────────

    fn infer_func(
        &mut self,
        env: &TypeEnv,
        equations: &[Equation],
    ) -> TResult<(Subst, Type)> {
        if equations.is_empty() {
            return Err(TypeError::other("Empty function definition"));
        }

        // Infer type of first equation
        let first = &equations[0];
        let (s, ty) = self.infer_equation(env, first)?;
        let mut acc_subst = s;
        let mut func_ty = ty;

        // Unify with remaining equations
        for eq in &equations[1..] {
            let (si, ti) = self.infer_equation(&env.apply(&acc_subst), eq)?;
            let su = unify(&func_ty.apply(&si), &ti, &mut self.gen)?;
            acc_subst = acc_subst.compose(&si).compose(&su);
            func_ty = func_ty.apply(&si).apply(&su);
        }

        Ok((acc_subst, func_ty))
    }

    fn infer_equation(
        &mut self,
        env: &TypeEnv,
        eq: &Equation,
    ) -> TResult<(Subst, Type)> {
        let mut env_ext = env.clone();
        let mut param_types = Vec::new();

        for pat in &eq.pats {
            let (pat_ty, bindings) = self.infer_pattern(env, pat)?;
            param_types.push(pat_ty);
            for (name, ty) in bindings {
                env_ext.insert(name, Scheme::mono(ty));
            }
        }

        let (s_body, t_body) = self.infer(&env_ext, &eq.body)?;

        // Build curried function type
        let mut result_ty = t_body;
        for pt in param_types.into_iter().rev() {
            result_ty = Type::arrow(pt.apply(&s_body), result_ty);
        }

        Ok((s_body, result_ty))
    }

    // ── Pattern Inference ───────────────────────────────

    /// Infer the type of a pattern and return variable bindings
    fn infer_pattern(&mut self, env: &TypeEnv, pat: &Pat) -> TResult<(Type, Vec<(String, Type)>)> {
        match pat {
            Pat::Wildcard => {
                let tv = self.gen.fresh_var();
                Ok((tv, Vec::new()))
            }
            Pat::Var(name) => {
                let tv = self.gen.fresh_var();
                Ok((tv.clone(), vec![(name.clone(), tv)]))
            }
            Pat::Lit(lit) => {
                Ok((self.lit_type(lit), Vec::new()))
            }
            Pat::Con(name, sub_pats) => {
                let mut bindings = Vec::new();
                let mut arg_types = Vec::new();
                for sp in sub_pats {
                    let (ty, bs) = self.infer_pattern(env, sp)?;
                    arg_types.push(ty);
                    bindings.extend(bs);
                }

                // Look up constructor type from env
                if let Some(scheme) = env.lookup(name) {
                    let con_ty = self.instantiate(scheme);
                    // Unroll: T1 -> T2 -> ... -> R
                    let (expected_args, result_ty) = unroll_arrows(&con_ty);

                    if expected_args.len() != arg_types.len() {
                        return Err(TypeError::bare(TypeErrorKind::ArityMismatch {
                            name: name.clone(),
                            expected: expected_args.len(),
                            found: arg_types.len(),
                        }));
                    }

                    // Unify each sub-pattern type with the constructor's parameter type
                    for (expected, actual) in expected_args.iter().zip(arg_types.iter()) {
                        let s = unify(expected, actual, &mut self.gen)?;
                        // Apply substitution (simplified — in a full implementation
                        // we'd thread subst through)
                        let _ = s;
                    }

                    Ok((result_ty, bindings))
                } else {
                    // Constructor not in env — fallback to fresh type variable
                    let con_tv = self.gen.fresh_var();
                    Ok((con_tv, bindings))
                }
            }
            Pat::Cons(head, tail) => {
                let (h_ty, h_binds) = self.infer_pattern(env, head)?;
                let (_t_ty, t_binds) = self.infer_pattern(env, tail)?;
                let list_ty = Type::list(h_ty.clone());
                let mut bindings = h_binds;
                bindings.extend(t_binds);
                Ok((list_ty, bindings))
            }
            Pat::Paren(inner) => self.infer_pattern(env, inner),
            Pat::Record(fields) => {
                let mut all_bindings: Vec<(String, Type)> = Vec::new();
                let mut field_types: Vec<(String, Type)> = Vec::new();
                for (name, sub_pat) in fields {
                    let (ty, bindings) = self.infer_pattern(env, sub_pat)?;
                    field_types.push((name.clone(), ty));
                    all_bindings.extend(bindings);
                }
                let row_var = self.gen.fresh();
                Ok((Type::Record(field_types, Some(row_var)), all_bindings))
            }
        }
    }

    // ── Binary Operator Inference ───────────────────────

    fn infer_binop(
        &mut self,
        env: &TypeEnv,
        op: BinOp,
        lhs: &Expr,
        rhs: &Expr,
    ) -> TResult<(Subst, Type)> {
        let (s1, t1) = self.infer(env, lhs)?;
        let env2 = env.apply(&s1);
        let (s2, t2) = self.infer(&env2, rhs)?;

        let (s3, result_ty) = match op {
            // Arithmetic: Int -> Int -> Int  OR  Float -> Float -> Float
            // If either operand is already known to be Float, require both Float.
            // Mod (%) is always Int-only.
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Pow => {
                let resolved_t1 = t1.apply(&s2);
                let resolved_t2 = t2.clone();
                let is_float = matches!(&resolved_t1, Type::Con(s) if s == "Float")
                            || matches!(&resolved_t2, Type::Con(s) if s == "Float");
                let num_ty = if is_float { Type::float() } else { Type::int() };
                let sa = unify(&resolved_t1, &num_ty, &mut self.gen)?;
                let sb = unify(&resolved_t2.apply(&sa), &num_ty, &mut self.gen)?;
                (sa.compose(&sb), num_ty)
            }
            BinOp::Mod => {
                let sa = unify(&t1.apply(&s2), &Type::int(), &mut self.gen)?;
                let sb = unify(&t2.apply(&sa), &Type::int(), &mut self.gen)?;
                (sa.compose(&sb), Type::int())
            }
            // Comparison: a -> a -> Bool
            BinOp::Eq | BinOp::Neq | BinOp::Lt | BinOp::Gt | BinOp::Lte | BinOp::Gte => {
                let su = unify(&t1.apply(&s2), &t2, &mut self.gen)?;
                (su, Type::bool())
            }
            // Logic: Bool -> Bool -> Bool
            BinOp::And | BinOp::Or => {
                let sa = unify(&t1.apply(&s2), &Type::bool(), &mut self.gen)?;
                let sb = unify(&t2.apply(&sa), &Type::bool(), &mut self.gen)?;
                (sa.compose(&sb), Type::bool())
            }
            // Concat: List a -> List a -> List a  OR  String -> String -> String
            BinOp::Concat => {
                let resolved_t1 = t1.apply(&s2);
                let resolved_t2 = t2.clone();
                let is_string = matches!(&resolved_t1, Type::Con(s) if s == "String")
                             || matches!(&resolved_t2, Type::Con(s) if s == "String");
                if is_string {
                    let sa = unify(&resolved_t1, &Type::string(), &mut self.gen)?;
                    let sb = unify(&resolved_t2.apply(&sa), &Type::string(), &mut self.gen)?;
                    (sa.compose(&sb), Type::string())
                } else {
                    let elem = self.gen.fresh_var();
                    let list_ty = Type::list(elem);
                    let sa = unify(&resolved_t1, &list_ty, &mut self.gen)?;
                    let sb = unify(&resolved_t2.apply(&sa), &list_ty.apply(&sa), &mut self.gen)?;
                    let final_ty = list_ty.apply(&sa).apply(&sb);
                    (sa.compose(&sb), final_ty)
                }
            }
            // Cons: a -> List a -> List a
            BinOp::Cons => {
                let list_ty = Type::list(t1.apply(&s2).clone());
                let su = unify(&t2, &list_ty, &mut self.gen)?;
                (su.clone(), list_ty.apply(&su))
            }
            // Pipe: a |> (a -> b) = b  (desugars to application)
            BinOp::Pipe => {
                let ret = self.gen.fresh_var();
                let su = unify(&t2, &Type::arrow(t1.apply(&s2).clone(), ret.clone()), &mut self.gen)?;
                (su.clone(), ret.apply(&su))
            }
            // Compose: (a -> b) >> (b -> c) = (a -> c)
            BinOp::Compose => {
                let a = self.gen.fresh_var();
                let b = self.gen.fresh_var();
                let c = self.gen.fresh_var();
                let sa = unify(&t1.apply(&s2), &Type::arrow(a.clone(), b.clone()), &mut self.gen)?;
                let sb = unify(
                    &t2.apply(&sa),
                    &Type::arrow(b.apply(&sa), c.clone()),
                    &mut self.gen,
                )?;
                let sc = sa.compose(&sb);
                (sc.clone(), Type::arrow(a.apply(&sc), c.apply(&sc)))
            }
            // Sequence: a ; b — evaluate a for effect, return b's type
            BinOp::Seq => {
                (Subst::new(), t2)
            }
        };

        Ok((s1.compose(&s2).compose(&s3), result_ty))
    }

    // ── List Comprehension ──────────────────────────────

    fn infer_list_comp(
        &mut self,
        env: &TypeEnv,
        body: &Expr,
        generators: &[Generator],
    ) -> TResult<(Subst, Type)> {
        let mut env_cur = env.clone();
        let mut s_acc = Subst::new();

        for gen in generators {
            match gen {
                Generator::Bind(name, source) => {
                    let (si, ti) = self.infer(&env_cur, source)?;
                    // source must be List a, and name gets type a
                    let elem = self.gen.fresh_var();
                    let su = unify(&ti, &Type::list(elem.clone()), &mut self.gen)?;
                    let s_combined = si.compose(&su);
                    env_cur = env_cur.apply(&s_combined);
                    env_cur.insert(name.clone(), Scheme::mono(elem.apply(&s_combined)));
                    s_acc = s_acc.compose(&s_combined);
                }
                Generator::Guard(guard_expr) => {
                    let (si, ti) = self.infer(&env_cur, guard_expr)?;
                    let su = unify(&ti, &Type::bool(), &mut self.gen)?;
                    let s_combined = si.compose(&su);
                    env_cur = env_cur.apply(&s_combined);
                    s_acc = s_acc.compose(&s_combined);
                }
            }
        }

        let (sb, tb) = self.infer(&env_cur, body)?;
        let final_s = s_acc.compose(&sb);
        Ok((final_s, Type::list(tb)))
    }

    // ── Helpers ──────────────────────────────────────────

    fn lit_type(&self, lit: &Lit) -> Type {
        match lit {
            Lit::Int(_) => Type::int(),
            Lit::Float(_) => Type::float(),
            Lit::Str(_) => Type::string(),
            Lit::Char(_) => Type::char(),
            Lit::Bool(_) => Type::bool(),
            Lit::Unit => Type::unit(),
        }
    }
}

// ── Linearity Checker ────────────────────────────────────
//
// A separate post-typecheck pass that enforces linear usage discipline.
// Runs only for functions with explicit `-o` type signatures.
// Linearity is opt-in: programs without `-o` annotations are unaffected.

use std::collections::{HashMap, HashSet};
type UsageMap = HashMap<String, usize>;

/// Walk a TypeExpr and return a list of booleans: true = that arrow position is linear.
/// E.g. `Int -o String -> Bool` → [true, false]
fn linear_positions_from_type_expr(texpr: &TypeExpr) -> Vec<bool> {
    let mut result = Vec::new();
    let mut cur = texpr;
    loop {
        match &cur.kind {
            TypeExprKind::Arrow(_, b) => { result.push(false); cur = b; }
            TypeExprKind::LinearArrow(_, b) => { result.push(true); cur = b; }
            TypeExprKind::Paren(inner) => { cur = inner; }
            _ => break,
        }
    }
    result
}

/// Collect all variable names bound by a pattern.
fn pat_bound_names(pat: &Pat) -> Vec<String> {
    match pat {
        Pat::Var(name) => vec![name.clone()],
        Pat::Cons(h, t) => {
            let mut v = pat_bound_names(h);
            v.extend(pat_bound_names(t));
            v
        }
        Pat::Con(_, sub_pats) => sub_pats.iter().flat_map(pat_bound_names).collect(),
        Pat::Paren(p) => pat_bound_names(p),
        Pat::Record(fields) => fields.iter().flat_map(|(_, p)| pat_bound_names(p)).collect(),
        Pat::Wildcard | Pat::Lit(_) => vec![],
    }
}

/// Merge two usage maps for sequential composition.
/// Returns an error if any linear variable is used more than once in total.
fn seq_usages(m1: UsageMap, m2: UsageMap, span: Span) -> TResult<UsageMap> {
    let mut result = m1;
    for (name, count) in m2 {
        let entry = result.entry(name.clone()).or_insert(0);
        *entry += count;
        if *entry > 1 {
            return Err(TypeError::new(
                TypeErrorKind::LinearDuplicate { name },
                Some(span),
            ));
        }
    }
    Ok(result)
}

/// Verify that two conditional branches use linear variables the same number of times.
/// This ensures that both execution paths consume the same linear resources.
fn check_branches_agree(m1: &UsageMap, m2: &UsageMap, span: Span) -> TResult<()> {
    let all_names: HashSet<&String> = m1.keys().chain(m2.keys()).collect();
    for name in all_names {
        let c1 = m1.get(name).copied().unwrap_or(0);
        let c2 = m2.get(name).copied().unwrap_or(0);
        if c1 != c2 {
            return Err(TypeError::new(
                TypeErrorKind::LinearUnused { name: name.clone() },
                Some(span),
            ));
        }
    }
    Ok(())
}

/// Walk an expression, tracking how many times each linear variable is used.
/// Returns the usage map, or an error if a linear variable is used more than once.
fn check_linear_in_expr(expr: &Expr, linear_vars: &HashSet<String>) -> TResult<UsageMap> {
    if linear_vars.is_empty() {
        return Ok(HashMap::new());
    }
    match &expr.kind {
        ExprKind::Lit(_) | ExprKind::Con(_) => Ok(HashMap::new()),

        ExprKind::Var(name) => {
            let mut m = HashMap::new();
            if linear_vars.contains(name) {
                m.insert(name.clone(), 1);
            }
            Ok(m)
        }

        ExprKind::App(f, x) => {
            let mf = check_linear_in_expr(f, linear_vars)?;
            let mx = check_linear_in_expr(x, linear_vars)?;
            seq_usages(mf, mx, x.span)
        }

        ExprKind::Lam(pats, body) => {
            // Lambda parameters shadow enclosing linear vars
            let bound: HashSet<String> = pats.iter()
                .flat_map(pat_bound_names)
                .collect();
            let inner: HashSet<String> = linear_vars.difference(&bound).cloned().collect();
            check_linear_in_expr(body, &inner)
        }

        ExprKind::Cond(cond, then, else_) => {
            let mc = check_linear_in_expr(cond, linear_vars)?;
            let mt = check_linear_in_expr(then, linear_vars)?;
            let me = check_linear_in_expr(else_, linear_vars)?;
            // Both branches must use linear vars the same number of times
            check_branches_agree(&mt, &me, else_.span)?;
            seq_usages(mc, mt, then.span)
        }

        ExprKind::BinOp(_, l, r) => {
            let ml = check_linear_in_expr(l, linear_vars)?;
            let mr = check_linear_in_expr(r, linear_vars)?;
            seq_usages(ml, mr, r.span)
        }

        ExprKind::Neg(e) | ExprKind::Paren(e) => check_linear_in_expr(e, linear_vars),

        ExprKind::Field(e, _) => check_linear_in_expr(e, linear_vars),

        ExprKind::Record(fields) => {
            let mut acc: UsageMap = HashMap::new();
            for (_, e) in fields {
                let m = check_linear_in_expr(e, linear_vars)?;
                acc = seq_usages(acc, m, e.span)?;
            }
            Ok(acc)
        }

        ExprKind::RecordUpdate { base, updates } => {
            let mut acc = check_linear_in_expr(base, linear_vars)?;
            for (_, e) in updates {
                let m = check_linear_in_expr(e, linear_vars)?;
                acc = seq_usages(acc, m, e.span)?;
            }
            Ok(acc)
        }

        ExprKind::List(elems) => {
            let mut acc: UsageMap = HashMap::new();
            for e in elems {
                let m = check_linear_in_expr(e, linear_vars)?;
                acc = seq_usages(acc, m, e.span)?;
            }
            Ok(acc)
        }

        ExprKind::Block(bindings, body) => {
            // Bindings may shadow linear vars; walk each binding value
            let mut acc: UsageMap = HashMap::new();
            let mut shadowed = linear_vars.clone();
            for binding in bindings {
                let m = check_linear_in_expr(&binding.value, &shadowed)?;
                acc = seq_usages(acc, m, binding.value.span)?;
                // After this binding is defined, its name shadows enclosing scope
                shadowed.remove(&binding.name);
            }
            let mb = check_linear_in_expr(body, &shadowed)?;
            seq_usages(acc, mb, body.span)
        }

        ExprKind::ListComp(e, generators) => {
            let mut acc = check_linear_in_expr(e, linear_vars)?;
            for gen in generators {
                let ge = match gen {
                    Generator::Guard(e) => e,
                    Generator::Bind(_, e) => e,
                };
                let mg = check_linear_in_expr(ge, linear_vars)?;
                acc = seq_usages(acc, mg, ge.span)?;
            }
            Ok(acc)
        }

        ExprKind::Range(lo, hi) => {
            let ml = check_linear_in_expr(lo, linear_vars)?;
            let mh = check_linear_in_expr(hi, linear_vars)?;
            seq_usages(ml, mh, hi.span)
        }

        // CONCURRENCY (Phase BC — linearity not yet analysed)
        ExprKind::Scope(body) => check_linear_in_expr(body, linear_vars),
        ExprKind::Spawn(body) => check_linear_in_expr(body, linear_vars),
        ExprKind::Prop(_, body) => check_linear_in_expr(body, linear_vars),
        ExprKind::When(lhs, rhs) => {
            let ml = check_linear_in_expr(lhs, linear_vars)?;
            let mr = check_linear_in_expr(rhs, linear_vars)?;
            seq_usages(ml, mr, expr.span)
        }
    }
}

/// Run linearity check for one function (all equations) given its type signature.
fn check_linear_func(equations: &[Equation], ty_sig: &TypeExpr) -> TResult<()> {
    let positions = linear_positions_from_type_expr(ty_sig);
    // Skip if there are no linear positions
    if positions.iter().all(|&lin| !lin) {
        return Ok(());
    }

    for eq in equations {
        // Map each positional pattern to a linear variable name (if applicable)
        let linear_vars: HashSet<String> = positions.iter()
            .zip(eq.pats.iter())
            .filter(|(&lin, _)| lin)
            .flat_map(|(_, pat)| pat_bound_names(pat))
            .collect();

        if linear_vars.is_empty() {
            // Wildcard / literal patterns in linear positions — nothing to track
            continue;
        }

        // Check that each linear variable is used exactly once in the body
        let usages = check_linear_in_expr(&eq.body, &linear_vars)?;

        for var_name in &linear_vars {
            match usages.get(var_name).copied().unwrap_or(0) {
                0 => return Err(TypeError::new(
                    TypeErrorKind::LinearUnused { name: var_name.clone() },
                    Some(eq.body.span),
                )),
                // count > 1 already caught by seq_usages → LinearDuplicate
                _ => {}
            }
        }
    }

    Ok(())
}

// ── Type Alias Helpers ──────────────────────────────────

/// Check if a TypeExpr mentions a given name (for recursive alias detection)
fn type_expr_mentions(texpr: &TypeExpr, name: &str) -> bool {
    match &texpr.kind {
        TypeExprKind::Con(n) => n == name,
        TypeExprKind::Var(_) => false,
        TypeExprKind::Arrow(a, b) | TypeExprKind::LinearArrow(a, b) | TypeExprKind::App(a, b) => {
            type_expr_mentions(a, name) || type_expr_mentions(b, name)
        }
        TypeExprKind::Paren(inner) => type_expr_mentions(inner, name),
    }
}

/// Collect a chain of type applications: `F a b` → (F, [a, b])
fn collect_type_app(texpr: &TypeExpr) -> (&TypeExpr, Vec<&TypeExpr>) {
    let mut args = Vec::new();
    let mut current = texpr;
    while let TypeExprKind::App(f, a) = &current.kind {
        args.push(a.as_ref());
        current = f.as_ref();
    }
    args.reverse();
    (current, args)
}

/// Substitute type variable names in a TypeExpr
fn substitute_type_expr(texpr: &TypeExpr, param_names: &[String], args: &[&TypeExpr]) -> TypeExpr {
    let span = texpr.span;
    let kind = match &texpr.kind {
        TypeExprKind::Var(v) => {
            // If this var matches a parameter name, replace with corresponding arg
            if let Some(i) = param_names.iter().position(|p| p == v) {
                return args[i].clone();
            }
            TypeExprKind::Var(v.clone())
        }
        TypeExprKind::Con(c) => TypeExprKind::Con(c.clone()),
        TypeExprKind::Arrow(a, b) => {
            TypeExprKind::Arrow(
                Box::new(substitute_type_expr(a, param_names, args)),
                Box::new(substitute_type_expr(b, param_names, args)),
            )
        }
        TypeExprKind::LinearArrow(a, b) => {
            TypeExprKind::LinearArrow(
                Box::new(substitute_type_expr(a, param_names, args)),
                Box::new(substitute_type_expr(b, param_names, args)),
            )
        }
        TypeExprKind::App(f, a) => {
            TypeExprKind::App(
                Box::new(substitute_type_expr(f, param_names, args)),
                Box::new(substitute_type_expr(a, param_names, args)),
            )
        }
        TypeExprKind::Paren(inner) => {
            TypeExprKind::Paren(Box::new(substitute_type_expr(inner, param_names, args)))
        }
    };
    TypeExpr::new(kind, span)
}
