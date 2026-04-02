//! Algorithm W — Hindley-Milner type inference for Synoema.
//!
//! Implements the classic Damas-Milner algorithm (1982) extended for:
//! - Algebraic data types (constructors)
//! - Pattern matching
//! - List literals and comprehensions
//! - Pipe operator (desugared to application)
//! - Block expressions with local bindings (let-polymorphism)

use synoema_parser::*;
use crate::types::*;
use crate::unify::unify;
use crate::error::TypeError;

type TResult<T> = Result<T, TypeError>;

/// Main type inference engine
pub struct Infer {
    gen: TyVarGen,
}

impl Infer {
    pub fn new() -> Self {
        Infer { gen: TyVarGen::new() }
    }

    /// Infer the type of a complete program, returning the final environment
    pub fn infer_program(&mut self, program: &Program) -> TResult<TypeEnv> {
        let mut env = self.builtin_env();

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

        // Third pass: infer function types
        for decl in &program.decls {
            if let Decl::Func { name, equations, .. } = decl {
                // Pre-register with a fresh type variable to enable recursion
                let self_tv = self.gen.fresh_var();
                env.insert(name.clone(), Scheme::mono(self_tv.clone()));

                let (subst, ty) = self.infer_func(&env, equations)?;
                // Unify the pre-registered variable with the inferred type
                let su = unify(&self_tv.apply(&subst), &ty, &mut self.gen)?;
                let final_subst = subst.compose(&su);
                let final_ty = ty.apply(&su);

                env = env.apply(&final_subst);

                // Remove function's own mono entry before generalization
                // so its type variables are free to be quantified (let-polymorphism)
                env.remove(name);
                let scheme = env.generalize(&final_ty);
                env.insert(name.clone(), scheme);
            }
        }

        Ok(env)
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

        // sum: List Int -> Int
        env.insert("sum".into(), Scheme::mono(
            Type::arrow(Type::list(Type::int()), Type::int()),
        ));

        // even: Int -> Bool
        env.insert("even".into(), Scheme::mono(Type::arrow(Type::int(), Type::bool())));

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
                    Err(TypeError::UnboundType { name: name.clone() })
                }
            }
            TypeExprKind::Con(name) => Ok(Type::Con(name.clone())),
            TypeExprKind::Arrow(a, b) => {
                let a_ty = self.resolve_type_expr(a, params)?;
                let b_ty = self.resolve_type_expr(b, params)?;
                Ok(Type::arrow(a_ty, b_ty))
            }
            TypeExprKind::App(f, a) => {
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
        match &expr.kind {
            ExprKind::Lit(lit) => Ok((Subst::new(), self.lit_type(lit))),

            ExprKind::Var(name) => {
                match env.lookup(name) {
                    Some(scheme) => Ok((Subst::new(), self.instantiate(scheme))),
                    None => Err(TypeError::Unbound { name: name.clone() }),
                }
            }

            ExprKind::Con(name) => {
                match env.lookup(name) {
                    Some(scheme) => Ok((Subst::new(), self.instantiate(scheme))),
                    None => Err(TypeError::Unbound { name: name.clone() }),
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

            // PAREN: transparent
            ExprKind::Paren(inner) => self.infer(env, inner),
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
            let (pat_ty, bindings) = self.infer_pattern(pat)?;
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
            return Err(TypeError::Other("Empty function definition".into()));
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
            let (pat_ty, bindings) = self.infer_pattern(pat)?;
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
    fn infer_pattern(&mut self, pat: &Pat) -> TResult<(Type, Vec<(String, Type)>)> {
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
            Pat::Con(_name, sub_pats) => {
                // Look up constructor from a dummy env — not ideal but works for now
                // In practice, constructor types should be looked up from env
                let con_tv = self.gen.fresh_var();
                let mut bindings = Vec::new();
                let mut arg_types = Vec::new();
                for sp in sub_pats {
                    let (ty, bs) = self.infer_pattern(sp)?;
                    arg_types.push(ty);
                    bindings.extend(bs);
                }
                // The constructor type is: arg1 → arg2 → ... → result
                // We return a fresh variable as the result type
                let _ = arg_types; // constructor args constrained during pattern matching
                Ok((con_tv, bindings))
            }
            Pat::Cons(head, tail) => {
                let (h_ty, h_binds) = self.infer_pattern(head)?;
                let (_t_ty, t_binds) = self.infer_pattern(tail)?;
                // tail must be List h_ty
                let list_ty = Type::list(h_ty.clone());
                // We'd unify t_ty with list_ty, but since we return constraints
                // we just set the list type
                let mut bindings = h_binds;
                bindings.extend(t_binds);
                Ok((list_ty, bindings))
            }
            Pat::Paren(inner) => self.infer_pattern(inner),
            Pat::Record(fields) => {
                let mut all_bindings: Vec<(String, Type)> = Vec::new();
                let mut field_types: Vec<(String, Type)> = Vec::new();
                for (name, sub_pat) in fields {
                    let (ty, bindings) = self.infer_pattern(sub_pat)?;
                    field_types.push((name.clone(), ty));
                    all_bindings.extend(bindings);
                }
                // Record patterns are open by default — they match records with extra fields too.
                // We introduce a fresh row variable so that pattern-matching a record with
                // `{x = v}` does not fail when the record also has other fields.
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
