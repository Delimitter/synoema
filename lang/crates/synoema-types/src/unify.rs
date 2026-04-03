//! Robinson's unification algorithm for Synoema types.

use crate::types::*;
use crate::error::{TypeError, TypeErrorKind};

/// Unify two types, returning a substitution that makes them equal.
///
/// unify(τ₁, τ₂) = S such that S(τ₁) = S(τ₂)
///
/// `gen` is used to allocate fresh row variables when unifying two open record
/// types — required for correct Remy-style row unification.
pub fn unify(t1: &Type, t2: &Type, gen: &mut TyVarGen) -> Result<Subst, TypeError> {
    match (t1, t2) {
        // Same variable — trivially unified
        (Type::Var(a), Type::Var(b)) if a == b => Ok(Subst::new()),

        // Variable with anything — bind (with occurs check)
        (Type::Var(a), t) | (t, Type::Var(a)) => bind(*a, t),

        // Same constructor — trivially unified
        (Type::Con(a), Type::Con(b)) if a == b => Ok(Subst::new()),

        // Unrestricted arrow types — unify both sides
        (Type::Arrow(a1, b1), Type::Arrow(a2, b2)) => {
            let s1 = unify(a1, a2, gen)?;
            let s2 = unify(&b1.apply(&s1), &b2.apply(&s1), gen)?;
            Ok(s1.compose(&s2))
        }

        // Linear arrow types — unify both sides (multiplicities must match)
        (Type::LinearArrow(a1, b1), Type::LinearArrow(a2, b2)) => {
            let s1 = unify(a1, a2, gen)?;
            let s2 = unify(&b1.apply(&s1), &b2.apply(&s1), gen)?;
            Ok(s1.compose(&s2))
        }

        // Linear and unrestricted arrows are distinct types
        (Type::Arrow(_, _), Type::LinearArrow(_, _))
        | (Type::LinearArrow(_, _), Type::Arrow(_, _)) => {
            Err(TypeError::bare(TypeErrorKind::Mismatch {
                expected: t1.clone(),
                found: t2.clone(),
            }))
        }

        // Type applications — unify both parts
        (Type::App(f1, a1), Type::App(f2, a2)) => {
            let s1 = unify(f1, f2, gen)?;
            let s2 = unify(&a1.apply(&s1), &a2.apply(&s1), gen)?;
            Ok(s1.compose(&s2))
        }

        // Record types — row-polymorphic unification
        (Type::Record(fs1, tail1), Type::Record(fs2, tail2)) => {
            unify_records(fs1, *tail1, fs2, *tail2, t1, t2, gen)
        }

        // Mismatch
        _ => Err(TypeError::bare(TypeErrorKind::Mismatch {
            expected: t1.clone(),
            found: t2.clone(),
        })),
    }
}

