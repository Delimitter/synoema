// SPDX-License-Identifier: BUSL-1.1
// Copyright (c) 2025-present Andrey Bubnov

//! Synoema JIT Runtime Library
//!
//! Provides heap-allocated data structures for JIT-compiled code.
//! All functions use `extern "C"` ABI and operate on i64 values.
//!
//! List representation: linked list of (head, tail) nodes.
//! - Nil = 0 (null pointer)
//! - Cons(h, t) = pointer to heap-allocated ListNode { head: i64, tail: i64 }
//!
//! String representation: tagged pointer scheme.
//! - Tag bit 1 (v & 2 == 2) means the value is a string pointer.
//! - Actual pointer: (v & !2) points to StrNode { len: i64, data: [u8] inline }.
//! - Heap allocations are 8-byte aligned, so bits 0-2 are free for tagging.
//! - Lists/ints/bools all have bit 1 clear (0 = nil, 1 = true, even ptrs for lists).
//!
//! Memory management: Phase 10.3 — region-based bump allocator.
//! All JIT heap objects are allocated from a thread-local arena.
//! Call `arena_reset()` after each top-level program run to reclaim all memory.

use std::alloc::{alloc, dealloc, Layout};
use std::cell::RefCell;

// ── Bump Allocator (Arena) ───────────────────────────────

const ARENA_SIZE: usize = 8 * 1024 * 1024; // 8 MB
const ARENA_ALIGN: usize = 8; // All JIT objects need at most 8-byte alignment
const MAX_REGION_DEPTH: usize = 64;

struct Arena {
    // Backing store: allocated with ARENA_ALIGN alignment so that
    // relative bump offsets produce correctly aligned pointers.
    ptr: *mut u8,
    offset: usize,
    // Overflow tracking: system-allocated blocks that arena_reset() must free.
    overflow_allocs: Vec<(*mut u8, Layout)>,
    overflow_warned: bool,
    // Region stack: saved offsets for automatic region-based memory reclamation.
    // region_enter() pushes current offset, region_exit() pops and restores.
    region_stack: [usize; MAX_REGION_DEPTH],
    region_depth: usize,
}

impl Arena {
    fn new() -> Self {
        let layout = Layout::from_size_align(ARENA_SIZE, ARENA_ALIGN).unwrap();
        let ptr = unsafe { alloc(layout) };
        if ptr.is_null() { panic!("Arena: allocation failed"); }
        Arena {
            ptr, offset: 0,
            overflow_allocs: Vec::new(), overflow_warned: false,
            region_stack: [0; MAX_REGION_DEPTH], region_depth: 0,
        }
    }

    fn alloc(&mut self, size: usize, align: usize) -> *mut u8 {
        // Compute absolute address of current bump position and align up
        let base = self.ptr as usize + self.offset;
        let aligned_abs = (base + align - 1) & !(align - 1);
        let new_offset = (aligned_abs - self.ptr as usize) + size;
        if new_offset > ARENA_SIZE {
            // Arena full: fall back to system allocator with tracking
            if !self.overflow_warned {
                eprintln!(
                    "[synoema] arena overflow: {} bytes requested, {}/{} used. \
                     Falling back to system allocator.",
                    size, self.offset, ARENA_SIZE
                );
                self.overflow_warned = true;
            }
            let layout = Layout::from_size_align(size, align).unwrap();
            let ptr = unsafe { alloc(layout) };
            self.overflow_allocs.push((ptr, layout));
            ptr
        } else {
            self.offset = new_offset;
            aligned_abs as *mut u8
        }
    }

    fn reset(&mut self) {
        self.offset = 0;
        self.overflow_warned = false;
        self.region_depth = 0;
        // Free all tracked overflow allocations
        for (ptr, layout) in self.overflow_allocs.drain(..) {
            unsafe { dealloc(ptr, layout); }
        }
    }

    fn region_enter(&mut self) {
        if self.region_depth >= MAX_REGION_DEPTH {
            eprintln!("[synoema] region depth limit ({}) reached, skipping", MAX_REGION_DEPTH);
            return;
        }
        self.region_stack[self.region_depth] = self.offset;
        self.region_depth += 1;
    }

    fn region_exit(&mut self) {
        if self.region_depth == 0 { return; }
        self.region_depth -= 1;
        self.offset = self.region_stack[self.region_depth];
    }
}

// Safety: Arena is only accessed through thread_local ARENA, so Send is fine.
unsafe impl Send for Arena {}
unsafe impl Sync for Arena {}

thread_local! {
    static ARENA: RefCell<Arena> = RefCell::new(Arena::new());
}

/// Reset the arena allocator — call after each top-level program run.
/// This reclaims all memory allocated since the last reset.
pub fn arena_reset() {
    ARENA.with(|a| a.borrow_mut().reset());
}

/// Save current arena offset (for per-scope reset in server loops).
pub extern "C" fn arena_save() -> i64 {
    ARENA.with(|a| a.borrow().offset as i64)
}

/// Restore arena to a previously saved offset (frees everything allocated since save).
/// WARNING: all pointers allocated after save become invalid.
pub extern "C" fn arena_restore(saved: i64) {
    ARENA.with(|a| {
        let mut arena = a.borrow_mut();
        let saved = saved as usize;
        if saved <= arena.offset {
            arena.offset = saved;
        }
    });
}

/// Enter a new memory region (push arena checkpoint).
/// Used by region inference to automatically manage per-scope allocations.
#[unsafe(no_mangle)]
pub extern "C" fn synoema_region_enter() -> i64 {
    ARENA.with(|a| { a.borrow_mut().region_enter(); 0 })
}

/// Exit current memory region (pop and restore arena offset).
/// All allocations since the matching region_enter become invalid.
#[unsafe(no_mangle)]
pub extern "C" fn synoema_region_exit() -> i64 {
    ARENA.with(|a| { a.borrow_mut().region_exit(); 0 })
}

#[inline]
fn arena_alloc(size: usize, align: usize) -> *mut u8 {
    ARENA.with(|a| a.borrow_mut().alloc(size, align))
}

// ── String Tag ──────────────────────────────────────────

const STR_TAG: i64 = 2; // bit 1

#[inline]
pub fn is_str(v: i64) -> bool { v & STR_TAG == STR_TAG }

