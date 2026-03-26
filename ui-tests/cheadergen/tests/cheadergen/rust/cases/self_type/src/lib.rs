//! Self-referential types using `Self` keyword: a struct and an enum each
//! contain a pointer to `Self`. The resolver must bind `Self` to the
//! containing type so that codegen emits the correct pointer type.

#[repr(C)]
pub struct ListNode {
    pub value: i32,
    pub next: *const Self,
}

#[repr(C)]
pub enum TreeNode {
    Leaf,
    Branch {
        value: i32,
        left: *mut Self,
        right: *mut Self,
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
