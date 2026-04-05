// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

use syn::visit::Visit;
use syn::{self, Visibility};

const MAX_SEARCH_RESULTS: usize = 5;

// ── Public types ─────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FnInfo {
    pub name: String,
    pub vis: &'static str,
    pub sig: String,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct StructInfo {
    pub name: String,
    pub vis: &'static str,
    pub fields: Vec<String>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct EnumInfo {
    pub name: String,
    pub vis: &'static str,
    pub variants: Vec<String>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct FileIndex {
    pub functions: Vec<FnInfo>,
    pub structs: Vec<StructInfo>,
    pub enums: Vec<EnumInfo>,
    pub test_count: usize,
    pub loc: usize,
}

#[derive(Debug, Clone)]
pub struct CrateIndex {
    pub name: String,
    pub purpose: String,
    pub files: HashMap<PathBuf, FileIndex>,
    pub internal_deps: Vec<String>,
    pub total_loc: usize,
    pub total_tests: usize,
}

#[derive(Debug, Clone)]
pub struct CrateSummary {
    pub name: String,
    pub purpose: String,
    pub loc: usize,
    pub tests: usize,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub file: String,
    pub line: usize,
    pub context: String,
}

// ── Cache entry ──────────────────────────────────────────

struct CachedFile {
    mtime: SystemTime,
    index: FileIndex,
}

// ── Live Index ───────────────────────────────────────────

pub struct LiveIndex {
    cache: Mutex<HashMap<PathBuf, CachedFile>>,
    crates_cache: Mutex<Option<(SystemTime, Vec<CrateSummary>)>>,
    root: PathBuf,
}

static INDEX: std::sync::LazyLock<LiveIndex> = std::sync::LazyLock::new(|| {
    LiveIndex {
        cache: Mutex::new(HashMap::new()),
        crates_cache: Mutex::new(None),
        root: super::resources::synoema_root(),
    }
});

pub fn global() -> &'static LiveIndex {
    &INDEX
}

impl LiveIndex {
    pub fn get_file(&self, path: &Path) -> Option<FileIndex> {
        let abs = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.root.join(path)
        };

        if !abs.exists() {
            return None;
        }

        let mtime = std::fs::metadata(&abs).ok()?.modified().ok()?;

        let mut cache = self.cache.lock().ok()?;
        if let Some(cached) = cache.get(&abs) {
            if cached.mtime == mtime {
                return Some(cached.index.clone());
            }
        }

        let content = std::fs::read_to_string(&abs).ok()?;
        let index = parse_file_content(&content);
        cache.insert(abs, CachedFile { mtime, index: index.clone() });
        Some(index)
    }

    pub fn get_crate(&self, name: &str) -> Option<CrateIndex> {
        let crate_dir = self.root.join("lang/crates").join(name);
        if !crate_dir.exists() {
            return None;
        }

        let mut files = HashMap::new();
        let mut total_loc = 0;
        let mut total_tests = 0;

        collect_rs_files(&crate_dir.join("src"), &mut |path| {
            if let Some(idx) = self.get_file(path) {
                total_loc += idx.loc;
                total_tests += idx.test_count;
                files.insert(path.to_path_buf(), idx);
            }
            true
        });

        collect_rs_files(&crate_dir.join("tests"), &mut |path| {
            if let Some(idx) = self.get_file(path) {
                total_loc += idx.loc;
                total_tests += idx.test_count;
                files.insert(path.to_path_buf(), idx);
            }
            true
        });

        let purpose = read_crate_purpose(&crate_dir);
        let internal_deps = read_internal_deps(&crate_dir);

        Some(CrateIndex { name: name.to_string(), purpose, files, internal_deps, total_loc, total_tests })
    }

    pub fn all_crates(&self) -> Vec<CrateSummary> {
        let crates_dir = self.root.join("lang/crates");

        // Check cache: if directory mtime unchanged, return cached result
        let dir_mtime = std::fs::metadata(&crates_dir)
            .ok()
            .and_then(|m| m.modified().ok());

        if let Some(mtime) = dir_mtime {
            if let Ok(guard) = self.crates_cache.lock() {
                if let Some((cached_mtime, ref cached)) = *guard {
                    if cached_mtime == mtime {
                        return cached.clone();
                    }
                }
            }
        }

        // Cache miss — full scan
        let mut result = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&crates_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !entry.path().join("Cargo.toml").exists() {
                    continue;
                }
                if let Some(ci) = self.get_crate(&name) {
                    result.push(CrateSummary {
                        name: ci.name,
                        purpose: ci.purpose,
                        loc: ci.total_loc,
                        tests: ci.total_tests,
                    });
                }
            }
        }

        result.sort_by(|a, b| a.name.cmp(&b.name));

        // Update cache
        if let Some(mtime) = dir_mtime {
            if let Ok(mut guard) = self.crates_cache.lock() {
                *guard = Some((mtime, result.clone()));
            }
        }

        result
    }

    pub fn search(&self, query: &str, scope: &str) -> Vec<SearchResult> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        let search_code = scope == "code" || scope == "all";
        let search_docs = scope == "docs" || scope == "all";

        if search_code {
            let crates_dir = self.root.join("lang/crates");
            collect_rs_files(&crates_dir, &mut |path| {
                search_in_file(path, &query_lower, &self.root, &mut results);
                results.len() < MAX_SEARCH_RESULTS
            });
        }

        if results.len() < MAX_SEARCH_RESULTS && search_docs {
            collect_files_by_ext(&self.root.join("docs"), "md", &mut |path| {
                search_in_file(path, &query_lower, &self.root, &mut results);
                results.len() < MAX_SEARCH_RESULTS
            });
            if results.len() < MAX_SEARCH_RESULTS {
                collect_files_by_ext(&self.root.join("context"), "md", &mut |path| {
                    search_in_file(path, &query_lower, &self.root, &mut results);
                    results.len() < MAX_SEARCH_RESULTS
                });
            }
        }

        results.truncate(MAX_SEARCH_RESULTS);
        results
    }
}

