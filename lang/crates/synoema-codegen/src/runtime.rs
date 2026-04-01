//! Synoema JIT Runtime Library
//!
//! Provides heap-allocated data structures for JIT-compiled code.
//! All functions use `extern "C"` ABI and operate on i64 values.
//!
//! List representation: linked list of (head, tail) nodes.
//! - Nil = 0 (null pointer)
//! - Cons(h, t) = pointer to heap-allocated ListNode { head: i64, tail: i64 }
//!
//! Note: This is an MVP runtime. It leaks memory (no GC).
//! Phase 10.1 will add region-based memory management.

use std::alloc::{alloc, Layout};

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
    let layout = Layout::new::<ListNode>();
    unsafe {
        let ptr = alloc(layout) as *mut ListNode;
        if ptr.is_null() {
            panic!("synoema_cons: allocation failed");
        }
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

/// Concatenate two lists: concat(a, b) = a ++ b
pub extern "C" fn synoema_concat(a: i64, b: i64) -> i64 {
    if a == 0 {
        return b;
    }
    // Copy list `a`, appending `b` at the end
    let head = synoema_head(a);
    let tail = synoema_tail(a);
    synoema_cons(head, synoema_concat(tail, b))
}

/// Get length of a list
pub extern "C" fn synoema_length(list: i64) -> i64 {
    let mut count = 0i64;
    let mut cur = list;
    while cur != 0 {
        count += 1;
        cur = unsafe { (*(cur as *const ListNode)).tail };
    }
    count
}

// ── Display ─────────────────────────────────────────────

/// Print a value (integer or list). Returns the value.
pub extern "C" fn synoema_show_val(val: i64) -> i64 {
    // Heuristic: if val looks like a valid heap pointer, treat as list
    // On 64-bit Linux, heap pointers are typically > 0x1000 and page-aligned-ish
    // Integers are typically small
    // This is a hack — proper solution needs tagged values (Phase 10)
    if is_likely_list_ptr(val) {
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

/// Heuristic: does this i64 look like a heap pointer?
/// Heap pointers on Linux are > 4096 and have certain alignment.
/// Small integers (< 100000) are almost certainly not pointers.
fn is_likely_list_ptr(val: i64) -> bool {
    val > 100_000 && val % 2 == 0 // heap allocs are at least 2-byte aligned
}

// ── String Support (minimal) ────────────────────────────

/// Create a string from a static pointer (returned by data section)
pub extern "C" fn synoema_print_str_ptr(ptr: i64, len: i64) -> i64 {
    if ptr != 0 && len > 0 {
        let slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };
        if let Ok(s) = std::str::from_utf8(slice) {
            println!("{}", s);
        }
    }
    0
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

// ── Filter/Map (implemented in runtime for efficiency) ──

/// Filter a list by a predicate function pointer
/// pred: extern "C" fn(i64) -> i64 (returns 0 or 1)
pub extern "C" fn synoema_filter(pred_ptr: i64, list: i64) -> i64 {
    if list == 0 { return 0; }
    let pred: extern "C" fn(i64) -> i64 = unsafe { std::mem::transmute(pred_ptr) };
    let node = unsafe { &*(list as *const ListNode) };
    let rest = synoema_filter(pred_ptr, node.tail);
    if pred(node.head) != 0 {
        synoema_cons(node.head, rest)
    } else {
        rest
    }
}