// ── Float Tag ────────────────────────────────────────────
//
// Floats are heap-boxed: a FloatNode { bits: i64 } holds the f64 bits.
// Tagged pointer: bit 2 set, bits 0 and 1 clear (so no conflict with strings).
// Detection: (v & 7) == 4  (bits 0,1,2 form the tag nibble; 4 = 0b100).

const FLOAT_TAG: i64 = 4; // bit 2

// ConNode tag: bit 0 set, bits 1-2 clear → no conflict with STR(bit1) or FLOAT(bit2)
const CON_TAG: i64 = 1;
// RecordNode tag: bits 0+2 set (= 5) → no conflict with STR(2) or FLOAT(4) or CON(1)
const RECORD_TAG: i64 = 5;

#[inline]
pub fn is_con(v: i64) -> bool { (v as u64) >= 0x10000 && v & 7 == CON_TAG }
#[inline]
pub fn is_record(v: i64) -> bool { (v as u64) >= 0x10000 && v & 7 == RECORD_TAG }

#[repr(C)]
struct FloatNode {
    bits: i64, // f64 bits stored as i64
}

#[inline]
pub fn is_float(v: i64) -> bool { v & 7 == FLOAT_TAG }

#[inline]
fn float_ptr(v: i64) -> *const FloatNode { (v & !FLOAT_TAG) as *const FloatNode }

/// Allocate a FloatNode and return a tagged float pointer.
/// `bits` is the result of `f64::to_bits() as i64`.
#[unsafe(no_mangle)]
pub extern "C" fn synoema_float_new(bits: i64) -> i64 {
    let ptr = arena_alloc(
        std::mem::size_of::<FloatNode>(),
        std::mem::align_of::<FloatNode>(),
    ) as *mut FloatNode;
    if ptr.is_null() { panic!("synoema_float_new: allocation failed"); }
    unsafe {
        (*ptr).bits = bits;
        (ptr as i64) | FLOAT_TAG
    }
}

/// Extract the f64 value from a tagged float pointer.
#[unsafe(no_mangle)]
pub extern "C" fn synoema_float_get(tagged: i64) -> f64 {
    let p = float_ptr(tagged);
    f64::from_bits(unsafe { (*p).bits } as u64)
}

/// Add two tagged floats; returns a new tagged float.
#[unsafe(no_mangle)]
pub extern "C" fn synoema_float_add(a: i64, b: i64) -> i64 {
    let fa = synoema_float_get(a);
    let fb = synoema_float_get(b);
    synoema_float_new((fa + fb).to_bits() as i64)
}

/// Subtract two tagged floats; returns a new tagged float.
#[unsafe(no_mangle)]
pub extern "C" fn synoema_float_sub(a: i64, b: i64) -> i64 {
    let fa = synoema_float_get(a);
    let fb = synoema_float_get(b);
    synoema_float_new((fa - fb).to_bits() as i64)
}

/// Multiply two tagged floats; returns a new tagged float.
#[unsafe(no_mangle)]
pub extern "C" fn synoema_float_mul(a: i64, b: i64) -> i64 {
    let fa = synoema_float_get(a);
    let fb = synoema_float_get(b);
    synoema_float_new((fa * fb).to_bits() as i64)
}

/// Divide two tagged floats; returns a new tagged float.
#[unsafe(no_mangle)]
pub extern "C" fn synoema_float_div(a: i64, b: i64) -> i64 {
    let fa = synoema_float_get(a);
    let fb = synoema_float_get(b);
    synoema_float_new((fa / fb).to_bits() as i64)
}

/// Compare two tagged floats: a < b. Returns 0 or 1 as i64.
#[unsafe(no_mangle)]
pub extern "C" fn synoema_float_lt(a: i64, b: i64) -> i64 {
    let fa = synoema_float_get(a);
    let fb = synoema_float_get(b);
    if fa < fb { 1 } else { 0 }
}

/// Compare two tagged floats: a > b. Returns 0 or 1 as i64.
#[unsafe(no_mangle)]
pub extern "C" fn synoema_float_gt(a: i64, b: i64) -> i64 {
    let fa = synoema_float_get(a);
    let fb = synoema_float_get(b);
    if fa > fb { 1 } else { 0 }
}

/// Compare two tagged floats: a <= b. Returns 0 or 1 as i64.
#[unsafe(no_mangle)]
pub extern "C" fn synoema_float_lte(a: i64, b: i64) -> i64 {
    let fa = synoema_float_get(a);
    let fb = synoema_float_get(b);
    if fa <= fb { 1 } else { 0 }
}

/// Compare two tagged floats: a >= b. Returns 0 or 1 as i64.
#[unsafe(no_mangle)]
pub extern "C" fn synoema_float_gte(a: i64, b: i64) -> i64 {
    let fa = synoema_float_get(a);
    let fb = synoema_float_get(b);
    if fa >= fb { 1 } else { 0 }
}

/// Compare two tagged floats for equality. Returns 0 or 1 as i64.
#[unsafe(no_mangle)]
pub extern "C" fn synoema_float_eq(a: i64, b: i64) -> i64 {
    let fa = synoema_float_get(a);
    let fb = synoema_float_get(b);
    if fa == fb { 1 } else { 0 }
}

/// Raise a tagged float to a tagged float power; returns a new tagged float.
#[unsafe(no_mangle)]
pub extern "C" fn synoema_float_pow(a: i64, b: i64) -> i64 {
    let fa = synoema_float_get(a);
    let fb = synoema_float_get(b);
    synoema_float_new(fa.powf(fb).to_bits() as i64)
}

/// Square root of a tagged float; returns a new tagged float.
#[unsafe(no_mangle)]
pub extern "C" fn synoema_float_sqrt(x: i64) -> i64 {
    let f = synoema_float_get(x);
    synoema_float_new(f.sqrt().to_bits() as i64)
}

/// Absolute value of a tagged float; returns a new tagged float.
#[unsafe(no_mangle)]
pub extern "C" fn synoema_float_abs(x: i64) -> i64 {
    let f = synoema_float_get(x);
    synoema_float_new(f.abs().to_bits() as i64)
}

/// Floor of a tagged float; returns a new tagged float.
#[unsafe(no_mangle)]
pub extern "C" fn synoema_float_floor(x: i64) -> i64 {
    let f = synoema_float_get(x);
    synoema_float_new(f.floor().to_bits() as i64)
}