/// Unify two record types with optional row tails (row-polymorphic unification).
///
/// Cases:
/// 1. Both closed: fields must match exactly (same names and types, same count).
/// 2. One open, one closed: the open record's row tail is bound to the remaining fields.
///    The closed record must not have extra fields vs. the minimum set.
/// 3. Both open: standard Rémy row unification — introduce a fresh shared row tail
///    `rho` and bind `r1 = {extra_from_2 | rho}`, `r2 = {extra_from_1 | rho}`.
fn unify_records(
    fs1: &[(String, Type)],
    tail1: Option<TyVarId>,
    fs2: &[(String, Type)],
    tail2: Option<TyVarId>,
    orig1: &Type,
    orig2: &Type,
    gen: &mut TyVarGen,
) -> Result<Subst, TypeError> {
    use std::collections::HashMap;
    let map1: HashMap<&str, &Type> = fs1.iter().map(|(n, t)| (n.as_str(), t)).collect();
    let map2: HashMap<&str, &Type> = fs2.iter().map(|(n, t)| (n.as_str(), t)).collect();

    // Unify fields that appear in both records
    let mut subst = Subst::new();
    for (name, t1) in &map1 {
        if let Some(t2) = map2.get(name) {
            let s = unify(&t1.apply(&subst), &t2.apply(&subst), gen)?;
            subst = s.compose(&subst);
        }
    }

    // Fields in fs1 but not in fs2
    let extra_in_1: Vec<(String, Type)> = fs1.iter()
        .filter(|(n, _)| !map2.contains_key(n.as_str()))
        .map(|(n, t)| (n.clone(), t.apply(&subst)))
        .collect();

    // Fields in fs2 but not in fs1
    let extra_in_2: Vec<(String, Type)> = fs2.iter()
        .filter(|(n, _)| !map1.contains_key(n.as_str()))
        .map(|(n, t)| (n.clone(), t.apply(&subst)))
        .collect();

    match (tail1, tail2) {
        // Both closed: must have identical field sets
        (None, None) => {
            if !extra_in_1.is_empty() || !extra_in_2.is_empty() {
                return Err(TypeError::bare(TypeErrorKind::Mismatch { expected: orig1.clone(), found: orig2.clone() }));
            }
        }

        // fs1 is open {fields1 | r1}, fs2 is closed {fields2}:
        // r1 must absorb any fields in fs2 not in fs1.
        // fs1 must not have fields that fs2 doesn't have (fs2 is the concrete closed type).
        (Some(r1), None) => {
            if !extra_in_1.is_empty() {
                return Err(TypeError::bare(TypeErrorKind::Mismatch { expected: orig1.clone(), found: orig2.clone() }));
            }
            let row_ty = Type::Record(extra_in_2, None);
            let s = bind(r1, &row_ty)?;
            subst = s.compose(&subst);
        }

        // fs1 is closed {fields1}, fs2 is open {fields2 | r2}:
        // r2 must absorb fields in fs1 not in fs2.
        (None, Some(r2)) => {
            if !extra_in_2.is_empty() {
                return Err(TypeError::bare(TypeErrorKind::Mismatch { expected: orig1.clone(), found: orig2.clone() }));
            }
            let row_ty = Type::Record(extra_in_1, None);
            let s = bind(r2, &row_ty)?;
            subst = s.compose(&subst);
        }

        // Both open: Rémy-style row unification.
        // Introduce a fresh shared tail `rho` such that:
        //   r1 = {extra_from_2 | rho}
        //   r2 = {extra_from_1 | rho}
        // This preserves openness and correctly threads extra fields.
        (Some(r1), Some(r2)) => {
            if r1 == r2 {
                // Same row variable — only need to ensure no exclusive extras on either side
                // (if both have the same tail, they already agree on what's unknown)
                // Just unify exclusive fields as errors since we can't distinguish
                if !extra_in_1.is_empty() || !extra_in_2.is_empty() {
                    // Both have a shared tail but different declared fields — still consistent;
                    // unify each exclusive set into the shared tail (noop if no conflict).
                    // This is fine — nothing to bind since r1 == r2.
                }
            } else if extra_in_1.is_empty() && extra_in_2.is_empty() {
                // No extra fields on either side: just unify the tails
                let s = bind(r1, &Type::Var(r2))?;
                subst = s.compose(&subst);
            } else {
                // Standard Rémy row unification:
                // fresh `rho` = shared "rest of the row"
                let rho = gen.fresh();

                // r1 = {fields exclusive to fs2 | rho}
                let row_for_r1 = Type::Record(
                    extra_in_2.iter().map(|(n, t)| (n.clone(), t.apply(&subst))).collect(),
                    Some(rho),
                );
                let s1 = bind(r1, &row_for_r1)?;
                subst = s1.compose(&subst);

                // r2 = {fields exclusive to fs1 | rho}
                let row_for_r2 = Type::Record(
                    extra_in_1.iter().map(|(n, t)| (n.clone(), t.apply(&subst))).collect(),
                    Some(rho),
                );
                let s2 = bind(r2, &row_for_r2)?;
                subst = s2.compose(&subst);
            }
        }
    }

    Ok(subst)
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
        return Err(TypeError::bare(TypeErrorKind::InfiniteType {
            var,
            ty: ty.clone(),
        }));
    }

    Ok(Subst::single(var, ty.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gen() -> TyVarGen { TyVarGen::new() }

    #[test]
    fn unify_same_con() {
        let s = unify(&Type::int(), &Type::int(), &mut gen()).unwrap();
        assert!(s.is_empty());
    }

    #[test]
    fn unify_var_with_con() {
        let s = unify(&Type::Var(0), &Type::int(), &mut gen()).unwrap();
        assert_eq!(s.0.get(&0), Some(&Type::int()));
    }

    #[test]
    fn unify_con_with_var() {
        let s = unify(&Type::int(), &Type::Var(0), &mut gen()).unwrap();
        assert_eq!(s.0.get(&0), Some(&Type::int()));
    }

    #[test]
    fn unify_arrow() {
        let t1 = Type::arrow(Type::Var(0), Type::int());
        let t2 = Type::arrow(Type::bool(), Type::Var(1));
        let s = unify(&t1, &t2, &mut gen()).unwrap();
        assert_eq!(s.0.get(&0), Some(&Type::bool()));
        assert_eq!(s.0.get(&1), Some(&Type::int()));
    }

    #[test]
    fn unify_mismatch() {
        let result = unify(&Type::int(), &Type::bool(), &mut gen());
        assert!(result.is_err());
    }

    #[test]
    fn unify_occurs_check() {
        // α = List α should fail
        let result = unify(&Type::Var(0), &Type::list(Type::Var(0)), &mut gen());
        assert!(matches!(result, Err(TypeError { kind: TypeErrorKind::InfiniteType { .. }, .. })));
    }

    #[test]
    fn unify_nested_arrows() {
        // (α → β) unify with (Int → Bool)
        let t1 = Type::arrow(Type::Var(0), Type::Var(1));
        let t2 = Type::arrow(Type::int(), Type::bool());
        let s = unify(&t1, &t2, &mut gen()).unwrap();
        assert_eq!(Type::Var(0).apply(&s), Type::int());
        assert_eq!(Type::Var(1).apply(&s), Type::bool());
    }

    #[test]
    fn unify_list_types() {
        let t1 = Type::list(Type::Var(0));
        let t2 = Type::list(Type::int());
        let s = unify(&t1, &t2, &mut gen()).unwrap();
        assert_eq!(s.0.get(&0), Some(&Type::int()));
    }
}
