//! Core type representations for Synoema's Hindley-Milner type system.

use std::collections::{HashMap, HashSet};
use std::fmt;

/// Unique identifier for type variables
pub type TyVarId = u32;

/// A monomorphic type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// Type variable: α, β, γ...
    Var(TyVarId),
    /// Type constant: Int, Float, Bool, String, Char
    Con(String),
    /// Function type: τ₁ → τ₂
    Arrow(Box<Type>, Box<Type>),
    /// Type constructor application: T τ₁ ... τₙ  (e.g. List Int, Maybe a)
    App(Box<Type>, Box<Type>),
    /// Structural record type: {name: T1, age: T2}
    /// The second field is a row-tail variable: None = closed record,
    /// Some(r) = open record {fields... | r} (row polymorphism).
    Record(Vec<(String, Type)>, Option<TyVarId>),
}

impl Type {
    pub fn int() -> Self { Type::Con("Int".into()) }
    pub fn float() -> Self { Type::Con("Float".into()) }
    pub fn bool() -> Self { Type::Con("Bool".into()) }
    pub fn string() -> Self { Type::Con("String".into()) }
    pub fn char() -> Self { Type::Con("Char".into()) }
    pub fn unit() -> Self { Type::Con("Unit".into()) }
    pub fn list(elem: Type) -> Self {
        Type::App(Box::new(Type::Con("List".into())), Box::new(elem))
    }
    pub fn arrow(from: Type, to: Type) -> Self {
        Type::Arrow(Box::new(from), Box::new(to))
    }

    /// Collect all free type variables in this type
    pub fn ftv(&self) -> HashSet<TyVarId> {
        match self {
            Type::Var(id) => {
                let mut s = HashSet::new();
                s.insert(*id);
                s
            }
            Type::Con(_) => HashSet::new(),
            Type::Arrow(a, b) | Type::App(a, b) => {
                let mut s = a.ftv();
                s.extend(b.ftv());
                s
            }
            Type::Record(fields, row_tail) => {
                let mut acc = fields.iter().fold(HashSet::new(), |mut a, (_, ty)| {
                    a.extend(ty.ftv()); a
                });
                if let Some(r) = row_tail {
                    acc.insert(*r);
                }
                acc
            }
        }
    }

    /// Apply a substitution to this type
    pub fn apply(&self, subst: &Subst) -> Type {
        match self {
            Type::Var(id) => {
                if let Some(ty) = subst.0.get(id) {
                    ty.apply(subst) // recursive apply for chained substitutions
                } else {
                    self.clone()
                }
            }
            Type::Con(_) => self.clone(),
            Type::Arrow(a, b) => Type::Arrow(
                Box::new(a.apply(subst)),
                Box::new(b.apply(subst)),
            ),
            Type::App(a, b) => Type::App(
                Box::new(a.apply(subst)),
                Box::new(b.apply(subst)),
            ),
            Type::Record(fields, row_tail) => {
                // Apply substitution to field types
                let new_fields = fields.iter().map(|(n, t)| (n.clone(), t.apply(subst))).collect();
                // If the row tail variable has a substitution, resolve it
                match row_tail {
                    None => Type::Record(new_fields, None),
                    Some(r) => {
                        if let Some(bound) = subst.0.get(r) {
                            // The row tail variable was substituted — merge the bound type
                            let bound = bound.apply(subst);
                            match bound {
                                Type::Record(extra_fields, inner_tail) => {
                                    // Merge: original fields + extra fields, propagate tail
                                    let mut merged = new_fields;
                                    for (n, t) in extra_fields {
                                        if !merged.iter().any(|(existing, _)| *existing == n) {
                                            merged.push((n, t));
                                        }
                                    }
                                    Type::Record(merged, inner_tail)
                                }
                                Type::Var(v) => Type::Record(new_fields, Some(v)),
                                _ => Type::Record(new_fields, Some(*r)),
                            }
                        } else {
                            Type::Record(new_fields, Some(*r))
                        }
                    }
                }
            }
        }
    }

    pub fn record(fields: Vec<(impl Into<String>, Type)>) -> Self {
        Type::Record(fields.into_iter().map(|(n, t)| (n.into(), t)).collect(), None)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Var(id) => {
                // Display as a, b, c, ..., z, a1, b1, ...
                let idx = *id as usize;
                if idx < 26 {
                    write!(f, "{}", (b'a' + idx as u8) as char)
                } else {
                    write!(f, "{}{}", (b'a' + (idx % 26) as u8) as char, idx / 26)
                }
            }
            Type::Con(name) => write!(f, "{}", name),
            Type::Arrow(a, b) => {
                match a.as_ref() {
                    Type::Arrow(_, _) => write!(f, "({}) -> {}", a, b),
                    _ => write!(f, "{} -> {}", a, b),
                }
            }
            Type::App(con, arg) => {
                match arg.as_ref() {
                    Type::App(_, _) | Type::Arrow(_, _) => write!(f, "{} ({})", con, arg),
                    _ => write!(f, "{} {}", con, arg),
                }
            }
            Type::Record(fields, row_tail) => {
                write!(f, "{{")?;
                for (i, (name, ty)) in fields.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}: {}", name, ty)?;
                }
                if let Some(r) = row_tail {
                    if !fields.is_empty() { write!(f, " | ")?; }
                    write!(f, "{}", Type::Var(*r))?;
                }
                write!(f, "}}")
            }
        }
    }
}