/// Ceiling of a tagged float; returns a new tagged float.
#[unsafe(no_mangle)]
pub extern "C" fn synoema_float_ceil(x: i64) -> i64 {
    let f = synoema_float_get(x);
    synoema_float_new(f.ceil().to_bits() as i64)
}

/// Round a tagged float to nearest integer (as float); returns a new tagged float.
#[unsafe(no_mangle)]
pub extern "C" fn synoema_float_round(x: i64) -> i64 {
    let f = synoema_float_get(x);
    synoema_float_new(f.round().to_bits() as i64)
}

/// Integer exponentiation: base^exp using a simple loop. Returns i64.
#[unsafe(no_mangle)]
pub extern "C" fn synoema_int_pow(base: i64, exp: i64) -> i64 {
    if exp < 0 { return 0; } // negative exponent → 0 for integers
    let mut result: i64 = 1;
    let mut b = base;
    let mut e = exp;
    // Fast exponentiation (binary method)
    while e > 0 {
        if e & 1 == 1 {
            result = result.wrapping_mul(b);
        }
        b = b.wrapping_mul(b);
        e >>= 1;
    }
    result
}

/// Absolute value of an integer. Returns i64.
#[unsafe(no_mangle)]
pub extern "C" fn synoema_abs_int(x: i64) -> i64 {
    x.abs()
}

/// Decode a tagged float pointer to f64 (for testing/display).
pub fn decode_float(v: i64) -> Option<f64> {
    if !is_float(v) { return None; }
    let raw = (v & !FLOAT_TAG) as usize;
    if raw < 0x1000 || raw % std::mem::align_of::<FloatNode>() != 0 {
        return None;
    }
    Some(synoema_float_get(v))
}

#[inline]
fn str_ptr(v: i64) -> *const StrNode { (v & !STR_TAG) as *const StrNode }

/// A heap-allocated string: { len: i64 } followed inline by the UTF-8 bytes.
#[repr(C)]
struct StrNode {
    len: i64,
    // UTF-8 bytes follow immediately after this field
}

/// A cons cell: (head, tail) where tail is a pointer to next node or 0 (nil)
#[repr(C)]
struct ListNode {
    head: i64,
    tail: i64, // pointer to next ListNode, or 0
}

// ── List Construction ───────────────────────────────────

/// Create an empty list (nil)
pub extern "C" fn synoema_nil() -> i64 {
    0
}

/// Cons: prepend element to list. Returns pointer to new node.
pub extern "C" fn synoema_cons(head: i64, tail: i64) -> i64 {
    let ptr = arena_alloc(
        std::mem::size_of::<ListNode>(),
        std::mem::align_of::<ListNode>(),
    ) as *mut ListNode;
    if ptr.is_null() {
        panic!("synoema_cons: allocation failed");
    }
    unsafe {
        (*ptr).head = head;
        (*ptr).tail = tail;
        ptr as i64
    }
}

/// Check if list is nil (empty). Returns 1 if nil, 0 if cons.
pub extern "C" fn synoema_is_nil(list: i64) -> i64 {
    if list == 0 { 1 } else { 0 }
}

/// Get head of a cons cell. Panics on nil.
pub extern "C" fn synoema_head(list: i64) -> i64 {
    if list == 0 {
        panic!("synoema_head: empty list");
    }
    unsafe { (*(list as *const ListNode)).head }
}

/// Get tail of a cons cell. Panics on nil.
pub extern "C" fn synoema_tail(list: i64) -> i64 {
    if list == 0 {
        panic!("synoema_tail: empty list");
    }
    unsafe { (*(list as *const ListNode)).tail }
}

/// Concatenate two lists or two strings: a ++ b
pub extern "C" fn synoema_concat(a: i64, b: i64) -> i64 {
    if is_str(a) {
        return synoema_str_concat(a, b);
    }
    if a == 0 {
        return b;
    }
    // Copy list `a`, appending `b` at the end
    let head = synoema_head(a);
    let tail = synoema_tail(a);
    synoema_cons(head, synoema_concat(tail, b))
}

/// Get length of a list or string
pub extern "C" fn synoema_length(list: i64) -> i64 {
    if is_str(list) {
        return synoema_str_length(list);
    }
    let mut count = 0i64;
    let mut cur = list;
    while cur != 0 {
        count += 1;
        cur = unsafe { (*(cur as *const ListNode)).tail };
    }
    count
}

// ── Closures ────────────────────────────────────────────

/// A heap-allocated closure: function pointer + environment pointer.
#[repr(C)]
struct ClosureNode {
    fn_ptr: i64,
    env_ptr: i64,
}

/// Allocate a closure node. Returns pointer as i64.
pub extern "C" fn synoema_make_closure(fn_ptr: i64, env_ptr: i64) -> i64 {
    let ptr = arena_alloc(
        std::mem::size_of::<ClosureNode>(),
        std::mem::align_of::<ClosureNode>(),
    ) as *mut ClosureNode;
    if ptr.is_null() { panic!("synoema_make_closure: allocation failed"); }
    unsafe {
        (*ptr).fn_ptr = fn_ptr;
        (*ptr).env_ptr = env_ptr;
        ptr as i64
    }
}

/// Allocate an environment array of `size` i64 slots. Returns pointer as i64.
pub extern "C" fn synoema_env_alloc(size: i64) -> i64 {
    if size == 0 { return 0; }
    let n = size as usize;
    let ptr = arena_alloc(
        n * std::mem::size_of::<i64>(),
        std::mem::align_of::<i64>(),
    ) as *mut i64;
    if ptr.is_null() { panic!("synoema_env_alloc: allocation failed"); }
    unsafe {
        std::ptr::write_bytes(ptr, 0, n);
        ptr as i64
    }
}

// ── Display ─────────────────────────────────────────────

/// Print a value (integer, float, string, or list). Returns the value.
pub extern "C" fn synoema_show_val(val: i64) -> i64 {
    if is_str(val) {
        let p = str_ptr(val);
        let len = unsafe { (*p).len } as usize;
        let data = unsafe { std::slice::from_raw_parts(p.add(1) as *const u8, len) };
        print!("{}", std::str::from_utf8(data).unwrap_or("<invalid utf8>"));
    } else if is_float(val) {
        print!("{}", synoema_float_get(val));
    } else if is_likely_list_ptr(val) {
        print_list(val);
    } else {
        print!("{}", val);
    }
    val
}

