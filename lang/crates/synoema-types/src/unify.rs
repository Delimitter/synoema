//! Robinson's unification algorithm for Synoema types.

use crate::types::*;
use crate::error::TypeError;

/// Unify two types, returning a substitution that makes them equal.
///
/// unify(τ₁, τ₂) = S such that S(τ₁) = S(τ₂)
pub fn unify(t1: &Type, t2: &Type) -> Result<Subst, TypeError> {
    match (t1, t2) {
        // Same variable — trivially unified
        (Type::Var(a), Type::Var(b)) if a == b => Ok(Subst::new()),

        // Variable with anything — bind (with occurs check)
        (Type::Var(a), t) | (t, Type::Var(a)) => bind(*a, t),

        // Same constructor — trivially unified
        (Type::Con(a), Type::Con(b)) if a == b => Ok(Subst::new()),

        // Arrow types — unify both sides
        (Type::Arrow(a1, b1), Type::Arrow(a2, b2)) => {
            let s1 = unify(a1, a2)?;
            let s2 = unify(&b1.apply(&s1), &b2.apply(&s1))?;
            Ok(s1.compose(&s2))
        }

        // Type applications — unify both parts
        (Type::App(f1, a1), Type::App(f2, a2)) => {
            let s1 = unify(f1, f2)?;
            let s2 = unify(&a1.apply(&s1), &a2.apply(&s1))?;
            Ok(s1.compose(&s2))
        }

        // Mismatch
        _ => Err(TypeError::Mismatch {
            expected: t1.clone(),
            found: t2.clone(),
        }),
    }
}

/// Bind a type variable to a type, with occurs check.
fn bind(var: TyVarId, ty: &Type) -> Result<Subst, TypeError> {
    // If ty is the same variable, no substitution needed
    if let Type::Var(v) = ty {
        if *v == var {
            return Ok(Subst::new());
        }
    }

    // Occurs check: prevent infinite types like α = List α
    if ty.ftv().contains(&var) {
        return Err(TypeError::InfiniteType {
            var,
            ty: ty.clone(),
        });
    }

    Ok(Subst::single(var, ty.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unify_same_con() {
        let s = unify(&Type::int(), &Type::int()).unwrap();
        assert!(s.is_empty());
    }

    #[test]
    fn unify_var_with_con() {
        let s = unify(&Type::Var(0), &Type::int()).unwrap();
        assert_eq!(s.0.get(&0), Some(&Type::int()));
    }

    #[test]
    fn unify_con_with_var() {
        let s = unify(&Type::int(), &Type::Var(0)).unwrap();
        assert_eq!(s.0.get(&0), Some(&Type::int()));
    }

    #[test]
    fn unify_arrow() {
        let t1 = Type::arrow(Type::Var(0), Type::int());
        let t2 = Type::arrow(Type::bool(), Type::Var(1));
        let s = unify(&t1, &t2).unwrap();
        assert_eq!(s.0.get(&0), Some(&Type::bool()));
        assert_eq!(s.0.get(&1), Some(&Type::int()));
    }

    #[test]
    fn unify_mismatch() {
        let result = unify(&Type::int(), &Type::bool());
        assert!(result.is_err());
    }

    #[test]
    fn unify_occurs_check() {
        // α = List α should fail
        let result = unify(&Type::Var(0), &Type::list(Type::Var(0)));
        assert!(matches!(result, Err(TypeError::InfiniteType { .. })));
    }

    #[test]
    fn unify_nested_arrows() {
        // (α → β) unify with (Int → Bool)
        let t1 = Type::arrow(Type::Var(0), Type::Var(1));
        let t2 = Type::arrow(Type::int(), Type::bool());
        let s = unify(&t1, &t2).unwrap();
        assert_eq!(Type::Var(0).apply(&s), Type::int());
        assert_eq!(Type::Var(1).apply(&s), Type::bool());
    }

    #[test]
    fn unify_list_types() {
        let t1 = Type::list(Type::Var(0));
        let t2 = Type::list(Type::int());
        let s = unify(&t1, &t2).unwrap();
        assert_eq!(s.0.get(&0), Some(&Type::int()));
    }
}
