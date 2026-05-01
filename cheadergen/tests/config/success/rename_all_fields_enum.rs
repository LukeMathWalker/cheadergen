#[cheadergen::config(rename_all_fields = "camelCase")]
#[repr(C)]
pub enum Message {
    Plain,
    WithData {
        message_text: String,
        sender_id: u32,
    },
}

fn main() {}