/// Print a value with newline
pub extern "C" fn synoema_println_val(val: i64) -> i64 {
    synoema_show_val(val);
    println!();
    val
}

/// Print an integer followed by newline
pub extern "C" fn synoema_print_int(val: i64) -> i64 {
    println!("{}", val);
    val
}

/// Print any tagged JIT value with newline. Returns 0 (unit).
/// Uses address-validated tag checks to avoid interpreting small integers as pointers.
pub extern "C" fn synoema_print_val(val: i64) -> i64 {
    // Only treat as a pointer if the address is plausibly a heap address (> 64KB).
    // Small integers (e.g. 42) can accidentally have tag bits set.
    let is_heap = (val as u64) >= 0x10000;
    if is_str(val) && is_heap {
        let p = str_ptr(val);
        let len = unsafe { (*p).len } as usize;
        let data = unsafe { std::slice::from_raw_parts(p.add(1) as *const u8, len) };
        print!("{}", std::str::from_utf8(data).unwrap_or("<invalid utf8>"));
    } else if is_float(val) && is_heap {
        print!("{}", synoema_float_get(val));
    } else if is_likely_list_ptr(val) {
        print_list(val);
    } else {
        // Plain integer or boolean (0=false/unit/nil, 1=true, n=integer)
        print!("{}", val);
    }
    println!();
    0 // unit
}

/// Read a line from stdin. Returns a tagged string value.
pub extern "C" fn synoema_readline() -> i64 {
    use std::io::BufRead;
    let stdin = std::io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).unwrap_or(0);
    if line.ends_with('\n') { line.pop(); }
    if line.ends_with('\r') { line.pop(); }
    let bytes = line.into_bytes().into_boxed_slice();
    let data_ptr = bytes.as_ptr() as i64;
    let len = bytes.len() as i64;
    Box::leak(bytes);
    synoema_str_new(data_ptr, len)
}

/// Print a list in [1 2 3] format
pub extern "C" fn synoema_print_list(list: i64) -> i64 {
    print_list(list);
    println!();
    list
}

fn print_list(list: i64) {
    print!("[");
    let mut cur = list;
    let mut first = true;
    while cur != 0 {
        if !first { print!(" "); }
        first = false;
        let node = unsafe { &*(cur as *const ListNode) };
        // Recursively handle nested lists
        if is_likely_list_ptr(node.head) {
            print_list(node.head);
        } else {
            print!("{}", node.head);
        }
        cur = node.tail;
    }
    print!("]");
}

/// Heuristic: does this i64 look like a list heap pointer?
/// Lists are raw 8-byte-aligned heap pointers with ALL low 3 bits clear.
/// Strings set bit 1, floats set bit 2, cons set bit 0, records set bits 0+2.
fn is_likely_list_ptr(val: i64) -> bool {
    val > 100_000 && val & 7 == 0
}

// ── String Support ──────────────────────────────────────

/// Allocate a StrNode from a raw byte pointer and length. Returns a tagged string pointer.
pub extern "C" fn synoema_str_new(data_ptr: i64, len: i64) -> i64 {
    let len_usize = len as usize;
    let total = std::mem::size_of::<StrNode>() + len_usize;
    let ptr = arena_alloc(total, std::mem::align_of::<StrNode>()) as *mut StrNode;
    if ptr.is_null() { panic!("synoema_str_new: allocation failed"); }
    unsafe {
        (*ptr).len = len;
        let dst = ptr.add(1) as *mut u8;
        std::ptr::copy_nonoverlapping(data_ptr as *const u8, dst, len_usize);
        (ptr as i64) | STR_TAG
    }
}

/// Convert an integer to its string representation. Returns a tagged string pointer.
pub extern "C" fn synoema_show_int(n: i64) -> i64 {
    let s = n.to_string();
    let bytes = s.as_bytes();
    let len = bytes.len();
    let total = std::mem::size_of::<StrNode>() + len;
    let ptr = arena_alloc(total, std::mem::align_of::<StrNode>()) as *mut StrNode;
    if ptr.is_null() { panic!("synoema_show_int: allocation failed"); }
    unsafe {
        (*ptr).len = len as i64;
        let dst = ptr.add(1) as *mut u8;
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), dst, len);
        (ptr as i64) | STR_TAG
    }
}

/// Concatenate two strings. Both must be tagged string pointers.
pub extern "C" fn synoema_str_concat(a: i64, b: i64) -> i64 {
    let pa = str_ptr(a);
    let pb = str_ptr(b);
    let la = unsafe { (*pa).len } as usize;
    let lb = unsafe { (*pb).len } as usize;
    let total_len = la + lb;
    let total = std::mem::size_of::<StrNode>() + total_len;
    let ptr = arena_alloc(total, std::mem::align_of::<StrNode>()) as *mut StrNode;
    if ptr.is_null() { panic!("synoema_str_concat: allocation failed"); }
    unsafe {
        (*ptr).len = total_len as i64;
        let dst = ptr.add(1) as *mut u8;
        std::ptr::copy_nonoverlapping(pa.add(1) as *const u8, dst, la);
        std::ptr::copy_nonoverlapping(pb.add(1) as *const u8, dst.add(la), lb);
        (ptr as i64) | STR_TAG
    }
}

/// Return the length (in bytes) of a tagged string.
pub extern "C" fn synoema_str_length(s: i64) -> i64 {
    let p = str_ptr(s);
    unsafe { (*p).len }
}

/// Compare two tagged strings for byte-equality. Returns 1 if equal, 0 otherwise.
pub extern "C" fn synoema_str_eq(a: i64, b: i64) -> i64 {
    let pa = str_ptr(a);
    let pb = str_ptr(b);
    let la = unsafe { (*pa).len } as usize;
    let lb = unsafe { (*pb).len } as usize;
    if la != lb { return 0; }
    let sa = unsafe { std::slice::from_raw_parts(pa.add(1) as *const u8, la) };
    let sb = unsafe { std::slice::from_raw_parts(pb.add(1) as *const u8, lb) };
    if sa == sb { 1 } else { 0 }
}

// ── String Stdlib ────────────────────────────────────────

/// Helper: extract (data_ptr, len) from a tagged string value.
#[inline]
unsafe fn str_data(v: i64) -> (*const u8, usize) {
    let p = str_ptr(v);
    let len = (*p).len as usize;
    let data = p.add(1) as *const u8;
    (data, len)
}