/// A polymorphic type scheme: ∀α₁...αₙ. τ
#[derive(Debug, Clone, PartialEq)]
pub struct Scheme {
    /// Bound type variables
    pub vars: Vec<TyVarId>,
    /// The underlying monomorphic type
    pub ty: Type,
}

impl Scheme {
    /// A monomorphic scheme (no quantified variables)
    pub fn mono(ty: Type) -> Self {
        Scheme { vars: Vec::new(), ty }
    }

    /// Free type variables (those NOT bound by the quantifier)
    pub fn ftv(&self) -> HashSet<TyVarId> {
        let mut s = self.ty.ftv();
        for v in &self.vars {
            s.remove(v);
        }
        s
    }

    /// Apply substitution (only to free variables)
    pub fn apply(&self, subst: &Subst) -> Scheme {
        // Remove bound variables from substitution
        let mut restricted = subst.clone();
        for v in &self.vars {
            restricted.0.remove(v);
        }
        Scheme {
            vars: self.vars.clone(),
            ty: self.ty.apply(&restricted),
        }
    }
}

impl fmt::Display for Scheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.vars.is_empty() {
            write!(f, "{}", self.ty)
        } else {
            write!(f, "∀")?;
            for (i, v) in self.vars.iter().enumerate() {
                if i > 0 { write!(f, " ")?; }
                let ty = Type::Var(*v);
                write!(f, "{}", ty)?;
            }
            write!(f, ". {}", self.ty)
        }
    }
}

// ── Substitution ────────────────────────────────────────

/// A substitution: mapping from type variables to types
#[derive(Debug, Clone, Default)]
pub struct Subst(pub HashMap<TyVarId, Type>);

impl Subst {
    pub fn new() -> Self { Subst(HashMap::new()) }

    /// Single-variable substitution
    pub fn single(var: TyVarId, ty: Type) -> Self {
        let mut m = HashMap::new();
        m.insert(var, ty);
        Subst(m)
    }

    /// Compose two substitutions: apply s1 then s2
    /// (s2 ∘ s1)(τ) = s2(s1(τ))
    pub fn compose(&self, other: &Subst) -> Subst {
        let mut result: HashMap<TyVarId, Type> = self.0.iter()
            .map(|(k, v)| (*k, v.apply(other)))
            .collect();
        // Add bindings from other that aren't in self
        for (k, v) in &other.0 {
            result.entry(*k).or_insert_with(|| v.clone());
        }
        Subst(result)
    }

    pub fn is_empty(&self) -> bool { self.0.is_empty() }
}

// ── Fresh Variable Generator ────────────────────────────

/// Generator for fresh type variable IDs
pub struct TyVarGen {
    next: TyVarId,
}

impl TyVarGen {
    pub fn new() -> Self { TyVarGen { next: 0 } }

    pub fn fresh(&mut self) -> TyVarId {
        let id = self.next;
        self.next += 1;
        id
    }

    pub fn fresh_var(&mut self) -> Type {
        Type::Var(self.fresh())
    }
}

// ── Type Environment ────────────────────────────────────

/// Type environment: Γ = mapping from variable names to type schemes
#[derive(Debug, Clone, Default)]
pub struct TypeEnv {
    bindings: HashMap<String, Scheme>,
}

impl TypeEnv {
    pub fn new() -> Self { TypeEnv { bindings: HashMap::new() } }

    pub fn insert(&mut self, name: String, scheme: Scheme) {
        self.bindings.insert(name, scheme);
    }

    pub fn lookup(&self, name: &str) -> Option<&Scheme> {
        self.bindings.get(name)
    }

    pub fn remove(&mut self, name: &str) {
        self.bindings.remove(name);
    }

    /// Free type variables of the entire environment
    pub fn ftv(&self) -> HashSet<TyVarId> {
        self.bindings.values()
            .flat_map(|s| s.ftv())
            .collect()
    }

    /// Apply substitution to all schemes in the environment
    pub fn apply(&self, subst: &Subst) -> TypeEnv {
        TypeEnv {
            bindings: self.bindings.iter()
                .map(|(k, v)| (k.clone(), v.apply(subst)))
                .collect(),
        }
    }

    /// Generalize a type into a scheme by quantifying over
    /// variables that are free in the type but not in the environment
    pub fn generalize(&self, ty: &Type) -> Scheme {
        let env_ftv = self.ftv();
        let ty_ftv = ty.ftv();
        let vars: Vec<TyVarId> = ty_ftv.difference(&env_ftv).copied().collect();
        Scheme { vars, ty: ty.clone() }
    }
}
