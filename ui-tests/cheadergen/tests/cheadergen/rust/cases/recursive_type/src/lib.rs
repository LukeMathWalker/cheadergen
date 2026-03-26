//! Recursive (self-referential) types: a struct and an enum each contain
//! a pointer back to themselves. The generated header must emit a full
//! definition (not an opaque forward declaration) for each type.

#[repr(C)]
pub struct ListNode {
    pub value: i32,
    pub next: *const ListNode,
}

#[repr(C)]
pub enum TreeNode {
    Leaf,
    Branch {
        value: i32,
        left: *mut TreeNode,
        right: *mut TreeNode,
    },
}

#[unsafe(no_mangle)]
pub extern "C" fn use_list(node: ListNode) -> ListNode {
    node
}

#[unsafe(no_mangle)]
pub extern "C" fn use_tree(node: TreeNode) -> TreeNode {
    node
}