/// Substring extraction. Clamps from/to to [0, len], ensures to >= from.
pub extern "C" fn synoema_str_slice(s: i64, from: i64, to: i64) -> i64 {
    let (data, len) = unsafe { str_data(s) };
    let f = (from as usize).min(len);
    let t = (to as usize).min(len).max(f);
    let slice = unsafe { std::slice::from_raw_parts(data.add(f), t - f) };
    let result = unsafe { std::str::from_utf8_unchecked(slice) };
    alloc_str(result)
}

/// Byte-based substring search starting at `from`. Returns index or -1.
pub extern "C" fn synoema_str_find(s: i64, sub: i64, from: i64) -> i64 {
    let (s_data, s_len) = unsafe { str_data(s) };
    let (sub_data, sub_len) = unsafe { str_data(sub) };
    let f = from as usize;
    if f > s_len { return -1; }
    if sub_len == 0 { return f as i64; }
    let haystack = unsafe { std::slice::from_raw_parts(s_data.add(f), s_len - f) };
    let needle = unsafe { std::slice::from_raw_parts(sub_data, sub_len) };
    // Simple byte search
    for i in 0..=(haystack.len().saturating_sub(needle.len())) {
        if &haystack[i..i + needle.len()] == needle {
            return (f + i) as i64;
        }
    }
    -1
}

/// Returns 1 if `s` starts with `prefix`, 0 otherwise.
pub extern "C" fn synoema_str_starts_with(s: i64, prefix: i64) -> i64 {
    let (s_data, s_len) = unsafe { str_data(s) };
    let (p_data, p_len) = unsafe { str_data(prefix) };
    if p_len == 0 { return 1; }
    if p_len > s_len { return 0; }
    let s_slice = unsafe { std::slice::from_raw_parts(s_data, p_len) };
    let p_slice = unsafe { std::slice::from_raw_parts(p_data, p_len) };
    if s_slice == p_slice { 1 } else { 0 }
}

/// Trim leading and trailing ASCII whitespace.
pub extern "C" fn synoema_str_trim(s: i64) -> i64 {
    let (data, len) = unsafe { str_data(s) };
    let bytes = unsafe { std::slice::from_raw_parts(data, len) };
    let trimmed = match std::str::from_utf8(bytes) {
        Ok(s) => s.trim(),
        Err(_) => return s,
    };
    alloc_str(trimmed)
}

/// Return byte length of string as untagged i64.
pub extern "C" fn synoema_str_len(s: i64) -> i64 {
    let p = str_ptr(s);
    unsafe { (*p).len }
}

/// Escape string for JSON: \, ", \n, \r, \t.
pub extern "C" fn synoema_json_escape(s: i64) -> i64 {
    let (data, len) = unsafe { str_data(s) };
    let bytes = unsafe { std::slice::from_raw_parts(data, len) };
    let mut out = Vec::with_capacity(len + len / 4);
    for &b in bytes {
        match b {
            b'\\' => { out.push(b'\\'); out.push(b'\\'); }
            b'"'  => { out.push(b'\\'); out.push(b'"'); }
            b'\n' => { out.push(b'\\'); out.push(b'n'); }
            b'\r' => { out.push(b'\\'); out.push(b'r'); }
            b'\t' => { out.push(b'\\'); out.push(b't'); }
            _ => out.push(b),
        }
    }
    let result = unsafe { std::str::from_utf8_unchecked(&out) };
    alloc_str(result)
}

/// Universal equality: dispatches on string/float tag at runtime.
/// Returns 1 if equal, 0 otherwise. Works for ints, bools, strings, and floats.
#[unsafe(no_mangle)]
pub extern "C" fn synoema_val_eq(a: i64, b: i64) -> i64 {
    // Only treat as heap pointer types if the address is plausibly a real heap address.
    // Small integers (e.g. 2, 6) can accidentally have tag bits set.
    let a_heap = (a as u64) >= 0x10000;
    let b_heap = (b as u64) >= 0x10000;
    if (is_str(a) && a_heap) || (is_str(b) && b_heap) {
        if (is_str(a) && a_heap) && (is_str(b) && b_heap) {
            synoema_str_eq(a, b)
        } else {
            0
        }
    } else if (is_float(a) && a_heap) || (is_float(b) && b_heap) {
        if (is_float(a) && a_heap) && (is_float(b) && b_heap) {
            synoema_float_eq(a, b)
        } else {
            0
        }
    } else if is_con(a) && is_con(b) {
        // Structural Con equality: compare tag + fields recursively
        let base_a = (a & !CON_TAG) as *const i64;
        let base_b = (b & !CON_TAG) as *const i64;
        let tag_a = unsafe { *base_a };
        let tag_b = unsafe { *base_b };
        if tag_a != tag_b { return 0; }
        let arity = unsafe { *base_a.add(1) } as usize;
        for i in 0..arity {
            let fa = unsafe { *base_a.add(4 + i) };
            let fb = unsafe { *base_b.add(4 + i) };
            if synoema_val_eq(fa, fb) == 0 { return 0; }
        }
        1
    } else if is_con(a) || is_con(b) || is_record(a) || is_record(b) {
        if a == b { 1 } else { 0 }
    } else if is_likely_list_ptr(a) || is_likely_list_ptr(b) {
        synoema_list_eq(a, b)
    } else {
        if a == b { 1 } else { 0 }
    }
}

/// Format a single JIT value as a Rust String (for building show strings).
fn format_val(val: i64) -> String {
    let is_heap = (val as u64) >= 0x10000;
    if is_str(val) && is_heap {
        let p = str_ptr(val);
        let len = unsafe { (*p).len } as usize;
        let data = unsafe { std::slice::from_raw_parts(p.add(1) as *const u8, len) };
        std::str::from_utf8(data).unwrap_or("<invalid utf8>").to_string()
    } else if is_float(val) && is_heap {
        let f = synoema_float_get(val);
        if f.fract() == 0.0 && f.abs() < 1e15 { format!("{:.1}", f) } else { format!("{}", f) }
    } else if is_con(val) {
        // Decode ConNode name for nested show
        let base = (val & !CON_TAG) as *const i64;
        let arity    = unsafe { *base.add(1) } as usize;
        let name_ptr = unsafe { *base.add(2) } as *const u8;
        let name_len = unsafe { *base.add(3) } as usize;
        let name_str = if name_len == 0 || name_ptr.is_null() { "Con".to_string() } else {
            let bytes = unsafe { std::slice::from_raw_parts(name_ptr, name_len) };
            std::str::from_utf8(bytes).unwrap_or("Con").to_string()
        };
        if arity == 0 { return name_str; }
        let mut s = name_str;
        for i in 0..arity {
            let field = unsafe { *base.add(4 + i) };
            s.push(' ');
            let fs = format_val(field);
            if fs.contains(' ') { s.push('('); s.push_str(&fs); s.push(')'); }
            else { s.push_str(&fs); }
        }
        s
    } else if is_likely_list_ptr(val) {
        format_list(val)
    } else {
        val.to_string()
    }
}