// ── syn parsing ──────────────────────────────────────────

fn parse_file_content(content: &str) -> FileIndex {
    let file = match syn::parse_file(content) {
        Ok(f) => f,
        Err(_) => return FileIndex {
            functions: vec![], structs: vec![], enums: vec![],
            test_count: 0, loc: count_loc(content),
        },
    };

    let mut visitor = IndexVisitor::default();
    visitor.visit_file(&file);

    FileIndex {
        functions: visitor.functions,
        structs: visitor.structs,
        enums: visitor.enums,
        test_count: visitor.test_count,
        loc: count_loc(content),
    }
}

#[derive(Default)]
struct IndexVisitor {
    functions: Vec<FnInfo>,
    structs: Vec<StructInfo>,
    enums: Vec<EnumInfo>,
    test_count: usize,
}

impl<'ast> Visit<'ast> for IndexVisitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let is_test = node.attrs.iter().any(|a| a.path().is_ident("test"));
        if is_test {
            self.test_count += 1;
        }

        self.functions.push(FnInfo {
            name: node.sig.ident.to_string(),
            vis: vis_str(&node.vis),
            sig: sig_to_string(&node.sig),
            line: node.sig.ident.span().start().line,
        });

        syn::visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        let is_test = node.attrs.iter().any(|a| a.path().is_ident("test"));
        if is_test {
            self.test_count += 1;
        }

        self.functions.push(FnInfo {
            name: node.sig.ident.to_string(),
            vis: vis_str(&node.vis),
            sig: sig_to_string(&node.sig),
            line: node.sig.ident.span().start().line,
        });

        syn::visit::visit_impl_item_fn(self, node);
    }

    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        let fields: Vec<String> = match &node.fields {
            syn::Fields::Named(f) => f.named.iter().map(|f| {
                let name = f.ident.as_ref().map(|i| i.to_string()).unwrap_or_default();
                format!("{name}: {}", type_to_short(&f.ty))
            }).collect(),
            syn::Fields::Unnamed(f) => f.unnamed.iter().enumerate().map(|(i, f)| {
                format!("{i}: {}", type_to_short(&f.ty))
            }).collect(),
            syn::Fields::Unit => vec![],
        };

        self.structs.push(StructInfo {
            name: node.ident.to_string(),
            vis: vis_str(&node.vis),
            fields,
            line: node.ident.span().start().line,
        });

        syn::visit::visit_item_struct(self, node);
    }

    fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum) {
        let variants: Vec<String> = node.variants.iter()
            .map(|v| v.ident.to_string())
            .collect();

        self.enums.push(EnumInfo {
            name: node.ident.to_string(),
            vis: vis_str(&node.vis),
            variants,
            line: node.ident.span().start().line,
        });

        syn::visit::visit_item_enum(self, node);
    }
}

// ── Helpers ──────────────────────────────────────────────

fn vis_str(vis: &Visibility) -> &'static str {
    match vis {
        Visibility::Public(_) => "pub",
        Visibility::Restricted(_) => "pub(restricted)",
        Visibility::Inherited => "priv",
    }
}

fn sig_to_string(sig: &syn::Signature) -> String {
    let args: Vec<String> = sig.inputs.iter().map(|arg| {
        match arg {
            syn::FnArg::Receiver(r) => {
                if r.reference.is_some() {
                    if r.mutability.is_some() { "&mut self" } else { "&self" }
                } else {
                    "self"
                }.to_string()
            }
            syn::FnArg::Typed(t) => {
                let pat = quote_to_short(&t.pat);
                let ty = type_to_short(&t.ty);
                format!("{pat}: {ty}")
            }
        }
    }).collect();

    let ret = match &sig.output {
        syn::ReturnType::Default => String::new(),
        syn::ReturnType::Type(_, ty) => format!(" -> {}", type_to_short(ty)),
    };

    format!("({}){ret}", args.join(", "))
}

