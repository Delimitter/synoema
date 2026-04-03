// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

//! Multi-file import resolver.
//!
//! Recursively loads imported files, detects circular imports,
//! and merges everything into a single flat Program.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::{Program, parse};
use synoema_lexer::Span;

/// Error from the import resolver.
#[derive(Debug, Clone)]
pub struct ImportError {
    pub message: String,
    pub span: Span,
    pub code: ImportErrorCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportErrorCode {
    Cycle,
    NotFound,
    ParseError,
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Resolve all `import "path"` declarations in a program by loading
/// and merging imported files into a single flat Program.
///
/// - `base_dir`: directory of the file being compiled (for relative path resolution)
/// - Circular imports are detected and reported as errors
/// - Diamond imports (same file imported twice) are loaded only once
pub fn resolve_imports(program: Program, base_dir: &Path) -> Result<Program, ImportError> {
    if program.imports.is_empty() {
        return Ok(program);
    }

    let mut seen = HashSet::new();
    let mut stack = Vec::new();
    let mut merged_decls = Vec::new();
    let mut merged_modules = Vec::new();
    let mut merged_uses = Vec::new();

    // Resolve each import from the root program
    for imp in &program.imports {
        resolve_recursive(
            &imp.path,
            imp.span,
            base_dir,
            &mut seen,
            &mut stack,
            &mut merged_decls,
            &mut merged_modules,
            &mut merged_uses,
        )?;
    }

    // Append the root file's own declarations AFTER imports
    merged_modules.extend(program.modules);
    merged_uses.extend(program.uses);
    merged_decls.extend(program.decls);

    Ok(Program {
        imports: vec![], // Resolved — no more imports
        decls: merged_decls,
        modules: merged_modules,
        uses: merged_uses,
    })
}

fn resolve_recursive(
    import_path: &str,
    span: Span,
    base_dir: &Path,
    seen: &mut HashSet<PathBuf>,
    stack: &mut Vec<PathBuf>,
    decls: &mut Vec<crate::ast::Decl>,
    modules: &mut Vec<crate::ast::ModuleDecl>,
    uses: &mut Vec<crate::ast::UseDecl>,
) -> Result<(), ImportError> {
    let resolved = base_dir.join(import_path);
    let canonical = resolved.canonicalize().map_err(|_| ImportError {
        message: format!("file not found: {}", resolved.display()),
        span,
        code: ImportErrorCode::NotFound,
    })?;

    // Cycle detection
    if stack.contains(&canonical) {
        let cycle: Vec<String> = stack.iter()
            .map(|p| p.file_name().unwrap_or_default().to_string_lossy().to_string())
            .collect();
        let current = canonical.file_name().unwrap_or_default().to_string_lossy();
        return Err(ImportError {
            message: format!("circular import detected: {} → {}", cycle.join(" → "), current),
            span,
            code: ImportErrorCode::Cycle,
        });
    }

    // Diamond: already loaded
    if seen.contains(&canonical) {
        return Ok(());
    }

    seen.insert(canonical.clone());
    stack.push(canonical.clone());

    // Read and parse
    let source = std::fs::read_to_string(&canonical).map_err(|e| ImportError {
        message: format!("cannot read {}: {}", canonical.display(), e),
        span,
        code: ImportErrorCode::NotFound,
    })?;

    let program = parse(&source).map_err(|e| ImportError {
        message: format!("parse error in {}: {}", canonical.display(), e),
        span,
        code: ImportErrorCode::ParseError,
    })?;

    // Resolve this file's imports first (recursively)
    let file_dir = canonical.parent().unwrap_or(base_dir);
    for imp in &program.imports {
        resolve_recursive(&imp.path, imp.span, file_dir, seen, stack, decls, modules, uses)?;
    }

    // Then add this file's declarations
    modules.extend(program.modules);
    uses.extend(program.uses);
    decls.extend(program.decls);

    stack.pop();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn make_temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("synoema_test_{}", name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("failed to create temp dir");
        dir
    }

    #[test]
    fn import_basic() {
        let dir = make_temp_dir("import_basic");
        fs::write(dir.join("math.sno"), "double x = x + x\n").unwrap();

        let program = parse("import \"math.sno\"\nmain = double 21").unwrap();
        let resolved = resolve_imports(program, &dir).unwrap();

        assert!(resolved.imports.is_empty());
        let names: Vec<_> = resolved.decls.iter().filter_map(|d| {
            if let crate::ast::Decl::Func { name, .. } = d { Some(name.as_str()) } else { None }
        }).collect();
        assert!(names.contains(&"double"), "imported function missing");
        assert!(names.contains(&"main"), "main function missing");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn import_diamond() {
        let dir = make_temp_dir("import_diamond");
        fs::write(dir.join("base.sno"), "base_val = 1\n").unwrap();
        fs::write(dir.join("a.sno"), "import \"base.sno\"\na_val = 2\n").unwrap();
        fs::write(dir.join("b.sno"), "import \"base.sno\"\nb_val = 3\n").unwrap();

        let program = parse("import \"a.sno\"\nimport \"b.sno\"\nmain = 42").unwrap();
        let resolved = resolve_imports(program, &dir).unwrap();

        let names: Vec<_> = resolved.decls.iter().filter_map(|d| {
            if let crate::ast::Decl::Func { name, .. } = d { Some(name.clone()) } else { None }
        }).collect();
        let base_count = names.iter().filter(|n| n.as_str() == "base_val").count();
        assert_eq!(base_count, 1, "diamond: base_val should appear exactly once");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn import_circular_detected() {
        let dir = make_temp_dir("import_circular");
        fs::write(dir.join("a.sno"), "import \"b.sno\"\na_val = 1\n").unwrap();
        fs::write(dir.join("b.sno"), "import \"a.sno\"\nb_val = 2\n").unwrap();

        let program = parse("import \"a.sno\"\nmain = 42").unwrap();
        let result = resolve_imports(program, &dir);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ImportErrorCode::Cycle);
        assert!(err.message.contains("circular import"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn import_not_found() {
        let dir = make_temp_dir("import_notfound");

        let program = parse("import \"nonexistent.sno\"\nmain = 42").unwrap();
        let result = resolve_imports(program, &dir);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ImportErrorCode::NotFound);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn import_no_imports_passthrough() {
        let program = parse("main = 42").unwrap();
        let resolved = resolve_imports(program.clone(), Path::new(".")).unwrap();
        assert_eq!(resolved.decls.len(), program.decls.len());
    }

    #[test]
    fn import_with_module() {
        let dir = make_temp_dir("import_module");
        fs::write(dir.join("math.sno"), "mod Math\n  square x = x * x\n").unwrap();

        let program = parse("import \"math.sno\"\nuse Math (square)\nmain = square 5").unwrap();
        let resolved = resolve_imports(program, &dir).unwrap();

        assert_eq!(resolved.modules.len(), 1);
        assert_eq!(resolved.modules[0].name, "Math");
        assert_eq!(resolved.uses.len(), 1);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn import_nested() {
        let dir = make_temp_dir("import_nested");
        fs::write(dir.join("base.sno"), "base_fn x = x\n").unwrap();
        fs::write(dir.join("mid.sno"), "import \"base.sno\"\nmid_fn x = base_fn x\n").unwrap();

        let program = parse("import \"mid.sno\"\nmain = mid_fn 42").unwrap();
        let resolved = resolve_imports(program, &dir).unwrap();

        let names: Vec<_> = resolved.decls.iter().filter_map(|d| {
            if let crate::ast::Decl::Func { name, .. } = d { Some(name.as_str()) } else { None }
        }).collect();
        assert!(names.contains(&"base_fn"), "deeply imported function missing");
        assert!(names.contains(&"mid_fn"), "mid-level imported function missing");
        assert!(names.contains(&"main"), "main missing");

        let _ = fs::remove_dir_all(&dir);
    }
}