/// Format a list pointer as "[a b c]"
fn format_list(list: i64) -> String {
    let mut s = String::from("[");
    let mut cur = list;
    let mut first = true;
    while cur != 0 {
        if !first { s.push(' '); }
        first = false;
        let node = unsafe { &*(cur as *const ListNode) };
        s.push_str(&format_val(node.head));
        cur = node.tail;
    }
    s.push(']');
    s
}

/// Allocate a tagged string from a Rust String.
fn alloc_str(s: &str) -> i64 {
    let bytes = s.as_bytes();
    let len = bytes.len();
    let total = std::mem::size_of::<StrNode>() + len;
    let ptr = arena_alloc(total, std::mem::align_of::<StrNode>()) as *mut StrNode;
    if ptr.is_null() { panic!("alloc_str: allocation failed"); }
    unsafe {
        (*ptr).len = len as i64;
        let dst = ptr.add(1) as *mut u8;
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), dst, len);
        (ptr as i64) | STR_TAG
    }
}

/// Convert any JIT value to a tagged string. Returns a tagged string pointer.
/// - int → decimal digits
/// - float → decimal representation
/// - string → identity (already a string)
/// - list → "[a b c]" format
/// - Con → "Name field0 field1 ..."
/// - Record → "{...}" (field names not stored at runtime)
pub extern "C" fn synoema_show_any(val: i64) -> i64 {
    let is_heap = (val as u64) >= 0x10000;
    if is_str(val) && is_heap {
        val // already a string, return as-is
    } else if is_float(val) && is_heap {
        alloc_str(&format_val(val))
    } else if is_con(val) {
        synoema_show_con(val)
    } else if is_record(val) {
        alloc_str("{..}") // field names not available at runtime
    } else if is_likely_list_ptr(val) {
        alloc_str(&format_list(val))
    } else {
        // Plain int (including 0=nil/false — indistinguishable at runtime)
        synoema_show_int(val)
    }
}

/// Convert a Bool value (0 = false, non-zero = true) to a tagged string "true"/"false".
pub extern "C" fn synoema_show_bool(v: i64) -> i64 {
    if v != 0 { alloc_str("true") } else { alloc_str("false") }
}

/// Convert a list to its string representation "[a b c]". Returns a tagged string.
pub extern "C" fn synoema_show_list(list: i64) -> i64 {
    alloc_str(&format_list(list))
}

/// Recursively compare two lists for equality. Returns 1 if equal, 0 otherwise.
pub extern "C" fn synoema_list_eq(a: i64, b: i64) -> i64 {
    let mut ca = a;
    let mut cb = b;
    loop {
        match (ca, cb) {
            (0, 0) => return 1, // both Nil
            (0, _) | (_, 0) => return 0, // one Nil, one not
            _ => {
                let na = unsafe { &*(ca as *const ListNode) };
                let nb = unsafe { &*(cb as *const ListNode) };
                // Compare heads recursively (handles nested lists / tagged values)
                if synoema_val_eq(na.head, nb.head) == 0 { return 0; }
                ca = na.tail;
                cb = nb.tail;
            }
        }
    }
}

/// Build a list [from..to] inclusive. Returns a tagged list (head=from, ..., head=to).
pub extern "C" fn synoema_range(from: i64, to: i64) -> i64 {
    // Build in reverse then it's already in order via recursion from the end.
    // Iterative approach: build from `to` down to `from`.
    let mut result = 0i64; // Nil
    let mut i = to;
    while i >= from {
        result = synoema_cons(i, result);
        i -= 1;
    }
    result
}

/// Decode a tagged string pointer to a Rust String (for display/testing).
///
/// Returns `None` for non-string values and also for values that pass the
/// `is_str` bit-check but are clearly not valid heap pointers (small integers
/// like 2, 6, 58, 59… where bit 1 happens to be set accidentally).
pub fn decode_str(v: i64) -> Option<String> {
    if !is_str(v) { return None; }
    // Sanity-check: untagged pointer must be 8-byte aligned and point to a
    // valid heap address (> 4096 to exclude small integers / null-page).
    let raw = (v & !STR_TAG) as usize;
    if raw < 0x1000 || raw % std::mem::align_of::<StrNode>() != 0 {
        return None;
    }
    let p = raw as *const StrNode;
    let len = unsafe { (*p).len } as usize;
    // Additional sanity: length must be reasonable (< 1 MB)
    if len > 1024 * 1024 { return None; }
    let data = unsafe { std::slice::from_raw_parts(p.add(1) as *const u8, len) };
    Some(std::str::from_utf8(data).unwrap_or("<invalid utf8>").to_string())
}

/// Display a JIT result value as a human-readable string.
pub fn display_value(v: i64) -> String {
    if let Some(s) = decode_str(v) {
        s
    } else if let Some(f) = decode_float(v) {
        format!("{}", f)
    } else {
        v.to_string()
    }
}

// ── Sum / Reduce helpers ────────────────────────────────

/// Sum all elements in a list
pub extern "C" fn synoema_sum(list: i64) -> i64 {
    let mut total = 0i64;
    let mut cur = list;
    while cur != 0 {
        let node = unsafe { &*(cur as *const ListNode) };
        total += node.head;
        cur = node.tail;
    }
    total
}

// ── Records ─────────────────────────────────────────────

/// RecordNode layout in memory:
/// [len: i64][field_hash_0: i64][val_0: i64][field_hash_1: i64][val_1: i64]...
/// A record pointer is a raw heap i64* pointer (no tag — records are not confused with ints/strings).
/// Note: RecordNode is only used as a memory-layout reference in comments; actual allocation uses i64 arrays.
#[repr(C)]
#[allow(dead_code)]
struct RecordNode {
    len: i64,
    // Followed by len * 2 i64 words: (hash, val) pairs
}