fn type_to_short(ty: &syn::Type) -> String {
    let s = quote::quote!(#ty).to_string();
    // Compact whitespace
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn quote_to_short(pat: &syn::Pat) -> String {
    let s = quote::quote!(#pat).to_string();
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn count_loc(content: &str) -> usize {
    content.lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with("//")
        })
        .count()
}

fn collect_rs_files(dir: &Path, cb: &mut dyn FnMut(&Path) -> bool) {
    if !dir.exists() {
        return;
    }
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_rs_files(&path, cb);
            } else if path.extension().is_some_and(|e| e == "rs") && !cb(&path) {
                return;
            }
        }
    }
}

fn collect_files_by_ext(dir: &Path, ext: &str, cb: &mut dyn FnMut(&Path) -> bool) {
    if !dir.exists() {
        return;
    }
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_files_by_ext(&path, ext, cb);
            } else if path.extension().is_some_and(|e| e == ext) && !cb(&path) {
                return;
            }
        }
    }
}

fn search_in_file(path: &Path, query: &str, root: &Path, results: &mut Vec<SearchResult>) {
    if results.len() >= MAX_SEARCH_RESULTS {
        return;
    }
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };
    let rel = path.strip_prefix(root).unwrap_or(path);
    let lines: Vec<&str> = content.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        if results.len() >= MAX_SEARCH_RESULTS {
            break;
        }
        if line.to_lowercase().contains(query) {
            let ctx_start = i.saturating_sub(1);
            let ctx_end = (i + 2).min(lines.len());
            let context = lines[ctx_start..ctx_end].join("\n");
            results.push(SearchResult {
                file: rel.display().to_string(),
                line: i + 1,
                context,
            });
        }
    }
}

fn read_crate_purpose(crate_dir: &Path) -> String {
    let cargo_toml = crate_dir.join("Cargo.toml");
    if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
        for line in content.lines() {
            if let Some(desc) = line.strip_prefix("description") {
                let desc = desc.trim().trim_start_matches('=').trim().trim_matches('"');
                if !desc.is_empty() {
                    return desc.to_string();
                }
            }
        }
    }
    String::new()
}

fn read_internal_deps(crate_dir: &Path) -> Vec<String> {
    let cargo_toml = crate_dir.join("Cargo.toml");
    let mut deps = Vec::new();
    if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
        for line in content.lines() {
            if line.contains("path = \"../../lang/crates/") || line.contains("path = \"../") {
                if let Some(name) = line.split_whitespace().next() {
                    let name = name.trim();
                    if !name.starts_with('[') && !name.starts_with('#') {
                        deps.push(name.to_string());
                    }
                }
            }
        }
    }
    deps
}

// ── Tests ────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_rust_code() {
        let code = r#"
            pub fn hello(name: &str) -> String {
                format!("hello {name}")
            }

            fn private_fn() {}

            pub struct Foo {
                pub x: i32,
                y: String,
            }

            pub enum Bar {
                A,
                B(i32),
                C { x: f64 },
            }

            #[test]
            fn test_something() {
                assert!(true);
            }
        "#;

        let index = parse_file_content(code);
        assert_eq!(index.functions.len(), 3);
        assert_eq!(index.functions[0].name, "hello");
        assert_eq!(index.functions[0].vis, "pub");
        assert_eq!(index.functions[1].name, "private_fn");
        assert_eq!(index.functions[1].vis, "priv");
        assert_eq!(index.structs.len(), 1);
        assert_eq!(index.structs[0].name, "Foo");
        assert_eq!(index.structs[0].fields.len(), 2);
        assert_eq!(index.enums.len(), 1);
        assert_eq!(index.enums[0].name, "Bar");
        assert_eq!(index.enums[0].variants, vec!["A", "B", "C"]);
        assert_eq!(index.test_count, 1);
    }

    #[test]
    fn parse_empty_file() {
        let index = parse_file_content("");
        assert!(index.functions.is_empty());
        assert!(index.structs.is_empty());
        assert!(index.enums.is_empty());
        assert_eq!(index.test_count, 0);
    }

    #[test]
    fn parse_invalid_syntax_returns_empty() {
        let index = parse_file_content("this is not valid rust {{{}}}}");
        assert!(index.functions.is_empty());
    }

    #[test]
    fn count_loc_skips_blanks_and_comments() {
        let code = "fn main() {}\n\n// comment\n   \nlet x = 1;\n";
        assert_eq!(count_loc(code), 2); // fn main() {} and let x = 1;
    }

    #[test]
    fn all_crates_returns_consistent_results() {
        let idx = global();
        let first = idx.all_crates();
        let second = idx.all_crates(); // should hit cache
        assert_eq!(first.len(), second.len());
        for (a, b) in first.iter().zip(second.iter()) {
            assert_eq!(a.name, b.name);
            assert_eq!(a.loc, b.loc);
        }
    }

    #[test]
    fn search_returns_at_most_max_results() {
        // "fn" appears in virtually every .rs file — should cap at MAX_SEARCH_RESULTS
        let idx = global();
        let results = idx.search("fn", "code");
        assert!(results.len() <= MAX_SEARCH_RESULTS);
        assert_eq!(results.len(), MAX_SEARCH_RESULTS);
    }
}
