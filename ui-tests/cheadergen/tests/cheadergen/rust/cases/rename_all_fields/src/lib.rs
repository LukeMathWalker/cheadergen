//! `#[cheadergen::config(rename_all_fields = "...")]` on an enum bulk-renames
//! the fields inside each struct variant. Variant names themselves are
//! unaffected (use `rename_all` for those).

#[cheadergen::config(export, rename_all_fields = "camelCase")]
#[repr(C)]
pub enum Message {
    Plain,
    WithBody {
        message_text: u32,
        sender_id: u32,
    },
}
