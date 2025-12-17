#[cfg(not(target_arch = "wasm32"))]
pub mod handler;

pub use local_chat_core::message::{Message, Peer, ChatEvent};
#[cfg(not(target_arch = "wasm32"))]
pub use handler::MessageHandler;