/// Compute a stable hash for a field name (FNV-1a 64-bit).
/// Used at compile time in compiler.rs and must match the runtime linear scan.
pub fn field_name_hash(name: &str) -> i64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in name.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h as i64
}

/// Allocate a RecordNode for `len` fields. Returns RECORD_TAG-tagged pointer.
/// Layout: [len: i64][(hash: i64, val: i64) × len]
#[unsafe(no_mangle)]
pub extern "C" fn synoema_record_new(len: i64) -> i64 {
    // 1 word for len + 2 words per field
    let total_words = 1 + len as usize * 2;
    let ptr = arena_alloc(
        total_words * std::mem::size_of::<i64>(),
        std::mem::align_of::<i64>(),
    ) as *mut i64;
    if ptr.is_null() { panic!("synoema_record_new: allocation failed"); }
    unsafe {
        std::ptr::write_bytes(ptr, 0, total_words);
        *ptr = len; // store len at offset 0
        (ptr as i64) | RECORD_TAG
    }
}

/// Store a field into a RecordNode at position `idx`. Strips RECORD_TAG.
#[unsafe(no_mangle)]
pub extern "C" fn synoema_record_set(rec: i64, idx: i64, hash: i64, val: i64) {
    let base = (rec & !RECORD_TAG) as *mut i64;
    let offset = (1 + idx * 2) as usize;
    unsafe {
        *base.add(offset) = hash;
        *base.add(offset + 1) = val;
    }
}

/// Linear scan to find the field matching `hash`. Returns its value.
/// Strips RECORD_TAG before use. Panics if hash is not found.
#[unsafe(no_mangle)]
pub extern "C" fn synoema_record_get(rec: i64, hash: i64) -> i64 {
    let base = (rec & !RECORD_TAG) as *const i64;
    let len = unsafe { *base } as usize;
    for i in 0..len {
        let slot = unsafe { *base.add(1 + i * 2) };
        if slot == hash {
            return unsafe { *base.add(1 + i * 2 + 1) };
        }
    }
    panic!("synoema_record_get: field not found (hash={})", hash);
}

// ── Algebraic Data Types ─────────────────────────────────

/// ConNode memory layout (pointer tagged with CON_TAG=1):
/// [tag: i64][arity: i64][name_ptr: i64][name_len: i64][field_0: i64]...[field_{arity-1}: i64]
/// - tag      = constructor index (0-based within its ADT definition)
/// - arity    = number of payload fields (for show/runtime inspection)
/// - name_ptr = raw pointer to constructor name bytes (static data section)
/// - name_len = byte length of constructor name
/// - fields follow at slot 4+idx
/// Returned pointer has CON_TAG=1 set in bit 0.

/// Allocate a new ConNode. `name_ptr`/`name_len` point to static name bytes.
/// Returns a CON_TAG-tagged pointer.
#[unsafe(no_mangle)]
pub extern "C" fn synoema_make_con(tag: i64, arity: i64, name_ptr: i64, name_len: i64) -> i64 {
    let n = 4 + arity as usize; // tag + arity + name_ptr + name_len + fields
    let ptr = arena_alloc(
        n * std::mem::size_of::<i64>(),
        std::mem::align_of::<i64>(),
    ) as *mut i64;
    if ptr.is_null() { panic!("synoema_make_con: allocation failed"); }
    unsafe {
        *ptr = tag;
        *ptr.add(1) = arity;
        *ptr.add(2) = name_ptr;
        *ptr.add(3) = name_len;
        std::ptr::write_bytes(ptr.add(4), 0, arity as usize);
        (ptr as i64) | CON_TAG
    }
}

/// Set payload field `idx` of a ConNode to `val`. Strips CON_TAG.
#[unsafe(no_mangle)]
pub extern "C" fn synoema_con_set(ptr: i64, idx: i64, val: i64) {
    let base = (ptr & !CON_TAG) as *mut i64;
    unsafe { *base.add(4 + idx as usize) = val; }
}

/// Load the numeric tag word of a ConNode (slot 0). Strips CON_TAG.
#[unsafe(no_mangle)]
pub extern "C" fn synoema_con_get_tag(ptr: i64) -> i64 {
    unsafe { *((ptr & !CON_TAG) as *const i64) }
}

/// Load payload field `idx` from a ConNode (slot 4+idx). Strips CON_TAG.
#[unsafe(no_mangle)]
pub extern "C" fn synoema_con_get_field(ptr: i64, idx: i64) -> i64 {
    unsafe { *((ptr & !CON_TAG) as *const i64).add(4 + idx as usize) }
}

/// Format a ConNode as "Name field0 field1 ...". Returns a tagged string.
pub extern "C" fn synoema_show_con(ptr: i64) -> i64 {
    let base = (ptr & !CON_TAG) as *const i64;
    let arity    = unsafe { *base.add(1) } as usize;
    let name_ptr = unsafe { *base.add(2) } as *const u8;
    let name_len = unsafe { *base.add(3) } as usize;
    let name_str = if name_len == 0 || name_ptr.is_null() {
        "Con".to_string()
    } else {
        let bytes = unsafe { std::slice::from_raw_parts(name_ptr, name_len) };
        std::str::from_utf8(bytes).unwrap_or("Con").to_string()
    };
    let mut s = name_str;
    for i in 0..arity {
        let field = unsafe { *base.add(4 + i) };
        s.push(' ');
        let fs = format_val(field);
        if fs.contains(' ') {
            s.push('('); s.push_str(&fs); s.push(')');
        } else {
            s.push_str(&fs);
        }
    }
    alloc_str(&s)
}

/// map f xs — apply a 1-arg closure to each list element. Returns mapped list.
/// closure_ptr: pointer to ClosureNode { fn_ptr: i64, env_ptr: i64 }
pub extern "C" fn synoema_map(closure_ptr: i64, list: i64) -> i64 {
    let fn_ptr_val = unsafe { *(closure_ptr as *const i64) };
    let env_ptr    = unsafe { *((closure_ptr + 8) as *const i64) };
    let fn_ptr: extern "C" fn(i64, i64) -> i64 = unsafe { std::mem::transmute(fn_ptr_val) };

    // Collect elements first to avoid mutating the list while iterating
    let mut elems: Vec<i64> = Vec::new();
    let mut cur = list;
    while cur != 0 {
        elems.push(unsafe { (*(cur as *const ListNode)).head });
        cur = unsafe { (*(cur as *const ListNode)).tail };
    }
    // Build result right-to-left (preserves order)
    let mut result = 0i64;
    for &elem in elems.iter().rev() {
        result = synoema_cons(fn_ptr(env_ptr, elem), result);
    }
    result
}

/// filter p xs — keep only elements where predicate p returns non-zero. Returns filtered list.
pub extern "C" fn synoema_filter(closure_ptr: i64, list: i64) -> i64 {
    let fn_ptr_val = unsafe { *(closure_ptr as *const i64) };
    let env_ptr    = unsafe { *((closure_ptr + 8) as *const i64) };
    let fn_ptr: extern "C" fn(i64, i64) -> i64 = unsafe { std::mem::transmute(fn_ptr_val) };

    let mut elems: Vec<i64> = Vec::new();
    let mut cur = list;
    while cur != 0 {
        elems.push(unsafe { (*(cur as *const ListNode)).head });
        cur = unsafe { (*(cur as *const ListNode)).tail };
    }
    let mut result = 0i64;
    for &elem in elems.iter().rev() {
        if fn_ptr(env_ptr, elem) != 0 {
            result = synoema_cons(elem, result);
        }
    }
    result
}

/// foldl f init xs — left fold with a curried 2-arg closure.
/// f is called as: partial = f acc → then partial elem → new_acc.
pub extern "C" fn synoema_foldl(f_closure: i64, init: i64, list: i64) -> i64 {
    let fn_ptr_val = unsafe { *(f_closure as *const i64) };
    let env_ptr    = unsafe { *((f_closure + 8) as *const i64) };
    let fn_ptr: extern "C" fn(i64, i64) -> i64 = unsafe { std::mem::transmute(fn_ptr_val) };

    let mut elems: Vec<i64> = Vec::new();
    let mut cur = list;
    while cur != 0 {
        elems.push(unsafe { (*(cur as *const ListNode)).head });
        cur = unsafe { (*(cur as *const ListNode)).tail };
    }
    let mut acc = init;
    for elem in elems {
        // Curried call: f(acc) returns a partial closure, then partial(elem) returns new acc
        let partial = fn_ptr(env_ptr, acc);
        let fn_ptr2_val = unsafe { *(partial as *const i64) };
        let env_ptr2    = unsafe { *((partial + 8) as *const i64) };
        let fn_ptr2: extern "C" fn(i64, i64) -> i64 = unsafe { std::mem::transmute(fn_ptr2_val) };
        acc = fn_ptr2(env_ptr2, elem);
    }
    acc
}

/// concatMap: apply a closure to each list element, concat resulting lists.
/// closure_ptr: pointer to ClosureNode { fn_ptr: i64, env_ptr: i64 }
/// list: linked list (nil = 0, otherwise ListNode { head, tail })
pub extern "C" fn synoema_concatmap(closure_ptr: i64, list: i64) -> i64 {
    // Collect elements (to avoid recursion stack overflow on large lists)
    let mut elems: Vec<i64> = Vec::new();
    let mut cur = list;
    while cur != 0 {
        let head = unsafe { (*(cur as *const ListNode)).head };
        elems.push(head);
        cur = unsafe { (*(cur as *const ListNode)).tail };
    }

    // Process right-to-left, prepending to result (gives correct left-to-right order)
    let fn_ptr_val = unsafe { *(closure_ptr as *const i64) };
    let env_ptr = unsafe { *((closure_ptr + 8) as *const i64) };
    let fn_ptr: extern "C" fn(i64, i64) -> i64 = unsafe { std::mem::transmute(fn_ptr_val) };

    let mut result = 0i64; // nil
    for &elem in elems.iter().rev() {
        let mapped = fn_ptr(env_ptr, elem); // returns a list
        result = synoema_concat(mapped, result);
    }
    result
}

// ── Concurrency: Channel Primitives (Phase C) ─────────────────────────────────
//
// JIT channels are Phase C sequential stubs: chan/send/recv work in
// a single-threaded JIT context. Concurrent JIT spawn is Phase D.
//
// ChanNodeRt is heap-allocated via Box and leaked (Box::into_raw) so the
// channel survives for the lifetime of the program. arena_reset() does NOT
// free channel memory — a known limitation of Phase C.

use std::sync::{Mutex as SyncMutex};
use std::sync::mpsc as mpsc_rt;

#[repr(C)]
struct ChanNodeRt {
    sender:   SyncMutex<mpsc_rt::Sender<i64>>,
    receiver: SyncMutex<mpsc_rt::Receiver<i64>>,
}

/// Create a new channel. Returns a raw pointer (as i64) to the ChanNodeRt.
#[unsafe(no_mangle)]
pub extern "C" fn synoema_chan_new() -> i64 {
    let (tx, rx) = mpsc_rt::channel::<i64>();
    let node = Box::new(ChanNodeRt {
        sender:   SyncMutex::new(tx),
        receiver: SyncMutex::new(rx),
    });
    Box::into_raw(node) as i64
}

/// Send a value on a channel. Returns 0 (Unit).
#[unsafe(no_mangle)]
pub extern "C" fn synoema_chan_send(chan: i64, val: i64) -> i64 {
    let node = chan as *const ChanNodeRt;
    if let Ok(guard) = unsafe { (*node).sender.lock() } {
        let _ = guard.send(val);
    }
    0 // Unit
}

/// Receive a value from a channel (blocking). Returns the received i64.
#[unsafe(no_mangle)]
pub extern "C" fn synoema_chan_recv(chan: i64) -> i64 {
    let node = chan as *const ChanNodeRt;
    match unsafe { (*node).receiver.lock() } {
        Ok(guard) => guard.recv().unwrap_or(0),
        Err(_) => 0,
    }
}

/// Runtime error handler for non-exhaustive patterns and similar failures.
/// Prints the error message to stderr and exits with code 1.
pub extern "C" fn synoema_match_error(ptr: i64, len: i64) -> i64 {
    let msg = if ptr != 0 && len > 0 {
        let slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };
        std::str::from_utf8(slice).unwrap_or("runtime error")
    } else {
        "runtime error"
    };
    eprintln!("Runtime error: {}", msg);
    std::process::exit(1);
}
